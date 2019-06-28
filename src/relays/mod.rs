//! Drivers for relay boards
//!
//! Relay boards contain relays, which can be on or off. Physical devices like valves and pumps
//! can be connected to relays in order to be toggled on and off. These boards usually communicate over a serial protocol, like RS-485.
//! Any device on our brew rig that can be toggled, like a pump or a valve, will use a relay.
//!
//! Supported Boards:
//! * [SmartHardware STR116 or STR008](struct.STR1.html)
//! * more to come...
//!
//! # Quickstart
//! If you just want to control an STR116 or and STR008 board, see [here](struct.STR1.html).
//!
//! **Note:** examples are in the struct documentation. See [here](struct.STR1.html#examples) for STR1 boards.
//!
//! # Hardware
//!
//! Relay boards contain relays, which can be on or off. Physical devices like valves and pumps
//! can be connected to relays in order to be toggled on and off. These boards usually communicate over a serial protocol, like RS-485.
//!
//! We use the STR1 line of relay boards from [`smart_hardware`](https://www.smarthardware.eu/index.php), based in Bulgaria. You can buy
//! these relay boards on eBay. Two examples of boards we use are STR116 and STR008,
//! having 16 or 8 relays respectively. Software should work with either one, as the only difference is
//! the number of relays available. If you're using an STR008, you can still ask
//! for the status of a relay out of bounds, 12 for example. If the relay doesn't exist, it will return `Off`.
//!
//! These relay boards are the most basic controller in our brewing rig. Check out [Adaptiman's brewing blog](https://adaptiman.com/category/brewing/)
//! for more information on our particular setup.
//!
//! **Note:** Relay boards require a bit of setup before before use. This package uses the default settings, but you'll need to set
//! the address (controller number). You can program the board to the default settings using a jumper,
//! the process for which is outlined in the [hardware guide, page 8](https://www.smarthardware.eu/manual/str1160000h_doc.pdf).
//! Default address is 254 in decimal, or `fe` in hex. You can leave it at that, or set it to something
//! new to keep track of multiple boards. You can set the address from the command line part of this package, or through rust
//! with the [`set_controller_num`](struct.STR1.html#method.set_controller_num) method.
//! See the [commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for details.
//!
//! # Usage
//! ## Command Line Usage
//! This crate provides a command line interface (CLI) for interacting with hardware.
//! See CLI usage [here](cli/index.html).
//!
//! # Rust Usage
//! You can import this module like this
//! ```rust
//! // in your crate root
//! extern crate brewdrivers;
//!
//! // in your application code
//! use brewdrivers::relays::*;
//! ```
//! This will bring in the necessary structs to interact with the boards, like [`STR1`](struct.STR1.html) and `Bytestring`.
//! See the docs for those specific structs for more detailed usage.
//! # Links:
//!
//! * [Software guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
//! * [Hardware guide](https://www.smarthardware.eu/manual/str1160000h_doc.pdf)
//! * [STR1 board description on eBay](https://bit.ly/31PUi8W)
//! * [SmartHardware homepage](https://www.smarthardware.eu/index.php)
//!
pub mod boards;
pub mod bytestring;

pub use boards::{STR1, State, BaudRate};
pub use bytestring::Bytestring;
