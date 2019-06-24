//! # Brewdrivers
//! This crate provides drivers for various hardware used to control [Navasota Brewing Cooperative's](http://navasotabrewing.com/) brewing rig.
//! Currently it has one working driver for an STR1XX relay board. Another driver will be created soon for an OmegaCN7500 PID.
//!
//! Click on one of the modules below to see usage and documentation for each driver.
//!
extern crate serialport;
extern crate hex;

pub mod str1;

use str1::{Str1, State};
