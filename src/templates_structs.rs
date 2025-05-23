use askama::Template;
use chrono::NaiveDateTime;
use crate::models::{Movie, ReservationDetail, Reservation, Room, Schedule, User};
use crate::AppError; // Import AppError

// Define the templates
#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub user_option: Option<User>
}

#[derive(Template)]
#[template(path = "reservations_list.html")]
pub struct ReservationsListTemplate {
    pub reservations: Vec<ReservationDetail>,
}

#[derive(Template)]
#[template(path = "reservation_form.html")]
pub struct ReservationFormTemplate {
    pub reservation: Option<Reservation>,
    pub users: Vec<User>,
    pub schedules: Vec<(Schedule, Movie, Room)>,
    pub error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub error_message: String,
    pub debug_info: String,
}

#[derive(Template)]
#[template(path = "movies.html")]
pub struct MoviesTemplate {
    pub movies: Vec<Movie>,
}

#[derive(Template)]
#[template(path = "movie.html")]
pub struct MovieTemplate {
    pub movie: Movie,
}
