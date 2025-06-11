use diesel::prelude::*;
use chrono::NaiveDateTime;
use crate::schema::*;
use serde::{Deserialize, Serialize};
use diesel::deserialize::QueryableByName;
use serde;

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = movies)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Movie {
    pub id: i32,
    pub title: String,
    pub year: i32,
    pub director: String,
    pub poster: String,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = rooms)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Room {
    pub id: i32,
    pub capacity: i32,
    pub label: String,
}

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = schedule)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Schedule {
    pub id: i32,
    pub movie_id: i32,
    pub room_id: i32,
    pub date: NaiveDateTime,
}

#[derive(Queryable, Identifiable, Associations, Selectable, Debug, PartialEq)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Schedule))]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Reservation {
    pub id: i32,
    pub user_id: i32,
    pub schedule_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewReservation {
    pub user_id: i32,
    pub schedule_id: i32,
}

#[derive(AsChangeset, Deserialize, Serialize, Debug)]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ReservationChangeset {
    pub user_id: Option<i32>,
    pub schedule_id: Option<i32>,
}

#[derive(Debug, QueryableByName)]
pub struct ReservationDetail {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub reservation_id: i32,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub user_email: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub movie_title: String,
    #[diesel(sql_type = diesel::sql_types::Varchar)]
    pub room_label: String,
    #[diesel(sql_type = diesel::sql_types::Timestamp)]
    pub schedule_date: NaiveDateTime,
}

#[derive(Debug)]
pub struct ScheduleDisplayInfo {
    pub schedule: Schedule,
    pub movie: Movie,
    pub room: Room,
    pub available_seats: i32,
}