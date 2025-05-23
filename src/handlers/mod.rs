// Re-export specific handler modules
pub mod movies;
pub mod reservations;
pub mod auth;

// Re-export the index handler from main.rs if it's still there,
// or move it to a dedicated 'common' or 'root' handler module.
// For now, let's assume it's moved to a new common module or directly in main.rs if simple.
// Given the refactoring, it's better to move it here.
use axum::response::{Html, IntoResponse};
use askama::Template;
use crate::templates_structs::IndexTemplate;
use crate::AppError; // Import AppError
use tower_sessions::Session;
use crate::models::User;
use crate::SESSION_USER_KEY;

pub async fn index_handler(
    session: Session
) -> Result<impl IntoResponse, AppError> {
    let template = IndexTemplate {
        user_option: session.get(SESSION_USER_KEY).await.unwrap().unwrap_or(None)
    };
    Ok(Html(template.render()?))
}
