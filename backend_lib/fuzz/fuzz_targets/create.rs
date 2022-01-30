#![no_main]
use libfuzzer_sys::fuzz_target;
use rocket::serde::json::{Value, json};
use backend_lib::api::CreateAPIRequestData;

fuzz_target!(|data: CreateAPIRequestData| {
    let c = rocket::local::blocking::Client::tracked(backend_lib::rocket()).unwrap();
    let json = c.post("/create").json(&json!(data)).dispatch().into_json::<Value>().unwrap();
    assert_eq!(json["success"], false);
});
