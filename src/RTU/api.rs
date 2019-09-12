// Recieve config from front end
// send config to all RTUs, get it back
// send it back to front end
use rocket_contrib::json::{Json};
use crate::master::configuration::Configuration;
use std::fs;


fn get_rtu_id() -> String {
    String::from(fs::read_to_string("/rtu_id").expect("Couldn't read RTU id").trim())
}

#[get("/")]
fn index() -> &'static str {
    "RTU API, you shouldn't see this"
}

#[get("/running")]
fn running() -> &'static str {
    r#"{"running":"true"}"#
}

// Receive a config
#[post("/configuration", format = "json", data = "<config>")]
fn receive_config(config: Json<Configuration>) -> String {
    println!("Config {} with id {} received", config.name, config.id);
    // Note: this may be logically flipped, test it
    let rtu = &config.RTUs.iter().find(|&rtu| rtu.id == get_rtu_id());
    if let Some(rtu) = rtu {
        // Found an RTU, enact
    }
    config.stringify().expect("Could not serialize config")
}

pub fn run() {
    rocket::ignite().mount("/", routes![index, receive_config, running]).launch();
}
