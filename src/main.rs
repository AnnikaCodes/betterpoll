#![feature(proc_macro_hygiene, decl_macro)]

use rocket::response::content::Html;

#[macro_use]
extern crate rocket;

mod api;
mod database;
mod error;
mod poll;

// TODO: write unit tests for databases, APIs, etc...

#[catch(404)]
fn not_found() -> Html<String> {
    Html(String::from(
        "You have reached the backend API for BetterVote, a ranked-choice voting website. <br />
        It is currently in development; see <a href='https://github.com/AnnikaCodes/bettervote'>the GitHub repository</a> for more information and API documentation."
    ))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .register("/", catchers![not_found])
        .mount("/", api::routes())
}
