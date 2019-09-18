#![allow(non_snake_case)]
#![feature(proc_macro_hygiene, decl_macro, drain_filter)]
extern crate serialport;
extern crate hex;
extern crate clap;
extern crate retry;
extern crate serde_json;
extern crate reqwest;

#[macro_use]
extern crate rocket;
// #[macro_use]
extern crate rocket_contrib;


pub mod RTU;
pub mod cli;
pub mod master;
