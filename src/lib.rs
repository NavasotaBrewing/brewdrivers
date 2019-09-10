#![allow(non_snake_case)]
extern crate serialport;
extern crate hex;
extern crate clap;
extern crate retry;
extern crate ws;
extern crate serde_json;

pub mod RTU;
pub mod cli;
pub mod master;
