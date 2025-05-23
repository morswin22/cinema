// Re-export specific handler modules
pub mod movies;
pub mod reservations;

// Re-export the index handler from main.rs if it's still there,
// or move it to a dedicated 'common' or 'root' handler module.
// For now, let's assume it's moved to a new common module or directly in main.rs if simple.
// Given the refactoring, it's better to move it here.
use axum::response::Html;
use askama::Template;
use crate::templates_structs::IndexTemplate;
use crate::AppError; // Import AppError

pub async fn index_handler() -> Result<Html<String>, AppError> {
    let template = IndexTemplate {};
    Ok(Html(template.render()?))
}
