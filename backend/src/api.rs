//! BetterVote API endpoints

use std::net::IpAddr;

use rocket::serde::json::{json, Json, Value};
use rocket::serde::{Deserialize};

use crate::database::postgres::PostgresConnection;
use crate::error::ErrorKind;
use crate::poll::{Poll, RankedChoiceVote};

/// Returns all the routes that should be made available
pub fn routes() -> Vec<rocket::Route> {
    routes![vote, create, poll_info]
}

fn handle_error(e: ErrorKind) -> Value {
    match e {
        ErrorKind::Internal(e) => {
            eprintln!("An error occured: {:?}", e);
            eprintln!("{:?}", backtrace::Backtrace::new());
            json!({ "error": "Sorry, an internal server error occured. The server's administrators have been notified.", "success": false })
        }
    }
}

#[derive(Deserialize)]
struct VoteAPIRequestData {
    pub choices: Vec<String>,
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
            return json!({ "error": "No IP address could be determined from the request.", "success": false });
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

    if vote.ranked_choices.is_empty() || vote.ranked_choices.len() > poll.candidates.len() {
        return json!({
            "error": format!("You must vote for between 1 and {} candidates", poll.candidates.len()),
            "success": false,
        });
    }

    for choice in &vote.ranked_choices {
        if !poll.candidates.contains(choice) {
            return json!({
                "error": format!("The choice '{}' is not a valid choice.", choice),
                "success": false,
            });
        }
    }

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

#[derive(Deserialize)]
#[cfg_attr(fuzzing, derive(arbitrary::Arbitrary, rocket::serde::Serialize, Debug))]
pub struct CreateAPIRequestData<'a> {
    pub name: String,
    pub candidates: Vec<String>,
    pub duration: i64,
    #[serde(rename = "numWinners")]
    pub num_winners: i64,
    pub id: Option<&'a str>,
    pub protection: Option<&'a str>,
}

#[post("/create", data = "<data>")]
async fn create(mut conn: PostgresConnection, data: Json<CreateAPIRequestData<'_>>) -> Value {
    let Json(request) = data;

    // Validate candidates
    if request.candidates.len() < 2 || request.candidates.len() > 1024 {
        return json!({
            "error": "The number of candidates must be between 2 and 1,024.",
            "success": false,
        });
    }
    for candidate in &request.candidates {
        if candidate.len() > 1024 {
            return json!({
                "error": "A candidate's name must be less than 1,024 characters.",
                "success": false,
            });
        }
        if candidate.trim().is_empty() {
            return json!({
                "error": "A candidate's name must not be empty.",
                "success": false,
            });
        }
    }

    // Validate duration
    if request.duration < 1 {
        return json!({
            "error": "The duration must be a positive, nonzero number.",
            "success": false,
        });
    }
    let duration = std::time::Duration::from_secs(request.duration as u64);

    // Validate numWinners
    if request.num_winners <= 0 {
        return json!({
            "error": "The number of winners must be a positive, nonzero number.",
            "success": false,
        });
    }
    if request.num_winners >= request.candidates.len() as i64 {
        return json!({
            "error": "The number of winners must be less than to the number of candidates.",
            "success": false,
        });
    }
    let num_winners: usize = match request.num_winners.try_into() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("An error occured: {:?}", e);
            eprintln!("{:?}", backtrace::Backtrace::new());
            return json!({ "error": "Sorry, an internal server error occured. The server's administrators have been notified.", "success": false });
        }
    };

    // validate ID
    let id = match request.id {
        Some(id) => {
            if id.is_empty() || id.len() > 32 {
                return json!({
                    "error": "The ID must be between 1 and 32 characters.",
                    "success": false,
                });
            }
            if id
                .chars()
                .any(|c| !c.is_ascii_alphanumeric() && c != '_' && c != '.' && c != '-')
            {
                return json!({
                    "error": "The ID must only contain ASCII alphanumeric characters, '-', '.', and '-'.",
                    "success": false,
                });
            }

            match conn.get_poll_by_id(id.to_string()).await {
                Ok(Some(_poll)) => {
                    return json!({
                        "error": format!("A poll already exists with the URL '{}'.", id),
                        "success": false,
                    })
                }
                Ok(None) => {}
                Err(e) => return handle_error(e),
            };

            Some(id.to_string())
        }
        None => None,
    };

    // Validate protection
    let protection = match request.protection {
        Some("ip") => true,
        Some("none") => false,
        Some(_) => {
            return json!({
                "error": "The protection must be either 'ip' or 'none'.",
                "success": false,
            })
        }
        None => false,
    };

    // Validate name
    if request.name.len() > 1024 || request.name.is_empty() {
        return json!({
            "error": "The name must be between 1 and 1,024 characters.",
            "success": false,
        });
    }

    let poll = match Poll::new(
        id,
        request.name,
        request.candidates,
        duration,
        num_winners,
        protection,
    ) {
        Ok(poll) => poll,
        Err(e) => return handle_error(e),
    };

    let id = poll.id.clone();
    match conn.add_poll(poll).await {
        Ok(_) => json!({ "success": true, "id": id }),
        Err(e) => handle_error(e),
    }
}

