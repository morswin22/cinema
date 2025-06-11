pub mod movies;
pub mod reservations;
pub mod auth;

use axum::response::{Html, IntoResponse};
use askama::Template;
use crate::extractors::session_user::OptionalUser;
use crate::templates_structs::IndexTemplate;
use crate::AppError;

pub async fn index_handler(
    OptionalUser(user_option): OptionalUser,
) -> Result<impl IntoResponse, AppError> {
    let template = IndexTemplate {
        user_option
    };
    Ok(Html(template.render()?))
}
