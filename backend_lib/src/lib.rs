#![feature(proc_macro_hygiene, decl_macro)]

use database::postgres::PostgresConnection;
use rocket::{response::content::Html, serde::json::{Value, json}};

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

// TODO: write unit tests for databases, APIs, etc...
// TODO: fuzz test the APIs?

#[catch(404)]
fn not_found() -> Html<String> {
    Html(String::from(
        "You have reached the backend API for BetterVote, a ranked-choice voting website. <br />
        It is currently in development; see <a href='https://github.com/AnnikaCodes/bettervote'>the GitHub repository</a> for more information and API documentation."
    ))
}

#[catch(422)]
fn bad_json() -> Value {
    json!({
        "success": false,
        "error": "You must provide valid JSON with all required fields for this endpoint specified. \
        Refer to the API documentation at TODO: for more information.",
    })
}

#[launch]
pub fn rocket() -> _ {
    rocket::build()
        .register("/", catchers![not_found, bad_json])
        .attach(PostgresConnection::fairing())
        .mount("/", api::routes())
}
