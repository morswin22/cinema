use diesel::prelude::*;
use chrono::NaiveDateTime;
use Cinema::{create_movie, establish_connection};
use Cinema::models::*;

fn main() {
    use Cinema::schema::movies::dsl::*;

    let connection = &mut establish_connection();

    create_movie(connection, NewMovie {
        title: "Inception",
        year: 2010,
        director: "Christopher Nolan",
    });

    let inserted: Movie = movies.order(id.desc()).first(connection).unwrap();
    println!("{}", inserted.title);

    let results = movies
        .limit(5)
        .select(Movie::as_select())
        .load(connection)
        .expect("Error loading movies");

    println!("Displaying {} movies", results.len());
    for movie in results {
        println!("{}", movie.title);
    }
}
