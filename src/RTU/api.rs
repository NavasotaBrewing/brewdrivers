// RTU API
//
// Recieve config from front end
// send config to all RTUs, get it back
// send it back to front end
use rocket_contrib::json::Json;
use rocket::config::{Config, Environment};
use crate::master::configuration::Configuration;

#[get("/")]
fn index() -> &'static str {
    "RTU API, you shouldn't see this"
}

#[get("/running")]
fn running() -> &'static str {
    r#"{"running":"true"}"#
}

#[post("/configuration", format = "json", data = "<config>")]
fn update_config(config: Json<Configuration>) -> String {
    // Receive a config, consume the Json wrapper, as one does
    let config = config.into_inner();
    // Update the config
    let updated_config = Configuration::update(&config, &config.mode);
    // Return to sender
    updated_config.stringify()
}

pub fn run() {

    let config = Config::build(Environment::Development)
        .address("0.0.0.0")
        .port(3012)
        .finalize().unwrap();

    let app = rocket::custom(config);

    let routes = routes![index, update_config, running];
    app.mount("/", routes).launch();
}