#[get("/poll/<pollid>")]
async fn poll_info(mut conn: PostgresConnection, pollid: String) -> Value {
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

    let mut result = json!({
        "success": true,
        "name": poll.title,
        "candidates": poll.candidates,
        "creationTime": poll.creation_time,
        "endingTime": poll.end_time,
        "numWinners": poll.num_winners,
        "protection": if poll.prohibit_double_vote_by_ip { json!("ip") } else { Value::Null },
        "numVotes": poll.votes.len(),
    });

    if let Some(mut winners) = poll.winners {
        result["ended"] = Value::Bool(true);
        // Sort in reverse order - lowest ranks first
        winners.sort_by(|a, b| b.rank.cmp(&a.rank));
        let mut winners_unranked = vec![];
        let mut cur_rank = 0;
        while let Some(winner) = winners.pop() {
            if winner.rank != cur_rank {
                if winners_unranked.len() < poll.num_winners {
                    // take the next rank
                    cur_rank += 1;
                } else {
                    // We have enough winners!
                    break;
                }
            }
            winners_unranked.push(winner.candidate);
        }
        result["winners"] = json!(winners_unranked);
    } else {
        result["ended"] = Value::Bool(false);
    }

    result
}

#[cfg(test)]
mod tests {
    use std::net::ToSocketAddrs;

    use postgres::NoTls;

    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use rocket::serde::json::{json, Value};


    use serial_test::serial;

    fn create_client() -> Client {
        Client::tracked(crate::rocket()).expect("valid rocket instance")
    }

    macro_rules! localhost_ip {
        () => {
            "127.0.0.1:49124".to_socket_addrs().unwrap().next().unwrap()
        };
    }

    /// Clears the database (so that unit tests don't interfere with each other)
    fn clear_db(client: &Client) {
        let db_url = client
            .rocket()
            .figment()
            .find_value("databases.test_db.url")
            .expect("No 'test_db' configured in Rocket.toml");
        let db_url = db_url.as_str().unwrap();

        let mut conn = postgres::Client::connect(db_url, NoTls).expect("bad database connection");

        conn.execute("DELETE FROM votes CASCADE", &[]).unwrap();
        conn.execute("DELETE FROM polls CASCADE", &[]).unwrap();
    }

    fn post(client: &Client, path: &str, data: Value) {
        let mut req = client.post(path);
        req.set_remote(localhost_ip!());
        let req = req.json(&data).dispatch();
        assert_eq!(req.status(), Status::Ok);

        let json = req.into_json::<Value>().unwrap();
        assert_eq!(json["success"], true, "no success: {:?}", json);
    }

