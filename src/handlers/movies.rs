use axum::{
    extract::{Path, State},
    response::Html,
};
use std::sync::Arc;
use askama::Template;

use crate::db::MysqlPool;
use crate::models::Movie;
use crate::{db, AppError}; // Import db functions and AppError
use crate::templates_structs::{MoviesTemplate, MovieTemplate}; // Import template structs

pub async fn movies_handler(
    State(pool): State<Arc<MysqlPool>>,
) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let movies = db::get_all_movies(&mut conn).map_err(AppError::Database)?;

    let template = MoviesTemplate { movies };
    Ok(Html(template.render()?))
}

pub async fn movie_handler(
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
