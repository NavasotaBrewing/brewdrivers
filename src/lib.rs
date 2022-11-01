//! Low level drivers for the Navasota Brewing Company's brewing control system.
//!
//! This library is one of a set of repositories in the 
//! [Brewery Control System project](https://github.com/NavasotaBrewing)
//! of the [Navasota Brewing Company](https://navasotabrewing.com). It contains low 
//! level drivers for devices we use in the brewing process.
//! 
//! the [`drivers`](crate::drivers) module contains the low level code for implementing Modbus or serial devices.
//! 
//! The [`controllers`](crate::controllers) module contains implementations built on those drivers for a specific hardware
//! controller, like the CN7500 from Omega Instruments for example.
//!
//! New drivers will be added as needed. See the [`examples/` directory](https://github.com/NavasotaBrewing/brewdrivers/tree/master/examples)
//! to see how to use this library, and see the [organization readme](https://github.com/NavasotaBrewing/readme) for more information about the
//! hardware and project as a whole.

#![allow(non_snake_case)]

#[allow(unused)]
const CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/rtu_conf.yaml";
#[allow(unused)]
const TEST_CONFIG_FILE: &'static str = "/etc/NavasotaBrewing/test_conf.yaml";

pub mod drivers;
pub mod controllers;
pub mod model;
pub mod state;

#[cfg(test)]
mod tests {
    use super::*;

    /// This is a special little function that will deserialize the test RTU configuration
    /// and return the device details of a given type of controller.
    /// This is just used in tests
    /// 
    /// If we don't have a physical control connected to our workstation (when running tests with cargo),
    /// then this will panic on the unwrap().
    pub fn test_device_from_type(con_type: controllers::Controller) -> model::Device {
        let rtu = crate::model::RTU::generate(Some(crate::TEST_CONFIG_FILE)).expect("Couldn't read config file into RTU model");
        rtu.devices.iter().find(|dev| dev.conn.controller == con_type ).unwrap().clone()
    }
    
    /// Same as test_device_from_type but filters by ID
    #[allow(dead_code)]
    pub fn test_device_from_id(id: &str) -> model::Device {
        let rtu = crate::model::RTU::generate(Some(crate::TEST_CONFIG_FILE)).expect("Couldn't read config file into RTU model");
        rtu.devices.iter().find(|dev| dev.id == id ).unwrap().clone()
    }

}