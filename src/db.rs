use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sql_types::Integer;
use diesel::MysqlConnection;
use dotenvy::dotenv;
use std::env;
use chrono::NaiveDateTime;

use crate::models::{
    Movie, NewMovie, NewReservation, NewRoom, NewSchedule, NewUser, Reservation, ReservationDetail,
    Room, Schedule, User, ReservationChangeset, LastInsertId,
};
use crate::schema::{movies, reservation, rooms, schedule, users};

// Type alias for the connection pool
pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
pub type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

/// Establishes a database connection pool.
/// Reads the DATABASE_URL from the environment.
pub fn establish_connection_pool() -> MysqlPool {
    dotenv().ok(); // Load .env file
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true) // Test connections when retrieved from the pool
        .build(manager)
        .expect("Could not build connection pool")
}

/// Helper function to retrieve the last inserted ID for MySQL.
/// This is necessary because MySQL does not support the `RETURNING` clause.
fn get_last_insert_id(conn: &mut MysqlConnection) -> QueryResult<i32> {
    diesel::sql_query("SELECT LAST_INSERT_ID() as id;")
        .load::<LastInsertId>(conn)?
        .into_iter()
        .next()
        .map(|res| res.id as i32)
        .ok_or(diesel::result::Error::NotFound) // Handle case where ID is not found
}

// --- CRUD Functions for Users ---

/// Creates a new user in the database.
/// Returns the ID of the newly created user.
pub fn create_user(conn: &mut MysqlConnection, new_user: NewUser) -> QueryResult<i32> {
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(conn)?; // Execute the insert

    get_last_insert_id(conn) // Use the helper function
}

/// Finds a user by their ID.
pub fn get_user_by_id(conn: &mut MysqlConnection, user_id: i32) -> QueryResult<User> {
    users::table.find(user_id).first(conn)
}

/// Gets all users.
pub fn get_all_users(conn: &mut MysqlConnection) -> QueryResult<Vec<User>> {
    users::table.load::<User>(conn)
}


// --- CRUD Functions for Movies ---

/// Creates a new movie in the database.
/// Returns the ID of the newly created movie.
pub fn create_movie(conn: &mut MysqlConnection, new_movie: NewMovie) -> QueryResult<i32> {
    diesel::insert_into(movies::table)
        .values(&new_movie)
        .execute(conn)?; // Execute the insert

    get_last_insert_id(conn) // Use the helper function
}

/// Finds a movie by its ID.
pub fn get_movie_by_id(conn: &mut MysqlConnection, movie_id: i32) -> QueryResult<Movie> {
    movies::table.find(movie_id).first(conn)
}

/// Gets all movies.
pub fn get_all_movies(conn: &mut MysqlConnection) -> QueryResult<Vec<Movie>> {
    movies::table.load::<Movie>(conn)
}

// --- CRUD Functions for Rooms ---

/// Creates a new room in the database.
/// Returns the ID of the newly created room.
pub fn create_room(conn: &mut MysqlConnection, new_room: NewRoom) -> QueryResult<i32> {
    diesel::insert_into(rooms::table)
        .values(&new_room)
        .execute(conn)?; // Execute the insert

    get_last_insert_id(conn) // Use the helper function
}

/// Finds a room by its ID.
pub fn get_room_by_id(conn: &mut MysqlConnection, room_id: i32) -> QueryResult<Room> {
    rooms::table.find(room_id).first(conn)
}

/// Gets all rooms.
pub fn get_all_rooms(conn: &mut MysqlConnection) -> QueryResult<Vec<Room>> {
    rooms::table.load::<Room>(conn)
}

// --- CRUD Functions for Schedule ---

/// Creates a new schedule entry in the database.
/// Returns the ID of the newly created schedule entry.
pub fn create_schedule(conn: &mut MysqlConnection, new_schedule: NewSchedule) -> QueryResult<i32> {
    diesel::insert_into(schedule::table)
        .values(&new_schedule)
        .execute(conn)?; // Execute the insert

    get_last_insert_id(conn) // Use the helper function
}

