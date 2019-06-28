//! Relay management
//!
//! Relay boards contain relays, which can be on or off. Physical devices like valves and pumps
//! can be connected to relays in order to be toggled on and off. These boards usually communicate over a serial protocol, like RS-485.
//! Any simple device on our brew rig that can be toggled, like a pump or a valve, will use a relay.
//!
//! This module contains code for communication with all types of relay boards we use. Anything related to board management, tooling,
//! and communication is in the `relays` module.
//!
//!
pub mod boards;
pub mod bytestring;

pub use boards::{STR1, State};
pub use bytestring::Bytestring;
