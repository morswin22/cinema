use diesel::prelude::*;
use crate::models::{NewMovie};

pub mod models;
pub mod schema;
pub mod db;

pub fn create_movie(conn: &mut MysqlConnection, new_movie: NewMovie) -> i32 {
    use crate::schema::movies;

    conn.transaction(|conn| {
        diesel::insert_into(movies::table)
            .values(&new_movie)
            .execute(conn)
            .expect("Error saving new movie");

        diesel::result::QueryResult::Ok(diesel::sql_query("SELECT LAST_INSERT_ID();").execute(conn).unwrap() as i32)
    }).unwrap()
}