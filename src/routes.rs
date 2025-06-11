use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use crate::db::MysqlPool;
use crate::handlers::{movies, reservations};
use crate::handlers;

pub fn app_router(pool: Arc<MysqlPool>) -> Router {
    Router::new()
        .route("/", get(handlers::index_handler))
        .nest("/movies", movie_routes())
        .nest("/reservations", reservation_routes())
        .merge(auth_routes())
        .with_state(pool)
}

fn movie_routes() -> Router<Arc<MysqlPool>> {
    Router::new()
        .route("/", get(movies::movies_handler))
        .route("/{movie_id}", get(movies::movie_handler))
}

fn auth_routes() -> Router<Arc<MysqlPool>> {
    Router::new()
        .route("/register", get(handlers::auth::show_register).post(handlers::auth::handle_register))
        .route("/login", get(handlers::auth::show_login).post(handlers::auth::handle_login))
        .route("/logout", get(handlers::auth::logout))
}

fn reservation_routes() -> Router<Arc<MysqlPool>> {
    Router::new()
        .route("/", get(reservations::list_reservations_handler))
        .route("/new", get(reservations::show_create_reservation_form))
        .route("/", post(reservations::create_reservation))
        .route("/edit/{id}", get(reservations::show_update_reservation_form))
        .route("/{id}", post(reservations::update_reservation))
        .route("/{id}", delete(reservations::delete_reservation))
        .route("/bulk_delete", post(reservations::delete_multiple_reservations))
}
