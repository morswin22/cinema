use axum::{
    extract::{Path, State, Form},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use std::sync::Arc;
use askama::Template;
use serde::Deserialize;
use serde::de::{self, Deserializer, SeqAccess, Visitor};
use std::fmt;
use diesel::{serialize::IsNull::No, Connection};
use diesel::result::{Error as DieselError, DatabaseErrorKind};
use crate::{db::MysqlPool, extractors::session_user::RequiredUser};
use crate::models::{NewReservation, ReservationDetail, ReservationChangeset, ScheduleDisplayInfo};
use crate::{db, AppError};
use crate::db::check_if_users_reservation;
use crate::templates_structs::{ReservationsListTemplate, ReservationFormTemplate};

#[derive(Deserialize)]
pub struct CreateReservationForm {
    pub schedule_id: i32,
}

#[derive(Deserialize)]
pub struct UpdateReservationForm {
    pub schedule_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteFormData {
    pub reservation_ids: String,
}


pub async fn list_reservations_handler(RequiredUser(user): RequiredUser, pool: State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    list_reservations(RequiredUser(user), pool, None)
}

pub fn list_reservations(RequiredUser(user): RequiredUser, State(pool): State<Arc<MysqlPool>>, error_message: Option<String>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let reservations = db::get_reservations_with_details(&mut conn, user.id)
        .map_err(AppError::Database)?;

    let template = ReservationsListTemplate {
        reservations, error_message,
    };
    Ok(Html(template.render()?))
}

pub async fn show_create_reservation_form(RequiredUser(user): RequiredUser, State(pool): State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let schedules_with_details = db::get_schedules_with_details(&mut conn).map_err(AppError::Database)?;

    let mut schedules_display_info: Vec<ScheduleDisplayInfo> = Vec::new();
    for (schedule, movie, room) in schedules_with_details {
        let current_reservations_count = db::get_reservations_count_for_schedule(&mut conn, schedule.id)
            .map_err(AppError::Database)?;
        let available_seats = room.capacity - current_reservations_count as i32;
        schedules_display_info.push(ScheduleDisplayInfo {
            schedule: schedule.clone(), // Clone to avoid move error
            movie: movie.clone(),       // Clone to avoid move error
            room: room.clone(),         // Clone to avoid move error
            available_seats,
        });
    }

    let template = ReservationFormTemplate {
        reservation: None,
        schedules: schedules_display_info,
    };
    Ok(Html(template.render()?))
}

pub async fn create_reservation(
    RequiredUser(user): RequiredUser,
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<CreateReservationForm>,
) -> Result<Response, AppError> {

    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let new_reservation = NewReservation {
        user_id: user.id,
        schedule_id: form.schedule_id,
    };

    match db::create_reservation(&mut conn, new_reservation) {
        Ok(_) => {
            Ok(list_reservations(RequiredUser(user), State(pool), None).into_response())
        }
        Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, info)) => {
            let user_friendly_error = Some("This user already has a reservation for the selected schedule.".to_string());
            tracing::warn!("Unique constraint violated: {:?}", info);
            Ok(list_reservations(RequiredUser(user), State(pool), user_friendly_error).into_response())
        }
        Err(DieselError::RollbackTransaction) => {
            let error_message = Some(format!(
                "Room capacity exceeded for schedule ID {}",
                form.schedule_id
            ));

            Ok(list_reservations(RequiredUser(user), State(pool), error_message).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create reservation: {:?}", e);
            let error_message = Some(format!("Failed to create reservation: {}", e));
            Ok(list_reservations(RequiredUser(user), State(pool), error_message).into_response())
        }
    }
}

pub async fn show_update_reservation_form(
    RequiredUser(user): RequiredUser,
    Path(id): Path<i32>,
    State(pool): State<Arc<MysqlPool>>,
) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let reservation = db::get_reservation_by_id(&mut conn, id).map_err(|e| match e {
        diesel::result::Error::NotFound => AppError::NotFound,
        _ => AppError::Database(e),
    })?;

    let schedules_with_details = db::get_schedules_with_details(&mut conn).map_err(AppError::Database)?;

    let mut schedules_display_info: Vec<ScheduleDisplayInfo> = Vec::new();
    for (schedule, movie, room) in schedules_with_details {
        let mut current_reservations_count = db::get_reservations_count_for_schedule(&mut conn, schedule.id)
            .map_err(AppError::Database)?;

        if reservation.schedule_id == schedule.id {
            current_reservations_count = current_reservations_count.saturating_sub(1);
        }

        let available_seats = room.capacity - current_reservations_count as i32;
        schedules_display_info.push(ScheduleDisplayInfo {
            schedule: schedule.clone(),
            movie: movie.clone(),
            room: room.clone(),
            available_seats,
        });
    }

    let template = ReservationFormTemplate {
        reservation: Some(reservation),
        schedules: schedules_display_info,
    };
    Ok(Html(template.render()?))
}

pub async fn update_reservation(
    RequiredUser(user): RequiredUser,
    Path(id): Path<i32>,
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<UpdateReservationForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let changeset = ReservationChangeset {
        user_id: Some(user.id),
        schedule_id: Some(form.schedule_id),
    };

    match db::update_reservation(&mut conn, id, changeset) {
        Ok(_) => {
            Ok(list_reservations(RequiredUser(user), State(pool), None).into_response())
        }
        Err(DieselError::RollbackTransaction) => {
            let error_message = Some(format!(
                "Room capacity exceeded for new schedule ID {}",
                form.schedule_id
            ));
            Ok(list_reservations(RequiredUser(user), State(pool), error_message).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to update reservation {}: {:?}", id, e);
            let error_message = Some(format!("Failed to update reservation: {}", e));
            Ok(list_reservations(RequiredUser(user), State(pool), error_message).into_response())
        }
    }
}

pub async fn delete_reservation(
    RequiredUser(user): RequiredUser,
    Path(id): Path<i32>,
    State(pool): State<Arc<MysqlPool>>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    if !check_if_users_reservation(&mut conn, vec![id], user.id)? {
        return Err(AppError::UserLoginError);
    }

    match db::delete_reservation(&mut conn, id) {
        Ok(_) => {
            Ok(list_reservations(RequiredUser(user), State(pool), None).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to delete reservation {}: {:?}", id, e);
            Err(AppError::Database(e))
        }
    }
}

pub async fn delete_multiple_reservations(
    RequiredUser(user): RequiredUser,
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

    if !check_if_users_reservation(&mut conn, reservation_ids.clone(), user.id)? {
        return Err(AppError::UserLoginError);
    }

    match db::delete_multiple_reservations(&mut conn, reservation_ids) {
        Ok(_) => Ok(list_reservations(RequiredUser(user), State(pool), None).into_response()),
        Err(e) => {
            tracing::error!("Failed to delete multiple reservations: {:?}", e);
            Err(AppError::Database(e))
        }
    }
}
