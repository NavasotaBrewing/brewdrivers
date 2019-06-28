//! Drivers for relay boards
//!
//! Supported boards:
//! * [SmartHardware STR1 Line](struct.STR1.html)
//! * More to come
//!
//! Documentation for each board type is on it's structs documentation page.
//! See [here](struct.STR1.html) for STR1 boards, currently the only supported board type.
//!
use std::time::Duration;
use std::io::Write;
use std::path::Path;

use hex;
use serialport::prelude::*;
use serialport::posix::TTYPort;
use std::io::Read;


use crate::relays::State::{On, Off};
use crate::relays::Bytestring;


#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}


/// STR1 Relay Boards
///
/// # Quickstart
/// ```rust
/// // remember to add `extern crate brewdrivers;` to your crate root
/// use brewdrivers::relays::*;
///
/// // Give it the controller number
/// let mut board = STR1::new(2);
///
/// // Returns State::On or State::Off
/// board.get_relay(0);
///
/// board.set_relay(5, State::On);
///
/// // Default board address is 254
/// // probably only need to set it once
/// board.set_controller_num(3);
/// // Set it back
/// board.set_controller_num(2);
/// ```
/// See the [usage](struct.STR1.html#usage) section for more details.
///
/// # Hardware
///
/// Relay boards contain relays, which can be on or off. Physical devices like valves and pumps
/// can be connected to relays in order to be toggled on and off. These boards usually communicate over a serial protocol, like RS-485.
///
/// This struct is for the STR1 line of relay boards from [`SmartHardware`](https://www.smarthardware.eu/index.php), based in Bulgaria.
/// It has been tested on the STR116 and STR108, but theoretically should work on any STR1 board. You can buy these boards on eBay.
/// The two boards we use are STR116 and STR008, having 16 or 8 relays respectively.
///
/// These relay boards are the most basic controller in our brewing rig. Check out [Adaptiman's brewing blog](https://adaptiman.com/category/brewing/)
/// for more information on our particular setup.
///
/// ## Hardware: Setup
///
/// I'll ask adaptiman to write a guide [on his website](https://adaptiman.com/) to outline how to physically connect the STR1 boards to a host.
/// We have it connected to a Raspberry Pi through an RS-232 hat. More details on his site (soon...).
///
/// Relay boards require a bit of setup before before use. This package uses the default settings for the STR1 boards,
/// but you'll probably want to set the address (controller number). You can program the board to the default settings using a jumper,
/// the process for which is outlined in the [hardware guide, page 8](https://www.smarthardware.eu/manual/str1160000h_doc.pdf). It's pretty
/// easy if you have a jumper, I think one is included. Default address is 254 in decimal (that's `fe` in hex, if you care).
/// You can leave it at 254, or set it to something new to keep track of multiple boards. You can set the address from the command line part of this package, or through rust
/// with the [`set_controller_num`](struct.STR1.html#method.set_controller_num) method. This package uses "address" and "controller_num" interchangeably.
///
/// You can change the baudrate of these boards with a pretty simple command. We use the default, 9600, because it's easy.
///
/// # Software
///
/// The STR1 board communicates over the RS-485 protocol. Bytestrings are written through the serial port, and the response can be read.
/// Any serial port can be used, but we use `/dev/ttyAMA0` on our raspberry pi 3 (see
/// [this blog post](https://adaptiman.com/brewing/communication-issues/) on how to set up the RPI3 for serial communication). On windows, the port
/// would probably look something like `/com3` but I don't know because I hardly use windows. The serial port can be opened like a normal serial port,
/// using any common serial package. Rust has `serialport` and python has `pyserial`, but many more exist. Each library handles reading and writing
/// differently. This package uses the `serialport` crate.
///
/// The most important part of communication with an STR1 board is the [`Bytestring`](../bytestring/).
/// The [`Bytestring`](../bytestring/) struct has documentation on the specifics of bytestring, as well as tools for using them.
///
/// # Usage
/// ## Command Line Usage
/// This crate provides a command line interface (CLI) for interacting with hardware.
/// See CLI usage [here](cli/index.html).
///
/// # Rust Usage
/// You can import this module like this
/// ```rust
/// // in your crate root
/// extern crate brewdrivers;
///
/// // in your application code
/// use brewdrivers::relays::*;
/// ```
/// This will bring in the necessary structs to interact with the boards, like [`STR1`](struct.STR1.html) and `Bytestring`.
/// See the docs for those specific structs for more detailed usage.
/// # Links:
///
/// * [Software guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
/// * [Hardware guide](https://www.smarthardware.eu/manual/str1160000h_doc.pdf)
/// * [STR1 board description on eBay](https://bit.ly/31PUi8W)
/// * [SmartHardware homepage](https://www.smarthardware.eu/index.php)
///
#[derive(Debug)]
pub struct STR1 {
    pub address: u8,
    pub port: TTYPort,
}

impl STR1 {
    pub fn new(address: u8) -> STR1 {
        STR1 {
            address,
            port: STR1::port()
        }
    }

    // Returns a port object to write to or read from
    fn port() -> TTYPort {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(20);
        settings.baud_rate = 9600;
        settings.data_bits = DataBits::Eight;
        settings.flow_control = FlowControl::None;
        settings.parity = Parity::None;
        settings.stop_bits = StopBits::One;


        TTYPort::open(&Path::new("/dev/ttyAMA0"), &settings).expect("Couldn't open port")
    }

    // Write to the device and return the bytearray it sends back
    pub fn write(&mut self, bytestring: Bytestring) -> Vec<u8> {
        match self.port.write(&bytestring.to_bytes()) {
            Ok(_) => {},
            Err(_) => println!("Error writing to serial device!"),
        };

        let mut output_buf: Vec<u8> = vec![];
        match self.port.read_to_end(&mut output_buf) {
            Ok(_) => {},
            Err(_) => { /* timeout, expected */ }
        }
        output_buf
    }

    pub fn set_relay(&mut self, relay_num: u8, state: State) {
        let new_state = match state {
            On => 1,
            Off => 0
        };

        // From the software guide
        // MA0, MA1, 0x08, 0x17, CN, start number output, number of outputs, 0/1, CS, MAE
        // MA0, MA1, CS, and MAE are added automatically
        let bytestring = Bytestring::from(vec![8, 23, self.address, relay_num, 1, new_state]);

        self.write(bytestring);
    }

    pub fn get_relay(&mut self, relay_num: u8) -> State {
        let bytestring = Bytestring::from(vec![7, 20, self.address, relay_num, 1]);
        let output_buf = self.write(bytestring);
        let result = hex::encode(output_buf);
        match result.chars().nth(7) {
            Some('1') => return On,
            _ => return Off,
        }
    }

    pub fn set_controller_num(&mut self, new_cn: u8) {
        // MA0, MA1, 0x06, 0x01, CN, newCN, CS, MAE
        let bytestring = Bytestring::from(vec![6, 1, self.address, new_cn]);
        println!("{:?}", self.write(bytestring));
        self.address = new_cn;
    }

    pub fn list_all_relays(&mut self) {
        println!("Controller {}", self.address);
        for i in 0..16 {
            std::thread::sleep(Duration::from_millis(10));
            println!("Relay {}: {:?}", i, self.get_relay(i));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_relay_status() {
        let mut board = STR1::new(2);
        board.set_relay(0, State::Off);

        assert_eq!(State::Off, board.get_relay(0));

        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));
    }
}
