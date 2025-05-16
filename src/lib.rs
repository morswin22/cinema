use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use crate::models::{NewMovie};

pub mod models;
pub mod schema;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_movie(conn: &mut MysqlConnection, new_movie: NewMovie) {
    use crate::schema::movies;

    diesel::insert_into(movies::table)
        .values(&new_movie)
        .execute(conn)
        .expect("Error saving new movie");
}