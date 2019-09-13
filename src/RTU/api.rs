// Recieve config from front end
// send config to all RTUs, get it back
// send it back to front end
use rocket_contrib::json::{Json};
use crate::RTU::relays::{State, STR1, Board};
use crate::master::configuration::{Configuration, Mode, Driver, RTU};
use std::fs;


fn get_rtu_id() -> String {
    String::from(fs::read_to_string("/rtu_id").expect("Couldn't read RTU id").trim())
}

// fn update(rtu: &RTU, mode: &Mode) -> RTU {
//     let mut new_rtu = RTU::from(&rtu.stringify().unwrap()).unwrap();
//     new_rtu.devices.iter().map(|device| {
//         match device.driver {
//             Driver::STR1 => {
//                 let mut board = STR1::with_address(device.controller_addr);
//                 match mode {
//                     Mode::Write => board.set_relay(device.addr, &device.state),
//                     Mode::Read => {},
//                 };
//                 device.state = board.get_relay(device.addr);
//             },
//             Driver::Omega => { /* not implemented */ }
//         }
//     });

//     new_rtu
// }

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
    // This is some bad code
    let new_config = Configuration::update(&config.stringify().unwrap(), &config.mode);
    new_config.stringify().expect("Could not serialize config")



    // new_config.update();
    // new_config.stringify().expect("Could not serialize config")
}

pub fn run() {
    rocket::ignite().mount("/", routes![index, receive_config, running]).launch();
}
