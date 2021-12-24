//! BetterVote API endpoints

use std::net::IpAddr;

use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::net::unix::SocketAddr;

use crate::database::postgres::PostgresConnection;
use crate::error::ErrorKind;
use crate::poll::RankedChoiceVote;

/// Returns all the routes that should be made available
pub fn routes() -> Vec<rocket::Route> {
    routes![vote, create, poll_info]
}

#[derive(Deserialize)]
struct VoteAPIRequestData {
    pub choices: Vec<String>,
}

fn handle_error(e: ErrorKind) -> Value {
    match e {
        ErrorKind::Internal(e) => {
            eprintln!("Error while adding a vote to a poll: {:?}", e);
            json!({ "error": "Sorry, an internal server error occured. The server's administrators have been notified.", "success": false })
        }
        ErrorKind::PubliclyVisible(e) => json!({ "error": format!("{}", e), "success": false }),
    }
}

#[post("/<pollid>/vote", data = "<data>")]
async fn vote(mut conn: PostgresConnection, pollid: String, data: Json<VoteAPIRequestData>, remote_addr: Option<IpAddr>) -> Value {
    let Json(request) = data;
    let voter_ip = match remote_addr {
        Some(ip) => ip,
        None => {
            return json!({ "error": "No IP address was provided.", "success": false });
        }
    };

    let vote = RankedChoiceVote {
        ranked_choices: request.choices,
        voter_ip,
    };

    let poll = match conn.get_poll_by_id(pollid.clone()).await {
        Ok(Some(poll)) => poll,
        Ok(None) => {
            return json!({
                "error": format!("No poll was found with the ID '{}'.", pollid),
                "success": false,
            })
        }
        Err(e) => return handle_error(e),
    };

    if poll.prohibit_double_vote_by_ip && poll.votes.iter().any(|v| v.voter_ip == vote.voter_ip) {
        return json!({
            "error": "You have already voted in this poll.",
            "success": false,
        });
    }

    match conn.add_vote_to_poll(pollid, vote).await {
        Ok(_) => json!({ "success": true }),
        Err(e) => handle_error(e),
    }
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
