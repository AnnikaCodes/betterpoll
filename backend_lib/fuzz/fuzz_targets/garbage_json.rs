#![no_main]
use libfuzzer_sys::fuzz_target;
use rocket::serde::json::{Value, json};

#[derive(Debug, arbitrary::Arbitrary)]
enum Route {
    Create,
    Vote,
}

#[derive(Debug, arbitrary::Arbitrary)]
struct Data {
    data: String,
    route: Route,
}

fuzz_target!(|data: Data| {
    let Data { data, route } = data;
    let c = rocket::local::blocking::Client::tracked(backend_lib::rocket()).unwrap();
    let endpoint = match route {
        Route::Create => c.post("/create"),
        Route::Vote => {
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
            c.post(format!("/poll/{}/vote", json["id"].as_str().unwrap()))
        },
    };
    let json = endpoint.json(&data).dispatch().into_json::<Value>().unwrap();
    assert_eq!(json["success"], false);
});
