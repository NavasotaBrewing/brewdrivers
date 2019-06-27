//! Drivers for relay boards
//!
//! **Note:** examples are in the struct documentation. See [here](struct.Str1xx.html#examples) for STR1XX boards.
//!
//! # About the boards
//!
//! Relay boards contain relays, which can be on or off. Physical devices like valves and pumps
//! can be connected to relays in order to be toggled on and off.
//!
//! We use the STR1XX line of relay boards from `smart_hardware`, based in Bulgaria. You can buy
//! these relay boards on eBay. Two examples of boards we use are STR116 and STR008,
//! having 16 or 8 relays respectively. Software should work with either one, as the only difference is
//! the number of relays available. If you're using an STR008, you can still ask
//! for the status of a relay out of bounds, 12 for example. If the relay doesn't exist, it will silently fail or return `Off`.
//!
//! These relay boards are the most basic controller in our brewing rig. Check out [Adaptiman's brewing blog](https://adaptiman.com/category/brewing/)
//! for more information on our particular setup.
//!
//! **Note:** Relay boards need to be programmed with an address before use.
//! See the [commands manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for details. This software provides
//! a way to change the controller number, [see here](struct.Str1xx.html#method.set_controller_num).
//!
//! # Command Line Usage
//! This crate provides a command line interface (CLI) for interacting with hardware.
//! See CLI usage [here](cli/index.html).
//!
//! # Links:
//!
//! * [STR1XX board description on eBay](https://bit.ly/31PUi8W)
//! * [Commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
//!
pub mod boards;
pub mod bytestring;

pub use boards::{Str1xx, State, BaudRate};
pub use bytestring::Bytestring;
