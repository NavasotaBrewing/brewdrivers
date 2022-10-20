//! Low level drivers for the Navasota Brewing Company's brewing control system.
//!
//! This library is one of a set of repositories in the 
//! [Brewery Control System project](https://github.com/NavasotaBrewing)
//! of the [Navasota Brewing Company](https://navasotabrewing.com). It contains low 
//! level drivers for devices we use in the brewing process.
//! 
//! the [`drivers`](crate::drivers) module contains the low level code for implementing Modbus or serial devices.
//! The [`controllers`](crate::controllers) module contains implementations built on those drivers for a specific hardware
//! controller, like the CN7500 from Omega Instruments for example.
//!
//! New drivers will be added as needed. See the [`examples/` directory](https://github.com/NavasotaBrewing/brewdrivers/tree/master/examples)
//! to see how to use this library, and see the [organization readme](https://github.com/NavasotaBrewing/readme) for more information about the
//! hardware and project as a whole.

#![allow(non_snake_case)]
pub mod drivers;
pub mod controllers;
