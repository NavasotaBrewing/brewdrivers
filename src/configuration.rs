#![allow(non_snake_case)]
use std::net::SocketAddrV4;

use serde::{Serialize, Deserialize};

use crate::relays::State;

#[derive(Debug, Serialize, Deserialize)]
enum Mode {
    Write,
    Read
}

#[derive(Debug, Serialize, Deserialize)]
enum Driver {
    STR1,
    Omega,
}

#[derive(Debug, Serialize, Deserialize)]
struct Device {
    driver: Driver,
    name: String,
    id: String,
    state: State,
    addr: u8,
    controller_addr: u8
}

#[derive(Debug, Serialize, Deserialize)]
struct RTU {
    name: String,
    location: String,
    id: String,
    ipv4: SocketAddrV4,
    devices: Vec<Device>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    name: String,
    description: String,
    mode: Mode,
    id: String,
    RTUs: Vec<RTU>
}
