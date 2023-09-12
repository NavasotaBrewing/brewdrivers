//! A 3 tiered driver library for interacting with Modbus devices on a SCADA-like network.
//!
//! This library is one of a set of repositories in the
//! [Brewery Control System project](https://github.com/NavasotaBrewing)
//! of the [Navasota Brewing Company](https://navasotabrewing.com). It contains low
//! level drivers for devices we use in the brewing process.
//!
//! # Layers
//!
//! This crate operates on 3 layers:
//! 1. [`drivers`](crate::drivers) -- Low level abtractions of Modbus or other serial devices. These are very general and
//! allow communication to nearly any device.
//! 2. [`controllers`](crate::controllers) -- Implementations of drivers for specific controllers. These are the controllers
//! we use to control field devices in the brewery system.
//! 3. [`model`](crate::model) -- A conceptual model of an RTU, containing a list of devices. These devices are serializable and can be
//! sent over the network. They contain connection details and state, so that they can use a `controller` to read or enact change
//! in a field device. You can write a configuration file that models the RTU and its devices, then this crate can read the configuration
//! file and update/enact the devices as necessary.
//!
//! New controllers will be added as needed. See the [`examples/` directory](https://github.com/NavasotaBrewing/brewdrivers/tree/master/examples)
//! to see how to use this library, and see the [organization documentation](https://github.com/NavasotaBrewing/documentation) for more information about the
//! hardware and project as a whole.

#![allow(non_snake_case)]

pub mod controllers;
pub mod defaults;
pub mod drivers;
pub mod model;
pub mod state;
pub mod logging_utils;

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
        let rtu = crate::model::RTU::generate(Some(crate::defaults::test_config_file()))
            .expect("Couldn't read config file into RTU model");
        rtu.devices
            .iter()
            .find(|dev| dev.conn.controller == con_type)
            .unwrap()
            .clone()
    }

    /// Same as test_device_from_type but filters by ID
    #[allow(dead_code)]
    pub fn test_device_from_id(id: &str) -> model::Device {
        let rtu = crate::model::RTU::generate(Some(crate::defaults::test_config_file()))
            .expect("Couldn't read config file into RTU model");
        rtu.devices.iter().find(|dev| dev.id == id).unwrap().clone()
    }
}
