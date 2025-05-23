use axum::{
    extract::{Path, Query, State, Form},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post, delete},
    Router, serve,
};
use askama::Template;
use chrono::NaiveDateTime;
use serde::Deserialize;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::{Level, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool};

use tower_http::trace::TraceLayer;
use dotenvy::dotenv;
use std::env;

mod db;
mod models;
mod schema;

use db::{establish_connection_pool, MysqlPool}; // Use the pool from db.rs
use models::{Movie, NewReservation, ReservationDetail, ReservationChangeset}; // Import ReservationChangeset

// Define the templates
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[derive(Template)]
#[template(path = "reservations_list.html")]
struct ReservationsListTemplate {
    reservations: Vec<ReservationDetail>,
}

#[derive(Template)]
#[template(path = "reservation_form.html")]
struct ReservationFormTemplate {
    reservation: Option<models::Reservation>,
    users: Vec<models::User>,
    schedules: Vec<(models::Schedule, models::Movie, models::Room)>,
    error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    // Changed to accept a String directly, avoiding complex enum processing by Askama
    error_message: String,
    debug_info: String,
}

#[derive(Template)]
#[template(path = "movies.html")]
struct MoviesTemplate {
    movies: Vec<Movie>,
}

#[derive(Template)]
#[template(path = "movie.html")]
struct MovieTemplate {
    movie: Movie,
}

// Form data structures for HTMX requests
#[derive(Deserialize)]
struct CreateReservationForm {
    user_id: i32,
    schedule_id: i32,
}

#[derive(Deserialize)]
struct UpdateReservationForm {
    user_id: Option<i32>,
    schedule_id: Option<i32>,
}

#[derive(Deserialize)]
struct DeleteMultipleReservationsForm {
    #[serde(rename = "reservation_ids[]")] // HTMX sends array inputs like this
    reservation_ids: Vec<i32>,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // Establish database connection pool using the function from db.rs
    let pool = establish_connection_pool();
    let shared_pool = Arc::new(pool); // Wrap in Arc for sharing across threads

    // Acquire a connection to create mock data
    let mut conn = shared_pool.get().expect("Failed to get database connection from pool");

    // Create mock data at startup
    match db::create_mock_data(&mut conn) {
        Ok(_) => info!("Mock data created successfully (or already exists)."),
        Err(e) => tracing::error!("Failed to create mock data: {:?}", e),
    }

    // Build our application with routes
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/movies", get(movies_handler))
        .route("/movie/{movie_id}", get(movie_handler)) // Changed to :movie_id for Path extractor
        .route("/reservations", get(list_reservations))
        .route("/reservations/new", get(show_create_reservation_form))
        .route("/reservations", post(create_reservation))
        .route("/reservations/edit/{id}", get(show_update_reservation_form))
        .route("/reservations/{id}", post(update_reservation)) // HTMX uses POST for form submission
        .route("/reservations/{id}", delete(delete_reservation)) // For single deletion
        .route("/reservations/bulk_delete", post(delete_multiple_reservations)) // For multiple deletions via form
        .fallback(|| async { AppError::NotFound })
        .layer(TraceLayer::new_for_http())
        .with_state(shared_pool); // Use .with_state instead of Extension for Axum 0.7+

    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .map_err(Error::Bind)?;

    if let Ok(addr) = listener.local_addr() {
        info!("Listening on http://{addr}/");
    }
    serve(listener, app).await.map_err(Error::Run)
}

#[derive(displaydoc::Display, thiserror::Error, Debug)]
enum Error {
    /// could not bind socket
    Bind(#[source] std::io::Error),
    /// could not run server
    Run(#[source] std::io::Error),
}

/// This enum contains any error that could occur while handling an incoming request.
#[derive(Debug, displaydoc::Display, thiserror::Error)]
enum AppError {
    /// not found
    NotFound,
    /// could not render template
    Render(#[from] askama::Error),
    /// Database error: {0}
    Database(#[from] diesel::result::Error),
    /// Failed to get database connection from pool: {0}
    PoolError(String),
    /// Bad Request: {0}
    BadRequest(String),
}

/// This is your error handler
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        };
        let tmpl = ErrorTemplate {
            error_message: self.to_string(), // Convert error to String for display
            debug_info: format!("{:?}", self), // Provide debug info as a string
        };
        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            // Fallback for when error template itself fails to render
            (status, format!("Error: {}", tmpl.error_message)).into_response()
        }
    }
}

async fn index_handler() -> Result<Html<String>, AppError> {
    let template = IndexTemplate {};
    Ok(Html(template.render()?))
}

async fn movies_handler(
    State(pool): State<Arc<MysqlPool>>,
) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let movies = db::get_all_movies(&mut conn).map_err(AppError::Database)?;

    let template = MoviesTemplate { movies };
    Ok(Html(template.render()?))
}

