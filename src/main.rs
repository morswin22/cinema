use axum::{
    extract::{Path, Query, State, Form},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post, delete},
    Router, serve,
};
use askama::Template;
use chrono::NaiveDateTime;
use diesel::result::Error::NotFound;
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
use models::User;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use tower_sessions::cookie::time::Duration;

// Import the new modules
mod db;
mod models;
mod schema;
mod routes; // New: Contains all route definitions
mod handlers; // New: Contains all handler functions
mod templates_structs; // New: Contains all Askama template structs
mod forms;

use db::{establish_connection_pool, MysqlPool}; // Use the pool from db.rs
use templates_structs::ErrorTemplate; // Import ErrorTemplate from its new location

const SESSION_USER_KEY: &str = "USER";

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

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(3)));

    // Build our application with routes from the new `routes` module
    let app = routes::app_router(shared_pool.clone()) // Pass the shared pool to the router builder
        .fallback(|| async { AppError::NotFound })
        .layer(TraceLayer::new_for_http())
        .layer(session_layer);
    // .with_state(shared_pool); // This call is now handled inside routes::app_router

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
pub enum AppError { // Made public for use in handlers
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
            error_message: self.to_string(),
            debug_info: format!("{:?}", self),
        };
        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, format!("Error: {}", tmpl.error_message)).into_response()
        }
    }
}
