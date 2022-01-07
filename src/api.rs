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

#[post("/poll/<pollid>/vote", data = "<data>")]
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

#[post("/create")]
fn create() -> Value {
    todo!();
    json!({ "success": true })
}

#[get("/poll/<pollid>")]
fn poll_info(pollid: String) -> Value {
    todo!();
}


mod tests {
    use rocket::local::blocking::Client;
    use rocket::http::Status;
    use rocket::serde::json::{json, Json, Value};

    use crate::database::postgres::PostgresConnection;

    fn create_client() -> Client {
        Client::tracked(crate::rocket()).expect("valid rocket instance")
    }
    // TODO: test errors/bad input
    #[test]
    fn vote_happy_path() {
        let client = create_client();
        todo!();
    }

    #[test]
    fn create_happy_path() {
        let client = create_client();

        let response_create_1 = client.post("/create").json(&json!({
            "name": "Test Poll 1",
            "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
            "duration": 100000i32,
            "numWinners": 1i32,
        })).dispatch();
        assert_eq!(&response_create_1.status(), &Status::Ok);
        let test_poll_1_json = response_create_1.into_json::<Value>().unwrap();
        let test_poll_1_id = test_poll_1_json["id"].as_str().unwrap();
        assert!(!test_poll_1_id.is_empty());
        assert_eq!(test_poll_1_json["success"], true);

        let response_create_2 = client.post("/create").json(&json!({
            "name": "Test Poll 2",
            "candidates": ["Candidate 3", "Candidate 5", "Candidate 4"],
            "duration": 10000i32,
            "numWinners": 2i32,
            "id": "testID",
            "protection": "ip",
        })).dispatch();
        assert_eq!(&response_create_2.status(), &Status::Ok);
        let test_poll_2_json = response_create_2.into_json::<Value>().unwrap();
        assert_eq!(test_poll_2_json["id"], "testID");
        assert_eq!(test_poll_2_json["success"], true);

        let response_info_1 = client.get(format!("/poll/{}", test_poll_1_id)).dispatch();
        assert_eq!(response_info_1.status(), Status::Ok);
        let response_info_1_json = response_info_1.into_json::<Value>().unwrap();
        assert_eq!(response_info_1_json["success"], true);
        assert_eq!(response_info_1_json["name"], "Test Poll 1");
        let candidates_1: Vec<&str> = response_info_1_json["candidates"].as_array().unwrap().iter().map(|c| c.as_str().unwrap()).collect();
        assert_eq!(candidates_1, vec!["Candidate 1", "Candidate 2", "Candidate 3"]);
        assert_eq!(response_info_1_json["endingTime"].as_u64().unwrap() - response_info_1_json["creationTime"].as_u64().unwrap(), 100000);
        assert_eq!(response_info_1_json["numWinners"], 1i32);
        assert_eq!(response_info_1_json["protection"], Value::Null);
        assert_eq!(response_info_1_json["ended"], false);
        assert_eq!(response_info_1_json["numWinners"], 1i32);

        todo!();
    }

    #[test]
    fn poll_info_happy_path() {
        let client = create_client();
        // need to test both completed and ongoing polls
        todo!();
    }
}
