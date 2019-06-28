//! # Brewdrivers
//! This crate provides drivers for various hardware used to control [Navasota Brewing Cooperative's](http://navasotabrewing.com/) brewing rig.
//!
//! Click on each of the modules below for drivers for that class of hardware.
//!
extern crate serialport;
extern crate hex;
extern crate clap;

pub mod relays;
pub mod cli;