async fn movie_handler(
    State(pool): State<Arc<MysqlPool>>,
    Path(movie_id): Path<i32>,
) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let movie = db::get_movie_by_id(&mut conn, movie_id).map_err(|e| match e {
        diesel::result::Error::NotFound => AppError::NotFound,
        _ => AppError::Database(e),
    })?;

    let template = MovieTemplate { movie };
    Ok(Html(template.render()?))
}

/// Handler to list reservations, typically for HTMX partial updates.
async fn list_reservations(State(pool): State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let reservations = db::get_reservations_with_details(&mut conn)
        .map_err(AppError::Database)?;

    let template = ReservationsListTemplate { reservations };
    Ok(Html(template.render()?))
}

/// Handler to show the form for creating a new reservation.
async fn show_create_reservation_form(State(pool): State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let users = db::get_all_users(&mut conn).map_err(AppError::Database)?;
    let schedules = db::get_schedules_with_details(&mut conn).map_err(AppError::Database)?;

    let template = ReservationFormTemplate {
        reservation: None,
        users,
        schedules,
        error_message: None,
    };
    Ok(Html(template.render()?))
}

/// Handler to create a new reservation from form data.
async fn create_reservation(
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<CreateReservationForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let new_reservation = NewReservation {
        user_id: form.user_id,
        schedule_id: form.schedule_id,
    };

    match db::create_reservation(&mut conn, new_reservation) {
        Ok(_) => {
            Ok(list_reservations(State(pool)).await?.into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create reservation: {:?}", e);
            let users = db::get_all_users(&mut conn).unwrap_or_default();
            let schedules = db::get_schedules_with_details(&mut conn).unwrap_or_default();
            let template = ReservationFormTemplate {
                reservation: None,
                users,
                schedules,
                error_message: Some(format!("Failed to create reservation: {}", e)),
            };
            Ok((StatusCode::BAD_REQUEST, Html(template.render()?)).into_response())
        }
    }
}

/// Handler to show the form for updating an existing reservation.
async fn show_update_reservation_form(
    Path(id): Path<i32>,
    State(pool): State<Arc<MysqlPool>>,
) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let reservation = db::get_reservation_by_id(&mut conn, id).map_err(|e| match e {
        diesel::result::Error::NotFound => AppError::NotFound,
        _ => AppError::Database(e),
    })?;

    let users = db::get_all_users(&mut conn).map_err(AppError::Database)?;
    let schedules = db::get_schedules_with_details(&mut conn).map_err(AppError::Database)?;

    let template = ReservationFormTemplate {
        reservation: Some(reservation),
        users,
        schedules,
        error_message: None,
    };
    Ok(Html(template.render()?))
}

/// Handler to update an existing reservation from form data.
async fn update_reservation(
    Path(id): Path<i32>,
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<UpdateReservationForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let changeset = ReservationChangeset {
        user_id: form.user_id,
        schedule_id: form.schedule_id,
    };

    match db::update_reservation(&mut conn, id, changeset) {
        Ok(_) => {
            Ok(list_reservations(State(pool)).await?.into_response())
        }
        Err(e) => {
            tracing::error!("Failed to update reservation {}: {:?}", id, e);
            let reservation = db::get_reservation_by_id(&mut conn, id).ok();
            let users = db::get_all_users(&mut conn).unwrap_or_default();
            let schedules = db::get_schedules_with_details(&mut conn).unwrap_or_default();
            let template = ReservationFormTemplate {
                reservation,
                users,
                schedules,
                error_message: Some(format!("Failed to update reservation: {}", e)),
            };
            Ok((StatusCode::BAD_REQUEST, Html(template.render()?)).into_response())
        }
    }
}

/// Handler to delete a single reservation.
async fn delete_reservation(
    Path(id): Path<i32>,
    State(pool): State<Arc<MysqlPool>>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    match db::delete_reservation(&mut conn, id) {
        Ok(_) => {
            Ok(list_reservations(State(pool)).await?.into_response())
        }
        Err(e) => {
            tracing::error!("Failed to delete reservation {}: {:?}", id, e);
            Err(AppError::Database(e))
        }
    }
}

/// Handler to delete multiple reservations from form data.
async fn delete_multiple_reservations(
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<DeleteMultipleReservationsForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    if form.reservation_ids.is_empty() {
        return Err(AppError::BadRequest("No reservations selected for deletion.".to_string()));
    }

    match db::delete_multiple_reservations(&mut conn, form.reservation_ids) {
        Ok(_) => {
            Ok(list_reservations(State(pool)).await?.into_response())
        }
        Err(e) => {
            tracing::error!("Failed to delete multiple reservations: {:?}", e);
            Err(AppError::Database(e))
        }
    }
}
