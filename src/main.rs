use askama::Template;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Router, serve};
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::{Level, info};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/movies", get(movies_handler))
        .fallback(|| async { AppError::NotFound })
        .layer(TraceLayer::new_for_http());

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

async fn movies_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug)]
    struct Movie {
        id: i32,
        title: String,
        year: i32,
        director: String,
    }

    #[derive(Debug, Template)]
    #[template(path = "movies.html")]
    struct Tmpl {
        movies: Vec<Movie>
    }

    let movies = vec![
        Movie {
            id: 1,
            title: "Movie 1".to_string(),
            year: 1999,
            director: "Abstrakcja".to_string()
        },
        Movie {
            id: 2,
            title: "Movie 2".to_string(),
            year: 2031,
            director: "Belzebub".to_string()
        },
    ];

    let template = Tmpl {
        movies
    };
    Ok(Html(template.render()?))
}