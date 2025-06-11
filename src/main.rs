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

mod db;
mod models;
mod schema;
mod routes;
mod handlers;
mod templates_structs;
mod forms;
mod extractors;

use db::{establish_connection_pool, MysqlPool};
use templates_structs::ErrorTemplate;

const SESSION_USER_KEY: &str = "USER";

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let pool = establish_connection_pool();
    let shared_pool = Arc::new(pool);

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(3)));

    let app = routes::app_router(shared_pool.clone())
        .fallback(|| async { AppError::NotFound })
        .layer(TraceLayer::new_for_http())
        .layer(session_layer);

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

#[derive(Debug, displaydoc::Display, thiserror::Error)]
pub enum AppError {
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
    /// User login error
    UserLoginError,
    /// User register error
    UserRegisterError,
    /// UnauthorizedError
    UnauthorizedError
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PoolError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::UserLoginError => StatusCode::IM_A_TEAPOT,
            AppError::UserRegisterError => StatusCode::IM_A_TEAPOT,
            AppError::UnauthorizedError => StatusCode::UNAUTHORIZED
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
