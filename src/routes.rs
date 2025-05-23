use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;
use crate::db::MysqlPool;
use crate::handlers::{movies, reservations}; // Import handler modules
use crate::handlers; // Import handlers module to access index_handler

/// Creates and returns the main application router.
// The app_router function now returns a Router that is generic over the state type S.
// The actual state (Arc<MysqlPool>) will be applied in main.rs.
pub fn app_router(pool: Arc<MysqlPool>) -> Router {
    Router::new()
        .route("/", get(handlers::index_handler))
        .nest("/movies", movie_routes())
        .nest("/reservations", reservation_routes())
        .with_state(pool) // Apply the shared pool to the top-level router
}

/// Defines routes related to movies.
// movie_routes now explicitly returns Router<Arc<MysqlPool>>
fn movie_routes() -> Router<Arc<MysqlPool>> {
    Router::new()
        .route("/", get(movies::movies_handler))
        .route("/{movie_id}", get(movies::movie_handler)) // Corrected path parameter syntax
}

/// Defines routes related to reservations.
// reservation_routes now explicitly returns Router<Arc<MysqlPool>>
fn reservation_routes() -> Router<Arc<MysqlPool>> {
    Router::new()
        .route("/", get(reservations::list_reservations))
        .route("/new", get(reservations::show_create_reservation_form))
        .route("/", post(reservations::create_reservation))
        .route("/edit/{id}", get(reservations::show_update_reservation_form))
        .route("/{id}", post(reservations::update_reservation))
        .route("/{id}", delete(reservations::delete_reservation))
        .route("/bulk_delete", post(reservations::delete_multiple_reservations))
}
