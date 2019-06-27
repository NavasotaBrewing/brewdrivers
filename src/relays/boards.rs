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

#[derive(Debug)]
pub enum BaudRate {
    Baud300,
    Baud600,
    Baud1200,
    Baud2400,
    Baud4800,
    Baud9600,
    Baud19200,
    Baud38400,
    Baud57600,
    Baud115200,
}


#[derive(Debug)]
pub struct Str1xx {
    pub address: u8,
    pub port: TTYPort,
}

impl Str1xx {
    pub fn new(address: u8) -> Str1xx {
        Str1xx {
            address,
            port: Str1xx::port()
        }
    }

    // Returns a port object to write to or read from
    fn port() -> TTYPort {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(20);
        settings.baud_rate = 19200;
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
        println!("Controller {} (Dec. {})", Bytestring::to_hex(self.address), self.address);
        for i in 0..16 {
            println!("Relay {}: {:?}", i, self.get_relay(i));
        }
    }

    pub fn set_baudrate(&mut self, new_baudrate: u32) {
        let new_baud_key = match new_baudrate {
            300 => 0,
            600 => 1,
            1200 => 2,
            2400 => 3,
            4800 => 4,
            9600 => 5,
            19200 => 6,
            38400 => 7,
            57600 => 8,
            115200 => 9,
            _ => 5
        };
        let bytestring = Bytestring::from(vec![8, 51, self.address, 170, 85, new_baud_key]);
        self.write(bytestring);
        match self.port.set_baud_rate(new_baudrate) {
            Ok(_) => {},
            Err(e) => println!("Could not set baud rate: {}", e),
        };
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_relay_status() {
        let mut board = Str1xx::new(2);
        board.set_relay(0, State::Off);

        assert_eq!(State::Off, board.get_relay(0));

        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));
    }
}