    #[test]
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn vote_happy_path() {
        let client = create_client();
        clear_db(&client);

        fn get_num_votes(c: &Client, id: &str) -> i64 {
            c.get(format!("/poll/{}", id))
                .dispatch()
                .into_json::<Value>()
                .unwrap()["numVotes"]
                .as_i64()
                .unwrap()
        }
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
    #[cfg_attr(feature = "no-db-test", ignore)]
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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn vote_nonexistent_poll() {
        let client = create_client();
        clear_db(&client);

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
        clear_db(&client);

        post(
            &client,
            "/create",
            json!({
                "name": "Voting Test - Expired Poll",
                "candidates": ["A", "B", "C", "D"],
                "duration": 5i32,
                "numWinners": 1i32,
                "id": "vote_expired",
            }),
        );
        let mut req = client.post("/poll/vote_expired/vote");
        req.set_remote(localhost_ip!());
        let json_before_expiry = req
            .json(&json!({ "choices": ["A", "B", "C", "D"] }))
            .dispatch()
            .into_json::<Value>()
            .unwrap();
        assert_eq!(json_before_expiry["success"], true);

        std::thread::sleep(std::time::Duration::from_secs(5));

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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn vote_ip_duplicate() {
        let client = create_client();
        clear_db(&client);

        post(
            &client,
            "/create",
            json!({
                "name": "Voting Test - Duplicate IP",
                "candidates": ["A", "B", "C", "D"],
                "duration": 10000i32,
                "numWinners": 1i32,
                "id": "vote_ip_duplicate",
                "protection": "ip",
            }),
        );

        let mut req = client.post("/poll/vote_ip_duplicate/vote");
        req.set_remote(localhost_ip!());
        let json = req
            .json(&json!({ "choices": ["A"] }))
            .dispatch()
            .into_json::<Value>()
            .unwrap();
        assert_eq!(json["success"], true);

        let json = client
            .post("/poll/vote_ip_duplicate/vote")
            .json(&json!({ "choices": ["A", "B", "C", "D"] }))
            .dispatch()
            .into_json::<Value>()
            .unwrap();
        assert_eq!(json["success"], false);
        assert!(!json["error"].as_str().unwrap().is_empty());
    }

    #[test]
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_happy_path() {
        let client = create_client();
        clear_db(&client);

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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_missing_params() {
        let client = create_client();
        clear_db(&client);

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
            let response = client.post("/create").json(&bad_json).dispatch();
            assert_eq!(response.status(), Status::UnprocessableEntity);
        }
    }

    #[test]
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_id_exists() {
        let client = create_client();
        clear_db(&client);

        post(
            &client,
            "/create",
            json!({
                "name": "Test Poll 1",
                "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
                "duration": 100000i32,
                "numWinners": 1i32,
                "id": "create_already_exists",
            }),
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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_not_enough_candidates() {
        let client = create_client();
        clear_db(&client);

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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_negative_duration() {
        let client = create_client();
        clear_db(&client);

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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_bad_num_winners() {
        let client = create_client();
        clear_db(&client);

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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_bad_id() {
        let client = create_client();
        clear_db(&client);

        for bad_id in [
            "",
            " ",
            "a b",
            "a b c",
            "morethan32charactersaaaaaaaaaaaaa",
            "&a&&",
            "a??b",
        ] {
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
            assert_eq!(json["success"], false, "ID `{}` was allowed", bad_id);
            assert!(!json["error"].as_str().unwrap().is_empty());
        }
    }

    #[test]
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn create_bad_protection() {
        let client = create_client();
        clear_db(&client);

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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn poll_info_happy_path() {
        let client = create_client();
        clear_db(&client);

        // ongoing
        post(
            &client,
            "/create",
            json!({
                "name": "Poll Info Test - Ongoing Happy Path",
                "candidates": ["A", "B", "C"],
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
        assert_eq!(candidates_ongoing, vec!["A", "B", "C"]);
        assert_eq!(
            response_info_ongoing_json["endingTime"].as_u64().unwrap()
                - response_info_ongoing_json["creationTime"].as_u64().unwrap(),
            10000
        );
        assert_eq!(response_info_ongoing_json["numWinners"], 1i32);
        assert_eq!(response_info_ongoing_json["numVotes"], 0i32);
        assert_eq!(response_info_ongoing_json["protection"], Value::Null);
        assert_eq!(response_info_ongoing_json["ended"], false);

        // ended
        post(
            &client,
            "/create",
            json!({
                "name": "Poll Info Test - Ended Happy Path",
                "candidates": ["A", "B", "C"],
                "duration": 2i32,
                "numWinners": 1i32,
                "id": "ended_happy",
            }),
        );
        post(
            &client,
            "/poll/ended_happy/vote",
            json!({
                "choices": ["A"],
            }),
        );
        std::thread::sleep(std::time::Duration::from_secs(3));

        // poll should be over now
        let response_info_ended = client.get("/poll/ended_happy").dispatch();
        assert_eq!(response_info_ended.status(), Status::Ok);
        let response_info_ended_json = response_info_ended.into_json::<Value>().unwrap();
        assert_eq!(response_info_ended_json["success"], true);
        assert_eq!(
            response_info_ended_json["name"],
            "Poll Info Test - Ended Happy Path"
        );
        let candidates_ended: Vec<&str> = response_info_ended_json["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        assert_eq!(candidates_ended, vec!["A", "B", "C"]);
        assert_eq!(
            response_info_ended_json["endingTime"].as_u64().unwrap()
                - response_info_ended_json["creationTime"].as_u64().unwrap(),
            2
        );
        assert_eq!(response_info_ended_json["numWinners"], 1i32);
        assert_eq!(response_info_ended_json["protection"], Value::Null);
        assert_eq!(response_info_ended_json["numVotes"], 1i32);
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
    #[cfg_attr(feature = "no-db-test", ignore)]
    #[serial]
    fn poll_info_nonexistent() {
        let client = create_client();
        clear_db(&client);

        let response_nonexistent = client.get("/poll/nonexistent").dispatch();
        assert_eq!(response_nonexistent.status(), Status::Ok);
        let response_nonexistent_json = response_nonexistent.into_json::<Value>().unwrap();
        assert_eq!(response_nonexistent_json["success"], false);
        assert_eq!(response_nonexistent_json["name"], json!(null));
    }
}
