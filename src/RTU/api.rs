#[get("/")]
fn index() -> &'static str {
    "RTU API"
}

// #[post("/configuration")]
// fn receive_config() -> JsonValue {

// }

pub fn run() {
    rocket::ignite().mount("/", routes![index]).launch();
}
