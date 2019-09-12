#![allow(non_snake_case)]
#![feature(proc_macro_hygiene, decl_macro)]
extern crate serialport;
extern crate hex;
extern crate clap;
extern crate retry;
extern crate serde_json;

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;


pub mod RTU;
pub mod cli;
pub mod master;
