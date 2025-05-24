use axum::{
    extract::{Path, State, Form},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use askama::Template;
use serde::Deserialize;

use crate::db::MysqlPool;
use crate::models::{NewReservation, ReservationDetail, ReservationChangeset};
use crate::{db, AppError}; // Import db functions and AppError
use crate::templates_structs::{ReservationsListTemplate, ReservationFormTemplate}; // Import template structs
use serde_with::{serde_as, OneOrMany};

// Form data structures for HTMX requests (moved here)
#[derive(Deserialize)]
pub struct CreateReservationForm {
    pub user_id: i32,
    pub schedule_id: i32,
}

#[derive(Deserialize)]
pub struct UpdateReservationForm {
    pub user_id: Option<i32>,
    pub schedule_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteFormData {
    pub reservation_ids: String,
}

/// Handler to list reservations, typically for HTMX partial updates.
pub async fn list_reservations(State(pool): State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let reservations = db::get_reservations_with_details(&mut conn)
        .map_err(AppError::Database)?;

    let template = ReservationsListTemplate { reservations };
    Ok(Html(template.render()?))
}

/// Handler to show the form for creating a new reservation.
pub async fn show_create_reservation_form(State(pool): State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let users = db::get_all_users(&mut conn).map_err(AppError::Database)?;
    let schedules = db::get_schedules_with_details(&mut conn).map_err(AppError::Database)?;

    let template = ReservationFormTemplate {
        reservation: None,
        users,
        schedules,
        error_message: None,
    };
    // Simplified return: Html<String> already implements IntoResponse
    Ok(Html(template.render()?))
}

//TODO: check capacity of rooms for reseracation and update of reservatiaon
/// Handler to create a new reservation from form data.
pub async fn create_reservation(
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
pub async fn show_update_reservation_form(
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
    // Simplified return: Html<String> already implements IntoResponse
    Ok(Html(template.render()?))
}

/// Handler to update an existing reservation from form data.
pub async fn update_reservation(
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
pub async fn delete_reservation(
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
pub async fn delete_multiple_reservations(
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<BulkDeleteFormData>,
) -> Result<Response, AppError> {
    let ids: Result<Vec<i32>, _> = form
        .reservation_ids
        .split(',')
        .map(str::parse::<i32>)
        .collect();

    let reservation_ids = match ids {
        Ok(v) => v,
        Err(_) => return Err(AppError::BadRequest("Invalid reservation ID format.".into())),
    };

    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    if reservation_ids.is_empty() {
        return Err(AppError::BadRequest("No reservations selected for deletion.".to_string()));
    }

    match db::delete_multiple_reservations(&mut conn, reservation_ids) {
        Ok(_) => Ok(list_reservations(State(pool)).await?.into_response()),
        Err(e) => {
            tracing::error!("Failed to delete multiple reservations: {:?}", e);
            Err(AppError::Database(e))
        }
    }
}

