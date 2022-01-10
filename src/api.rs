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
async fn vote(
    mut conn: PostgresConnection,
    pollid: String,
    data: Json<VoteAPIRequestData>,
    remote_addr: Option<IpAddr>,
) -> Value {
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
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::serde::json::{json, Json, Value};

    use crate::database::postgres::PostgresConnection;
    use serial_test::serial;

    fn create_client() -> Client {
        Client::tracked(crate::rocket()).expect("valid rocket instance")
    }
    // TODO: test errors/bad input

    fn post(client: &Client, path: &str, data: Value) {
        let req = client.post(path).json(&data).dispatch();
        assert_eq!(req.status(), Status::Ok);
        assert_eq!(req.into_json::<Value>().unwrap()["success"], true);
    }

    #[test]
    #[serial]
    fn vote_happy_path() {
        let client = create_client();

        fn get_num_votes(c: &Client, id: &str) -> i64 {
            c.get(format!("/poll/{}", id))
                .dispatch()
                .into_json::<Value>()
                .unwrap()["numVotes"]
                .as_i64()
                .unwrap()
        }
        // TODO make this assert_eq a function
        post(
            &client,
            "/create",
            json!({
                "name": "Voting Test - Happy Path",
                "candidates": ["A", "B", "C", "D"],
                "duration": 10000i32,
                "numWinners": 1i32,
                "id": "vote_happy",
            }),
        );

        assert_eq!(get_num_votes(&client, "vote_happy"), 0);

        post(
            &client,
            "/poll/vote_happy/vote",
            json!({
                "choices": ["A", "B", "C", "D"],
            }),
        );
        assert_eq!(get_num_votes(&client, "vote_happy"), 1);

        post(
            &client,
            "/poll/vote_happy/vote",
            json!({
                "choices": ["D", "C", "B"],
            }),
        );
        assert_eq!(get_num_votes(&client, "vote_happy"), 2);
    }

    #[test]
    #[serial]
    fn vote_invalid_candidates() {
        let client = create_client();

        post(
            &client,
            "/create",
            json!({
                "name": "Voting Test - Invalid Candidates",
                "candidates": ["A", "B", "C", "D", "E"],
                "duration": 10000i32,
                "numWinners": 1i32,
                "id": "vote_invalid_candidates",
            }),
        );

        for bad_json in [
            // All invalid
            json!({ "choices": ["I'm not real"] }),
            // One invalid
            json!({ "choices": ["A", "I'm not real", "B"] }),
            // No candidates
            json!({ "choices": [] }),
            // Duplicate candidates
            json!({ "choices": ["A", "B", "A", "C"] }),
        ] {
            let json = client
                .post("/poll/vote_invalid_candidates/vote")
                .json(&bad_json)
                .dispatch()
                .into_json::<Value>()
                .unwrap();
            assert_eq!(json["success"], false);
            assert!(!json["error"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    #[serial]
    fn vote_nonexistent_poll() {
        let client = create_client();

        let json = client
            .post("/poll/vote_nonexistent_poll/vote")
            .json(&json!({ "choices": ["A", "B", "C", "D"] }))
            .dispatch()
            .into_json::<Value>()
            .unwrap();
        assert_eq!(json["success"], false);
        assert!(!json["error"].as_str().unwrap().is_empty());
    }

    #[test]
    fn vote_expired_poll() {
        let client = create_client();

        post(
            &client,
            "/create",
            json!({
                "name": "Voting Test - Expired Poll",
                "candidates": ["A", "B", "C", "D"],
                "duration": 1i32,
                "numWinners": 1i32,
                "id": "vote_expired",
            }),
        );
        let json_before_expiry = client
            .post("/poll/vote_expired/vote")
            .json(&json!({ "choices": ["A", "B", "C", "D"] }))
            .dispatch()
            .into_json::<Value>()
            .unwrap();
        assert_eq!(json_before_expiry["success"], true);

        std::thread::sleep(std::time::Duration::from_secs(1));

        let json_after_expiry = client
            .post("/poll/vote_expired/vote")
            .json(&json!({ "choices": ["A", "B", "C", "D"] }))
            .dispatch()
            .into_json::<Value>()
            .unwrap();
        assert_eq!(json_after_expiry["success"], false);
        assert!(!json_after_expiry["error"].as_str().unwrap().is_empty());
    }

    #[test]
    #[serial]
    fn create_happy_path() {
        let client = create_client();

        let response_create_1 = client
            .post("/create")
            .json(&json!({
                "name": "Test Poll 1",
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "duration": 100000i32,
                "numWinners": 1i32,
            }))
            .dispatch();
        assert_eq!(&response_create_1.status(), &Status::Ok);
        let test_poll_1_json = response_create_1.into_json::<Value>().unwrap();
        let test_poll_1_id = test_poll_1_json["id"].as_str().unwrap();
        assert!(!test_poll_1_id.is_empty());
        assert_eq!(test_poll_1_json["success"], true);

        let response_create_2 = client
            .post("/create")
            .json(&json!({
                "name": "Test Poll 2",
                "candidates": ["Candidate 3", "Candidate 5", "Candidate 4"],
                "duration": 10000i32,
                "numWinners": 2i32,
                "id": "testID",
                "protection": "ip",
            }))
            .dispatch();
        assert_eq!(&response_create_2.status(), &Status::Ok);
        let test_poll_2_json = response_create_2.into_json::<Value>().unwrap();
        assert_eq!(test_poll_2_json["id"], "testID");
        assert_eq!(test_poll_2_json["success"], true);

        let response_info_1 = client.get(format!("/poll/{}", test_poll_1_id)).dispatch();
        assert_eq!(response_info_1.status(), Status::Ok);
        let response_info_1_json = response_info_1.into_json::<Value>().unwrap();
        assert_eq!(response_info_1_json["success"], true);
        assert_eq!(response_info_1_json["name"], "Test Poll 1");
        let candidates_1: Vec<&str> = response_info_1_json["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        assert_eq!(
            candidates_1,
            vec!["Candidate 1", "Candidate 2", "Candidate 3"]
        );
        assert_eq!(
            response_info_1_json["endingTime"].as_u64().unwrap()
                - response_info_1_json["creationTime"].as_u64().unwrap(),
            100000
        );
        assert_eq!(response_info_1_json["numWinners"], 1i32);
        assert_eq!(response_info_1_json["protection"], Value::Null);
        assert_eq!(response_info_1_json["ended"], false);

        let response_info_2 = client.get("/poll/testID").dispatch();
        assert_eq!(response_info_2.status(), Status::Ok);
        let response_info_2_json = response_info_2.into_json::<Value>().unwrap();
        assert_eq!(response_info_2_json["success"], true);
        assert_eq!(response_info_2_json["name"], "Test Poll 2");
        let candidates_2: Vec<&str> = response_info_2_json["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        assert_eq!(
            candidates_2,
            vec!["Candidate 3", "Candidate 5", "Candidate 4"]
        );
        assert_eq!(
            response_info_2_json["endingTime"].as_u64().unwrap()
                - response_info_2_json["creationTime"].as_u64().unwrap(),
            10000
        );
        assert_eq!(response_info_2_json["numWinners"], 2i32);
        assert_eq!(response_info_2_json["protection"], "ip");
        assert_eq!(response_info_2_json["ended"], false);
    }

    #[test]
    #[serial]
    fn create_missing_params() {
        let client = create_client();

        for bad_json in [
            // No name
            json!({
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "duration": 100000i32,
                "numWinners": 1i32,
            }),
            // No numWinners
            json!({
                "name": "Test Poll 1",
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "duration": 100000i32,
            }),
            // No duration
            json!({
                "name": "Test Poll 1",
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "numWinners": 1i32,
            }),
            // No candidates
            json!({
                "name": "Test Poll 1",
                "duration": 100000i32,
                "numWinners": 1i32,
            }),
        ] {
            let response = client
                .post("/create")
                .json(&bad_json)
                .dispatch();
            assert_eq!(response.status(), Status::Ok);
            let json = response.into_json::<Value>().unwrap();
            assert_eq!(json["success"], false);
            assert!(!json["error"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    #[serial]
    fn create_id_exists() {
        let client = create_client();

        post(
            &client,
            "/create",
            json!({
                "name": "Test Poll 1",
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "duration": 100000i32,
                "numWinners": 1i32,
                "id": "create_already_exists",
            })
        );


        let response = client
            .post("/create")
            .json(&json!({
                "name": "A Different Name",
                "candidates": ["A"],
                "duration": 10000i32,
                "numWinners": 2i32,
                "id": "create_already_exists",
            }))
            .dispatch();
        let json = response.into_json::<Value>().unwrap();
        assert_eq!(json["success"], false);
        assert!(!json["error"].as_str().unwrap().is_empty());
    }

    #[test]
    #[serial]
    fn create_not_enough_candidates() {
        let client = create_client();

        let response = client
            .post("/create")
            .json(&json!({
                "name": "Test Poll 1",
                "candidates": ["Candidate 1"],
                "duration": 100000i32,
                "numWinners": 1i32,
            }))
            .dispatch();
        let json = response.into_json::<Value>().unwrap();
        assert_eq!(json["success"], false);
        assert!(!json["error"].as_str().unwrap().is_empty());
    }

    #[test]
    #[serial]
    fn create_negative_duration() {
        let client = create_client();

        let response = client
            .post("/create")
            .json(&json!({
                "name": "Test Poll 1",
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "duration": -2i32,
                "numWinners": 1i32,
            }))
            .dispatch();
        let json = response.into_json::<Value>().unwrap();
        assert_eq!(json["success"], false);
        assert!(!json["error"].as_str().unwrap().is_empty());
    }

    #[test]
    #[serial]
    fn create_bad_num_winners() {
        let client = create_client();

        for bad_num_winners in [-1i32, 0i32, 3i32, 5i32] {
            let response = client
                .post("/create")
                .json(&json!({
                    "name": "Test Poll",
                    "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                    "duration": 100000i32,
                    "numWinners": bad_num_winners,
                }))
                .dispatch();
            let json = response.into_json::<Value>().unwrap();
            assert_eq!(json["success"], false);
            assert!(!json["error"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    #[serial]
    fn create_bad_id() {
        let client = create_client();

        for bad_id in ["", " ", "a b", "a b c", "morethan32charactersaaaaaaaaaaaaa", ".-_", "a??b"] {
            let response = client
                .post("/create")
                .json(&json!({
                    "name": "Test Poll",
                    "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                    "duration": 100000i32,
                    "numWinners": 1i32,
                    "id": bad_id,
                }))
                .dispatch();
            let json = response.into_json::<Value>().unwrap();
            assert_eq!(json["success"], false);
            assert!(!json["error"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    #[serial]
    fn create_bad_protection() {
        let client = create_client();

        let response = client
            .post("/create")
            .json(&json!({
                "name": "Test Poll",
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "duration": 100000i32,
                "numWinners": 1i32,
                "protection": "invalid",
            }))
            .dispatch();
        let json = response.into_json::<Value>().unwrap();
        assert_eq!(json["success"], false);
        assert!(!json["error"].as_str().unwrap().is_empty());
    }


    #[test]
    #[serial]
    fn poll_info_happy_path() {
        let client = create_client();

        // ongoing
        post(
            &client,
            "/create",
            json!({
                "name": "Poll Info Test - Ongoing Happy Path",
                "candidates": ["A", "B"],
                "duration": 10000i32,
                "numWinners": 1i32,
                "id": "ongoing_happy",
            }),
        );

        let response_info_ongoing = client.get("/poll/ongoing_happy").dispatch();
        assert_eq!(response_info_ongoing.status(), Status::Ok);
        let response_info_ongoing_json = response_info_ongoing.into_json::<Value>().unwrap();
        assert_eq!(response_info_ongoing_json["success"], true);
        assert_eq!(
            response_info_ongoing_json["name"],
            "Poll Info Test - Ongoing Happy Path"
        );
        let candidates_ongoing: Vec<&str> = response_info_ongoing_json["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        assert_eq!(candidates_ongoing, vec!["A", "B"]);
        assert_eq!(
            response_info_ongoing_json["endingTime"].as_u64().unwrap()
                - response_info_ongoing_json["creationTime"].as_u64().unwrap(),
            10000
        );
        assert_eq!(response_info_ongoing_json["numWinners"], 2i32);
        assert_eq!(response_info_ongoing_json["numVotes"], 0i32);
        assert_eq!(response_info_ongoing_json["protection"], Value::Null);
        assert_eq!(response_info_ongoing_json["ended"], false);

        // ended
        post(
            &client,
            "/create",
            json!({
                "name": "Poll Info Test - Ended Happy Path",
                "candidates": ["A", "B"],
                "duration": 2i32,
                "numWinners": 1i32,
                "id": "ended_happy",
            }),
        );
        post(
            &client,
            "/poll/ended_happy/vote",
            json!({
                "candidates": ["A"],
            }),
        );
        std::thread::sleep(std::time::Duration::from_secs(2));

        // poll should be over now
        let response_info_ended = client.get("/poll/ended_happy").dispatch();
        assert_eq!(response_info_ended.status(), Status::Ok);
        let response_info_ended_json = response_info_ended.into_json::<Value>().unwrap();
        assert_eq!(response_info_ended_json["success"], true);
        assert_eq!(
            response_info_ended_json["name"],
            "Poll Info Test - Ongoing Happy Path"
        );
        let candidates_ended: Vec<&str> = response_info_ended_json["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        assert_eq!(candidates_ended, vec!["A", "B"]);
        assert_eq!(
            response_info_ended_json["endingTime"].as_u64().unwrap()
                - response_info_ended_json["creationTime"].as_u64().unwrap(),
            10000
        );
        assert_eq!(response_info_ended_json["numWinners"], 2i32);
        assert_eq!(response_info_ended_json["protection"], Value::Null);
        assert_eq!(response_info_ended_json["numVotes"], 2i32);
        assert_eq!(response_info_ended_json["ended"], true);
        let winners_ended: Vec<&str> = response_info_ended_json["winners"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        assert_eq!(winners_ended, vec!["A"]);
    }

    #[test]
    #[serial]
    fn poll_info_nonexistent() {
        let client = create_client();

        let response_nonexistent = client.get("/poll/nonexistent").dispatch();
        assert_eq!(response_nonexistent.status(), Status::Ok);
        let response_nonexistent_json = response_nonexistent.into_json::<Value>().unwrap();
        assert_eq!(response_nonexistent_json["success"], false);
        assert_eq!(
            response_nonexistent_json["nonexistent"],
            "TODO: Figure out to check for existence"
        );
    }
}
