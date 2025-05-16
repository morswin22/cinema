mod models;
mod schema;

use askama::Template;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Router, serve};
use diesel::result::Error::NotFound;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::{Level, info};
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use models::Movie;
use axum::extract::Extension;
use dotenvy::dotenv;
use std::env;

type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/movies", get(movies_handler))
        .route("/movie/{movie_id}", get(movie_handler))
        .fallback(|| async { AppError::NotFound })
        .layer(TraceLayer::new_for_http())
        .layer(Extension(pool));

    // TODO: In a real application you would most likely read the configuration from a config file.
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .map_err(Error::Bind)?;

    if let Ok(addr) = listener.local_addr() {
        info!("Listening on http://{addr}/");
    }
    serve(listener, app).await.map_err(Error::Run)
}

#[derive(displaydoc::Display, pretty_error_debug::Debug, thiserror::Error)]
enum Error {
    /// could not bind socket
    Bind(#[source] std::io::Error),
    /// could not run server
    Run(#[source] std::io::Error),
}

/// This enum contains any error that could occur while handling an incoming request.
///
/// TODO: In a real application you would most likely have multiple error sources, e.g. database errors,
#[derive(Debug, displaydoc::Display, thiserror::Error)]
enum AppError {
    /// not found
    NotFound,
    /// could not render template
    Render(#[from] askama::Error),
}

/// This is your error handler
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // It uses an askama template to display its content.
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct Tmpl {
            err: AppError,
        }

        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let tmpl = Tmpl {
            err: self,
        };
        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}

async fn index_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "index.html")]
    struct Tmpl {
    }

    let template = Tmpl {
    };
    Ok(Html(template.render()?))
}

async fn movies_handler(
    Extension(pool): Extension<MysqlPool>,
) -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "movies.html")]
    struct Tmpl {
        movies: Vec<Movie>
    }

    use crate::schema::movies::dsl::*;

    let mut conn = pool.get().unwrap();

    let template = Tmpl {
        movies: movies.load(&mut conn).unwrap()
    };
    Ok(Html(template.render()?))
}

async fn movie_handler(
    Extension(pool): Extension<MysqlPool>,
    Path(movie_id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "movie.html")]
    struct Tmpl {
        movie: Movie
    }

    use crate::schema::movies::dsl::*;

    let mut conn = pool.get().unwrap();

    let template = Tmpl {
        movie: movies.find(movie_id).first(&mut conn).map_err(|_| AppError::NotFound)?
    };

    Ok(Html(template.render()?))
}