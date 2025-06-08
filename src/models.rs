use diesel::prelude::*;
use chrono::NaiveDateTime;
use crate::schema::*;
use serde::{Deserialize, Serialize};
use diesel::deserialize::QueryableByName; // Import QueryableByName
use serde;

// User Model
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

// Movie Model
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

// Room Model
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = rooms)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Room {
    pub id: i32,
    pub capacity: i32,
    pub label: String,
}

#[derive(Insertable)]
#[diesel(table_name = rooms)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewRoom<'a> {
    pub capacity: i32,
    pub label: &'a str,
}

// Schedule Model
#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Clone)]
#[diesel(table_name = schedule)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Schedule {
    pub id: i32,
    pub movie_id: i32,
    pub room_id: i32,
    pub date: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = schedule)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct NewSchedule {
    pub movie_id: i32,
    pub room_id: i32,
    pub date: NaiveDateTime,
}

// Reservation Model
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

// New struct for updating Reservation
#[derive(AsChangeset, Deserialize, Serialize, Debug)]
#[diesel(table_name = reservation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct ReservationChangeset {
    pub user_id: Option<i32>,
    pub schedule_id: Option<i32>,
}

// Struct for displaying reservation details (joins)
// Changed to QueryableByName for raw SQL queries
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
    #[diesel(sql_type = diesel::sql_types::Timestamp)] // Use Timestamp for NaiveDateTime
    pub schedule_date: NaiveDateTime,
}

// New struct to handle LAST_INSERT_ID() for MySQL
#[derive(Debug, QueryableByName)]
pub struct LastInsertId {
    #[diesel(sql_type = diesel::sql_types::Bigint)] // Specify the SQL type
    pub id: i64, // Use i64 as LAST_INSERT_ID() returns BIGINT
}

#[derive(Debug)]
pub struct ScheduleDisplayInfo {
    pub schedule: Schedule,
    pub movie: Movie,
    pub room: Room,
    pub available_seats: i32,
}