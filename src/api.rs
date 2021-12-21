//! BetterVote API endpoints

use rocket::serde::json::{json, Value};

/// Returns all the routes that should be made available
pub fn routes() -> Vec<rocket::Route> {
    routes![vote, create, poll_info]
}

#[post("/<pollid>/vote")]
fn vote(pollid: String) -> Value {
    todo!();
    json!({ "success": true })
}

#[put("/<pollid>/create")]
fn create(pollid: String) -> Value {
    todo!();
    json!({ "success": true })
}

#[get("/<pollid>")]
fn poll_info(pollid: String) -> Value {
    todo!();
}
