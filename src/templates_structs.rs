use askama::Template;
use crate::models::{Movie, ReservationDetail, Reservation, ScheduleDisplayInfo, User};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub user_option: Option<User>
}

#[derive(Template)]
#[template(path = "reservations_list.html")]
pub struct ReservationsListTemplate {
    pub reservations: Vec<ReservationDetail>,
    pub error_message: Option<String>,
}

#[derive(Template)]
#[template(path = "reservation_form.html")]
pub struct ReservationFormTemplate {
    pub reservation: Option<Reservation>,
    pub schedules: Vec<ScheduleDisplayInfo>,
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
