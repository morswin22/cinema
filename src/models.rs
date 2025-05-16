use diesel::prelude::*;
use chrono::NaiveDateTime;
use crate::schema::*;

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Queryable, Identifiable, Selectable, Debug)]
#[diesel(table_name = movies)]
pub struct Movie {
    pub id: i32,
    pub title: String,
    pub year: i32,
    pub director: String,
}

#[derive(Insertable)]
#[diesel(table_name = movies)]
pub struct NewMovie<'a> {
    pub title: &'a str,
    pub year: i32,
    pub director: &'a str,
}

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = rooms)]
pub struct Room {
    pub id: i32,
    pub capacity: i32,
    pub label: String,
}

#[derive(Insertable)]
#[diesel(table_name = rooms)]
pub struct NewRoom<'a> {
    pub capacity: i32,
    pub label: &'a str,
}

#[derive(Queryable, Identifiable, Selectable)]
#[diesel(table_name = schedule)]
pub struct Schedule {
    pub id: i32,
    pub movie_id: i32,
    pub room_id: i32,
    pub date: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = schedule)]
pub struct NewSchedule {
    pub movie_id: i32,
    pub room_id: i32,
    pub date: NaiveDateTime,
}

#[derive(Queryable, Identifiable, Associations, Selectable)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Schedule))]
#[diesel(table_name = reservation)]
pub struct Reservation {
    pub id: i32,
    pub user_id: i32,
    pub schedule_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = reservation)]
pub struct NewReservation {
    pub user_id: i32,
    pub schedule_id: i32,
}
