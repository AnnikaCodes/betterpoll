#![no_main]
use libfuzzer_sys::fuzz_target;
use rocket::serde::json::{Value, json};
use std::net::ToSocketAddrs;

fuzz_target!(|data: Vec<String>| {
    let c = rocket::local::blocking::Client::tracked(backend_lib::rocket()).unwrap();
    let response = c
        .post("/create")
        .json(&json!({
            "name": "Test Poll 1",
            "candidates": ["Candidate 1", "Candidate 2", "Candidate 3"],
            "duration": 100000i32,
            "numWinners": 1i32,
        }))
        .dispatch();
    let json = response.into_json::<Value>().unwrap();

    let mut req = c.post(format!("/poll/{}/vote", json["id"].as_str().unwrap())).json(&json!({"choices": data}));
    req.set_remote("127.0.0.1:49124".to_socket_addrs().unwrap().next().unwrap());

    let json = req.dispatch().into_json::<Value>().unwrap();
    assert_eq!(json["success"], false);
});
