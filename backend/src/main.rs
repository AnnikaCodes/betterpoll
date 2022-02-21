#![feature(proc_macro_hygiene, decl_macro)]

use database::postgres::PostgresConnection;
use rocket::{response::content::Html, serde::json::{Value, json}, http::Method};

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;

#[cfg(not(fuzzing))]
mod api;
#[cfg(fuzzing)]
pub mod api;
mod database;
mod error;
mod poll;

#[catch(404)]
fn not_found() -> Html<String> {
    Html(String::from(
        "You have reached the backend API for BetterPoll, a ranked-choice voting website. <br />
        It is currently in development; see <a href='https://github.com/AnnikaCodes/betterpoll'>the GitHub repository</a> for more information and API documentation."
    ))
}

#[catch(422)]
fn bad_json() -> Value {
    json!({
        "success": false,
        "error": "You must provide valid JSON with all required fields for this endpoint specified. \
        Refer to the API documentation at https://github.com/AnnikaCodes/betterpoll#api for more information.",
    })
}

#[launch]
pub fn rocket() -> _ {
    dotenv::dotenv().ok();
    let cors_regex = std::env::var("ALLOWED_ORIGINS")
        .expect("The environment variable ALLOWED_ORIGINS must be set to a regular expression defining allowed origins for API access.");

    let cors = rocket_cors::CorsOptions {
        allowed_origins: rocket_cors::AllowedOrigins::some_regex(&[cors_regex]),
        allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
        ..Default::default()
    }
    .to_cors().unwrap();

    rocket::build()
        .register("/", catchers![not_found, bad_json])
        .attach(PostgresConnection::fairing())
        .attach(cors)
        .mount("/", api::routes())
}
