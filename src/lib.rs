#![allow(non_snake_case)]
#![feature(proc_macro_hygiene, decl_macro, drain_filter)]
extern crate serialport;
extern crate hex;
extern crate retry;
extern crate serde_json;
extern crate reqwest;
extern crate shrust;

extern crate futures;

#[macro_use]
extern crate rocket;
// #[macro_use]
extern crate rocket_contrib;
extern crate rocket_cors;


pub mod master;
pub mod RTU;
pub mod cli;
