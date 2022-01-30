#[rocket::launch]
fn launch() -> _ {
    backend_lib::rocket()
}
