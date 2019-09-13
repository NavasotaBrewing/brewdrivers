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
    // Get the model of the RTU this is running on
    let rtu = config.RTUs.iter().filter(|rtu| rtu.id == get_rtu_id()).collect::<Vec<&RTU>>()[0];
    // Update the values or set the values
    // let updated_rtu = update(&rtu, &config.mode);
    // Need a mutable version
    let mut updated_config = Configuration::from(&config.stringify().unwrap()).unwrap();
    // Remove the RTU
    updated_config.RTUs.drain_filter(|rtu| rtu.id == get_rtu_id());
    // Add back the updated RTU
    updated_config.RTUs.push(updated_rtu.clone());
    // Return updated config
    updated_config.stringify().expect("Could not serialize config")
}

pub fn run() {
    rocket::ignite().mount("/", routes![index, receive_config, running]).launch();
}
