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
use diesel::serialize::IsNull::No;
use crate::db::MysqlPool;
use crate::models::{NewReservation, ReservationDetail, ReservationChangeset, ScheduleDisplayInfo};
use crate::{db, AppError};
use crate::templates_structs::{ReservationsListTemplate, ReservationFormTemplate};

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


pub async fn list_reservations_handler(pool: State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    list_reservations(pool, None)
}

/// Handler to list reservations, typically for HTMX partial updates.
pub fn list_reservations(State(pool): State<Arc<MysqlPool>>, error_message: Option<String>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let reservations = db::get_reservations_with_details(&mut conn)
        .map_err(AppError::Database)?;

    let template = ReservationsListTemplate {
        reservations, error_message,
    };
    Ok(Html(template.render()?))
}

/// Handler to show the form for creating a new reservation.
pub async fn show_create_reservation_form(State(pool): State<Arc<MysqlPool>>) -> Result<Html<String>, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let users = db::get_all_users(&mut conn).map_err(AppError::Database)?;
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
        users,
        schedules: schedules_display_info,
    };
    Ok(Html(template.render()?))
}

/// Handler to create a new reservation from form data.
pub async fn create_reservation(
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<CreateReservationForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    // Capacity check
    let schedule = db::get_schedule_by_id(&mut conn, form.schedule_id)
        .map_err(AppError::Database)?;
    let room = db::get_room_by_id(&mut conn, schedule.room_id)
        .map_err(AppError::Database)?;
    let current_reservations_count = db::get_reservations_count_for_schedule(&mut conn, form.schedule_id)
        .map_err(AppError::Database)?;

    if current_reservations_count as i32 >= room.capacity {
        let users = db::get_all_users(&mut conn).unwrap_or_default();
        let schedules_with_details = db::get_schedules_with_details(&mut conn).unwrap_or_default();
        let mut schedules_display_info: Vec<ScheduleDisplayInfo> = Vec::new();
        for (s, m, r) in schedules_with_details {
            let count = db::get_reservations_count_for_schedule(&mut conn, s.id).unwrap_or(0);
            schedules_display_info.push(ScheduleDisplayInfo {
                schedule: s.clone(), // Clone to avoid move error
                movie: m.clone(),    // Clone to avoid move error
                room: r.clone(),     // Clone to avoid move error
                available_seats: r.capacity - count as i32
            });
        }

        let error_message = Some(format!(
            "Room capacity exceeded for schedule ID {}. Available seats: {}",
            form.schedule_id,
            room.capacity - current_reservations_count as i32
        ));

        return Ok(list_reservations(State(pool), error_message).into_response());
    }

    let new_reservation = NewReservation {
        user_id: form.user_id,
        schedule_id: form.schedule_id,
    };

    match db::create_reservation(&mut conn, new_reservation) {
        Ok(_) => {
            Ok(list_reservations(State(pool), None).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to create reservation: {:?}", e);
            let users = db::get_all_users(&mut conn).unwrap_or_default();
            let schedules_with_details = db::get_schedules_with_details(&mut conn).unwrap_or_default();
            let mut schedules_display_info: Vec<ScheduleDisplayInfo> = Vec::new();
            for (s, m, r) in schedules_with_details {
                let count = db::get_reservations_count_for_schedule(&mut conn, s.id).unwrap_or(0);
                schedules_display_info.push(ScheduleDisplayInfo {
                    schedule: s.clone(), // Clone to avoid move error
                    movie: m.clone(),    // Clone to avoid move error
                    room: r.clone(),     // Clone to avoid move error
                    available_seats: r.capacity - count as i32
                });
            }
            let error_message = Some(format!("Failed to create reservation: {}", e));
            Ok(list_reservations(State(pool), error_message).into_response())
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
    let schedules_with_details = db::get_schedules_with_details(&mut conn).map_err(AppError::Database)?;

    let mut schedules_display_info: Vec<ScheduleDisplayInfo> = Vec::new();
    for (schedule, movie, room) in schedules_with_details {
        let mut current_reservations_count = db::get_reservations_count_for_schedule(&mut conn, schedule.id)
            .map_err(AppError::Database)?;

        // If the reservation being updated is for this schedule, decrement the count by 1
        // as this slot is effectively "freed up" for the purpose of checking new capacity.
        if reservation.schedule_id == schedule.id {
            current_reservations_count = current_reservations_count.saturating_sub(1);
        }

        let available_seats = room.capacity - current_reservations_count as i32;
        schedules_display_info.push(ScheduleDisplayInfo {
            schedule: schedule.clone(), // Clone to avoid move error
            movie: movie.clone(),       // Clone to avoid move error
            room: room.clone(),         // Clone to avoid move error
            available_seats,
        });
    }

    let template = ReservationFormTemplate {
        reservation: Some(reservation),
        users,
        schedules: schedules_display_info,
    };
    Ok(Html(template.render()?))
}

/// Handler to update an existing reservation from form data.
pub async fn update_reservation(
    Path(id): Path<i32>,
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<UpdateReservationForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;

    let current_reservation = db::get_reservation_by_id(&mut conn, id)
        .map_err(AppError::Database)?;

    let mut new_schedule_id = current_reservation.schedule_id;
    if let Some(s_id) = form.schedule_id {
        new_schedule_id = s_id;
    }

    // Only perform capacity check if the schedule is being changed or if we need to re-validate
    if form.schedule_id.is_some() && new_schedule_id != current_reservation.schedule_id {
        let new_schedule = db::get_schedule_by_id(&mut conn, new_schedule_id)
            .map_err(AppError::Database)?;
        let new_room = db::get_room_by_id(&mut conn, new_schedule.room_id)
            .map_err(AppError::Database)?;
        let current_reservations_for_new_schedule = db::get_reservations_count_for_schedule(&mut conn, new_schedule_id)
            .map_err(AppError::Database)?;

        if current_reservations_for_new_schedule as i32 >= new_room.capacity {
            let reservation = db::get_reservation_by_id(&mut conn, id).ok();
            let users = db::get_all_users(&mut conn).unwrap_or_default();
            let schedules_with_details = db::get_schedules_with_details(&mut conn).unwrap_or_default();
            let mut schedules_display_info: Vec<ScheduleDisplayInfo> = Vec::new();
            for (s, m, r) in schedules_with_details {
                let mut count = db::get_reservations_count_for_schedule(&mut conn, s.id).unwrap_or(0);
                if let Some(res) = &reservation {
                    if res.schedule_id == s.id {
                        count = count.saturating_sub(1);
                    }
                }
                schedules_display_info.push(ScheduleDisplayInfo {
                    schedule: s.clone(), // Clone to avoid move error
                    movie: m.clone(),    // Clone to avoid move error
                    room: r.clone(),     // Clone to avoid move error
                    available_seats: r.capacity - count as i32
                });
            }

            let error_message = Some(format!(
                "Room capacity exceeded for new schedule ID {}. Available seats: {}",
                new_schedule_id,
                new_room.capacity - current_reservations_for_new_schedule as i32
            ));
            return Ok(list_reservations(State(pool), error_message).into_response());
        }
    }


    let changeset = ReservationChangeset {
        user_id: form.user_id,
        schedule_id: form.schedule_id,
    };

    match db::update_reservation(&mut conn, id, changeset) {
        Ok(_) => {
            Ok(list_reservations(State(pool), None).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to update reservation {}: {:?}", id, e);
            let reservation = db::get_reservation_by_id(&mut conn, id).ok();
            let users = db::get_all_users(&mut conn).unwrap_or_default();
            let schedules_with_details = db::get_schedules_with_details(&mut conn).unwrap_or_default();
            let mut schedules_display_info: Vec<ScheduleDisplayInfo> = Vec::new();
            for (s, m, r) in schedules_with_details {
                let mut count = db::get_reservations_count_for_schedule(&mut conn, s.id).unwrap_or(0);
                if let Some(res) = &reservation {
                    if res.schedule_id == s.id {
                        count = count.saturating_sub(1); // Adjust count if this is the current schedule of the reservation being edited
                    }
                }
                schedules_display_info.push(ScheduleDisplayInfo {
                    schedule: s.clone(), // Clone to avoid move error
                    movie: m.clone(),    // Clone to avoid move error
                    room: r.clone(),     // Clone to avoid move error
                    available_seats: r.capacity - count as i32
                });
            }
            let error_message = Some(format!("Failed to update reservation: {}", e));
            Ok(list_reservations(State(pool), error_message).into_response())
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
            Ok(list_reservations(State(pool), None).into_response())
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
        Ok(_) => Ok(list_reservations(State(pool), None).into_response()),
        Err(e) => {
            tracing::error!("Failed to delete multiple reservations: {:?}", e);
            Err(AppError::Database(e))
        }
    }
}