/// Finds a schedule entry by its ID.
pub fn get_schedule_by_id(conn: &mut MysqlConnection, schedule_id: i32) -> QueryResult<Schedule> {
    schedule::table.find(schedule_id).first(conn)
}

/// Gets all schedule entries.
pub fn get_all_schedules(conn: &mut MysqlConnection) -> QueryResult<Vec<Schedule>> {
    schedule::table.load::<Schedule>(conn)
}

/// Gets all schedules with movie and room details.
pub fn get_schedules_with_details(
    conn: &mut MysqlConnection,
) -> QueryResult<Vec<(Schedule, Movie, Room)>> {
    schedule::table
        .inner_join(movies::table)
        .inner_join(rooms::table)
        .select((Schedule::as_select(), Movie::as_select(), Room::as_select()))
        .load::<(Schedule, Movie, Room)>(conn)
}

// --- CRUD Functions for Reservations ---

/// Creates a new reservation.
/// Returns the ID of the newly created reservation.
pub fn create_reservation(
    conn: &mut MysqlConnection,
    new_reservation: NewReservation,
) -> QueryResult<i32> {
    diesel::insert_into(reservation::table)
        .values(&new_reservation)
        .execute(conn)?; // Execute the insert

    get_last_insert_id(conn) // Use the helper function
}

/// Updates an existing reservation.
/// Returns the updated reservation.
pub fn update_reservation(
    conn: &mut MysqlConnection,
    reservation_id: i32,
    changeset: ReservationChangeset, // Now accepts ReservationChangeset
) -> QueryResult<Reservation> {
    use crate::schema::reservation::dsl::*;

    let rows_affected = diesel::update(reservation.find(reservation_id))
        .set(changeset) // Pass the changeset struct directly
        .execute(conn)?; // Use execute and handle the result

    if rows_affected == 0 {
        // If no rows were affected, it means the reservation ID was not found.
        // Return a Diesel error indicating no record found.
        Err(diesel::result::Error::NotFound)
    } else {
        // If successful, fetch the updated reservation.
        reservation.find(reservation_id).first(conn)
    }
}

/// Retrieves a single reservation by its ID.
pub fn get_reservation_by_id(
    conn: &mut MysqlConnection,
    res_id: i32,
) -> QueryResult<Reservation> {
    reservation::table.find(res_id).first(conn)
}

/// Retrieves all reservations.
pub fn get_all_reservations(conn: &mut MysqlConnection) -> QueryResult<Vec<Reservation>> {
    reservation::table.load::<Reservation>(conn)
}

/// Retrieves reservations made by a specific user.
pub fn get_reservations_by_user_id(
    conn: &mut MysqlConnection,
    user_id_param: i32,
) -> QueryResult<Vec<Reservation>> {
    use crate::schema::reservation::dsl::*;
    reservation
        .filter(user_id.eq(user_id_param))
        .load::<Reservation>(conn)
}

/// Retrieves all reservations with full details (who made it, movie, room, schedule).
pub fn get_reservations_with_details(
    conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
    user_id: i32,
) -> QueryResult<Vec<ReservationDetail>> {
    // Use a raw SQL query with explicit aliases for QueryableByName
    diesel::sql_query(
        "SELECT
            r.id as reservation_id,
            u.email as user_email,
            m.title as movie_title,
            ro.label as room_label,
            s.date as schedule_date
        FROM reservation r
        INNER JOIN users u ON r.user_id = u.id
        INNER JOIN schedule s ON r.schedule_id = s.id
        INNER JOIN movies m ON s.movie_id = m.id
        INNER JOIN rooms ro ON s.room_id = ro.id
        WHERE u.id = ?"
    )
        .bind::<Integer, _>(user_id)
        .load::<ReservationDetail>(conn)
}

/// Counts the number of existing reservations for a given schedule.
pub fn get_reservations_count_for_schedule(
    conn: &mut MysqlConnection,
    schedule_id_param: i32,
) -> QueryResult<i64> {
    use crate::schema::reservation::dsl::*;
    reservation
        .filter(schedule_id.eq(schedule_id_param))
        .count()
        .get_result(conn)
}

