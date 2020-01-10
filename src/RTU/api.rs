// RTU API
//
// Recieve config from front end
// send config to all RTUs, get it back
// send it back to front end
use rocket_contrib::json::Json;
use rocket::config::{Config, Environment};
use rocket_cors;
use rocket::http::Method;
use rocket::{get, routes};
use rocket_cors::{AllowedHeaders, AllowedOrigins};

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
    println!("Got the following: {:?}", config);
    let updated_config = Configuration::update(&config, &config.mode);
    println!("After updating: {:?}", updated_config);
    // Return to sender
    updated_config.stringify()
}

pub fn run() {

    let config = Config::build(Environment::Development)
        .address("0.0.0.0")
        .port(3012)
        .finalize().unwrap();

    let app = rocket::custom(config);

    let allowed_origins = AllowedOrigins::all();

    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post].into_iter().map(From::from).collect(),
        allowed_headers: AllowedHeaders::some(&["Authorization", "Content-Type", "Accept"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors().unwrap();


    let routes = routes![index, update_config, running];
    app
        .mount("/", routes)
        .attach(cors)
        .launch();
}
