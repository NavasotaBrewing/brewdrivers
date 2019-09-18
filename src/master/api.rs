// Recieve config from front end
// send config to all RTUs, get it back
// send it back to front end
use std::net::SocketAddrV4;

use rocket::config::{Config, Environment};
use rocket_contrib::json::{Json};
use reqwest;
use reqwest::header::{HeaderValue, CONTENT_TYPE};

use crate::master::configuration::Configuration;

#[get("/")]
fn index() -> &'static str {
    "Master API, you shouldn't see this"
}

#[get("/running")]
fn running() -> &'static str {
    r#"{"running":"true"}"#
}

#[post("/configuration", format = "json", data = "<config>")]
fn propogate_to_RTUs(config: Json<Configuration>) -> String {
    // Client to help us send to RTUs
    let client = reqwest::Client::new();
    // Mutable clone of config
    let mut current_config = config.clone();
    // Gets addresses of all RTUs
    let addrs = current_config.RTUs.iter().map(|rtu| rtu.ipv4 ).collect::<Vec<SocketAddrV4>>();
    for addr in addrs {
        // TODO: refactor this, maybe into send_to_rtu() or something similar
        // send config to http://addr/configuration
        // returned config becomes new config to send out, so they
        // update as they go along.

        // Send the config to the RTU...
        let mut body = client
            .post(&format!("http://{}/configuration", addr))
            .body(current_config.stringify())
            .header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
            .send()
            .expect("Something went wrong, could not post to RTU");

        // ...and update current_config with whatever is returned
        current_config = Configuration::from(&body.text().unwrap())
                                            .expect(&format!("Could not get configuration back from RTU with address {}", addr));
    };
    // Return the fully updated config, probably back to the front end
    current_config.stringify()
}

pub fn run() {
    let config = Config::build(Environment::Development)
        .address("0.0.0.0")
        .port(8000)
        .finalize().unwrap();

    let app = rocket::custom(config);

    let routes = routes![index, propogate_to_RTUs, running];
    app.mount("/", routes).launch();
}