/// Deletes a single reservation by its ID.
/// Returns the number of deleted rows (should be 1 if successful).
pub fn delete_reservation(conn: &mut MysqlConnection, res_id: i32) -> QueryResult<usize> {
    use crate::schema::reservation::dsl::*;
    // TODO: only delete reservations of the currently logged in user
    diesel::delete(reservation.filter(id.eq(res_id))).execute(conn)
}

/// Deletes multiple reservations by a list of IDs.
/// Returns the number of deleted rows.
pub fn delete_multiple_reservations(
    conn: &mut MysqlConnection,
    res_ids: Vec<i32>,
) -> QueryResult<usize> {
    use crate::schema::reservation::dsl::*;
    // TODO: only delete reservations of the currently logged in user
    diesel::delete(reservation.filter(id.eq_any(res_ids))).execute(conn)
}

pub fn create_mock_data(conn: &mut MysqlConnection) -> QueryResult<()> {
    use tracing::info;

    info!("Attempting to create mock data...");

    // Create Users
    let user1 = NewUser { email: "alice@example.com", password: "password123" };
    let user2 = NewUser { email: "bob@example.com", password: "securepassword" };

    let user1_id = create_user(conn, user1)?;
    let user2_id = create_user(conn, user2)?;
    info!("Created users with IDs: {}, {}", user1_id, user2_id);

    // Create Movies
    let movie1 = NewMovie { title: "The Rust Movie", year: 2023, director: "Ferris" };
    let movie2 = NewMovie { title: "Async Adventures", year: 2024, director: "Tokio" };

    let movie1_id = create_movie(conn, movie1)?;
    let movie2_id = create_movie(conn, movie2)?;
    info!("Created movies with IDs: {}, {}", movie1_id, movie2_id);

    // Create Rooms
    let room1 = NewRoom { capacity: 100, label: "Screen 1" };
    let room2 = NewRoom { capacity: 50, label: "Screen 2" };
    let room3 = NewRoom { capacity: 1, label: "Screen 3" };

    let room1_id = create_room(conn, room1)?;
    let room2_id = create_room(conn, room2)?;
    let room3_id = create_room(conn, room3)?;
    info!("Created rooms with IDs: {}, {}, {}", room1_id, room2_id, room3_id);

    // Create Schedules
    let schedule1 = NewSchedule {
        movie_id: movie1_id,
        room_id: room1_id,
        date: NaiveDateTime::parse_from_str("2025-06-01 18:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    };
    let schedule2 = NewSchedule {
        movie_id: movie2_id,
        room_id: room2_id,
        date: NaiveDateTime::parse_from_str("2025-06-01 20:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    };
    let schedule3 = NewSchedule {
        movie_id: movie1_id,
        room_id: room2_id,
        date: NaiveDateTime::parse_from_str("2025-06-02 14:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    };

    let schedule4 = NewSchedule {
        movie_id: movie1_id,
        room_id: room3_id,
        date: NaiveDateTime::parse_from_str("2025-06-02 15:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    };

    let schedule1_id = create_schedule(conn, schedule1)?;
    let schedule2_id = create_schedule(conn, schedule2)?;
    let schedule3_id = create_schedule(conn, schedule3)?;
    let schedule4_id = create_schedule(conn, schedule4)?;
    info!("Created schedules with IDs: {}, {}, {}, {}", schedule1_id, schedule2_id, schedule3_id, schedule4_id);

    // Create Reservations (optional, linking users and schedules)
    let reservation1 = NewReservation {
        user_id: user1_id,
        schedule_id: schedule1_id,
    };
    let reservation2 = NewReservation {
        user_id: user2_id,
        schedule_id: schedule2_id,
    };

    create_reservation(conn, reservation1)?;
    create_reservation(conn, reservation2)?;
    info!("Created sample reservations.");


    Ok(())
}