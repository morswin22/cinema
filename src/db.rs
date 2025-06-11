use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::sql_types::Integer;
use diesel::MysqlConnection;
use dotenvy::dotenv;
use std::env;
use diesel::dsl::{count_star};
use crate::models::{
    Movie, NewReservation, Reservation, ReservationDetail,
    Room, Schedule, ReservationChangeset,
};
use crate::schema::{movies, reservation, rooms, schedule};

pub type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;
pub type MysqlPooledConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

pub fn establish_connection_pool() -> MysqlPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder()
        .max_size(20)
        .min_idle(Some(5))
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}

pub fn get_movie_by_id(conn: &mut MysqlConnection, movie_id: i32) -> QueryResult<Movie> {
    movies::table.find(movie_id).first(conn)
}

pub fn get_all_movies(conn: &mut MysqlConnection) -> QueryResult<Vec<Movie>> {
    movies::table.load::<Movie>(conn)
}

pub fn get_schedules_with_details(
    conn: &mut MysqlConnection,
) -> QueryResult<Vec<(Schedule, Movie, Room)>> {
    schedule::table
        .inner_join(movies::table)
        .inner_join(rooms::table)
        .select((Schedule::as_select(), Movie::as_select(), Room::as_select()))
        .load::<(Schedule, Movie, Room)>(conn)
}

pub fn create_reservation(
    conn: &mut MysqlConnection,
    new_reservation: NewReservation,
) -> QueryResult<i32> {
    conn.transaction(|conn| {
        diesel::insert_into(reservation::table)
            .values(&new_reservation)
            .execute(conn)?;

        if check_if_capacity_exceeded(conn, new_reservation.schedule_id)? {
            return Err(diesel::result::Error::RollbackTransaction);
        }

        Ok(0)
    })
}

pub fn update_reservation(
    conn: &mut MysqlConnection,
    reservation_id: i32,
    changeset: ReservationChangeset,
) -> QueryResult<Reservation> {
    use crate::schema::reservation::dsl::*;

    conn.transaction(|conn| {
        let rows_affected = diesel::update(reservation.find(reservation_id))
            .set(&changeset)
            .execute(conn)?;

        if rows_affected == 0 {
            Err(diesel::result::Error::NotFound)
        } else {
            if let Some(form_schedule_id) = changeset.schedule_id {
                if check_if_capacity_exceeded(conn, form_schedule_id)? {
                    return Err(diesel::result::Error::RollbackTransaction);
                }
            }

            reservation.find(reservation_id).first(conn)
        }
    })
}

pub fn get_reservation_by_id(
    conn: &mut MysqlConnection,
    res_id: i32,
) -> QueryResult<Reservation> {
    reservation::table.find(res_id).first(conn)
}

pub fn get_reservations_by_user_id(
    conn: &mut MysqlConnection,
    user_id_param: i32,
) -> QueryResult<Vec<Reservation>> {
    use crate::schema::reservation::dsl::*;
    reservation
        .filter(user_id.eq(user_id_param))
        .load::<Reservation>(conn)
}

pub fn get_reservations_with_details(
    conn: &mut PooledConnection<ConnectionManager<MysqlConnection>>,
    user_id: i32,
) -> QueryResult<Vec<ReservationDetail>> {
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

pub fn check_if_users_reservation(conn: &mut MysqlConnection, res_ids: Vec<i32>, user_id_value: i32) -> QueryResult<bool> {
    use crate::schema::reservation::dsl::*;

    let res_ids_len = res_ids.len();
    let result = reservation
        .filter(id.eq_any(res_ids))
        .filter(user_id.eq(user_id_value))
        .select(count_star())
        .first::<i64>(conn).unwrap_or(0);

    Ok(result == res_ids_len as i64)
}

pub fn delete_reservation(conn: &mut MysqlConnection, res_id: i32) -> QueryResult<usize> {
    use crate::schema::reservation::dsl::*;
    diesel::delete(reservation.filter(id.eq(res_id))).execute(conn)
}

pub fn delete_multiple_reservations(
    conn: &mut MysqlConnection,
    res_ids: Vec<i32>
) -> QueryResult<usize> {
    use crate::schema::reservation::dsl::*;
    diesel::delete(reservation.filter(id.eq_any(res_ids))).execute(conn)
}

pub fn check_if_capacity_exceeded(conn: &mut MysqlConnection, schedule_id: i32) -> QueryResult<bool> {
    #[derive(QueryableByName)]
    struct ExceededResult {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_exceeded: bool,
    }

    let query = diesel::sql_query(
        "SELECT EXISTS (
            SELECT 1
            FROM schedule s
            JOIN rooms r ON s.room_id = r.id
            LEFT JOIN reservation res ON s.id = res.schedule_id
            WHERE s.id = ?
            GROUP BY s.id, r.capacity
            HAVING COUNT(res.id) > r.capacity
        ) AS is_exceeded"
    ).bind::<Integer, _>(schedule_id);

    let result = query.load::<ExceededResult>(conn)?
        .into_iter()
        .next()
        .map(|r| r.is_exceeded)
        .unwrap_or(false);

    Ok(result)
}
