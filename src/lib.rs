//! Low level drivers for the Navasota Brewing Company's brewing control system.
//!
//! This library is one of a set of repositories in the 
//! [Brewery Control System project](https://github.com/NavasotaBrewing)
//! of the [Navasota Brewing Company](https://navasotabrewing.com). It contains low 
//! level drivers for devices we use in the brewing process.
//! 
//! Each module in this library will contain a different set of drivers. For instance, the [`omega`](crate::omega)
//! module contains drivers for OMEGA Engineering devices, and the [`relays`](crate::relays) module provides drivers
//! for any type of relay board.
//!
//! New drivers will be added as needed. See the [`examples/` directory](https://github.com/NavasotaBrewing/brewdrivers/tree/master/examples)
//! to see how to use this library, and see the [organization readme](https://github.com/NavasotaBrewing/readme) for more information about the
//! hardware and project as a whole.

#![allow(non_snake_case)]
pub mod relays;
pub mod omega;
pub mod modbus;
pub mod controller_pool;


pub mod controllers {
    pub use super::controller_pool::{ControllerPool, Controller};
    pub use super::omega::CN7500;
    pub use super::relays::{STR1, Waveshare};
}
