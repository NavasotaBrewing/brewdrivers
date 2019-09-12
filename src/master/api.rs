// Recieve config from front end
// send config to all RTUs, get it back
// send it back to front end
use rocket_contrib::json::{Json};
use crate::master::configuration::Configuration;

#[get("/")]
fn index() -> &'static str {
    "Master API, you shouldn't see this"
}

#[post("/configuration", format = "json", data = "<config>")]
fn receive_config(config: Json<Configuration>) -> String {
    // Send to all RTUs
    config.stringify().expect("Could not serialize config")
}

pub fn run() {
    rocket::ignite().mount("/", routes![index, receive_config]).launch();
}
