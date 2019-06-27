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
use std::str;

use hex;
use serialport::prelude::*;
use crate::helpers::to_hex;

use State::{On, Off};


#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}


#[derive(Debug)]
pub struct Str1xx {
    pub address: u8
}

impl Str1xx {
    pub fn new(address: u8) -> Str1xx {
        Str1xx {
            address
        }
    }

    // Returns a port object to write to or read from
    fn port() -> Box<SerialPort> {
        let port_name = String::from("/dev/ttyAMA0");
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(20);
        settings.baud_rate = 9600;
        settings.data_bits = DataBits::Eight;
        settings.flow_control = FlowControl::None;
        settings.parity = Parity::None;
        settings.stop_bits = StopBits::One;
        settings.timeout = Duration::from_millis(10);

        serialport::open_with_settings(&port_name, &settings).expect("Couldnt open port")
    }

    // Write to the device and return the bytearray it sends back
    pub fn write(&self, bytestring: Bytestring) -> Vec<u8> {
        let mut port = Str1xx::port();

        match port.write(&bytestring.to_bytes()) {
            Ok(_) => {},
            Err(_) => println!("Error writing to serial device!"),
        };

        let mut output_buf: Vec<u8> = vec![];
        match port.read(&mut output_buf) {
            Ok(_) => return output_buf,
            Err(_) => println!("Error reading from serial device!"),
        };

        return vec![];
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
        println!("{:?}", hex::encode(output_buf));
        unimplemented!();
    }

    pub fn set_controller_num(&mut self, new_cn: u8) {
        // MA0, MA1, 0x06, 0x01, CN, newCN, CS, MAE
        let mut bytestring = Bytestring::from(vec![6, 1, self.address, new_cn]);
        self.write(bytestring);
        self.address = new_cn;
    }

    pub fn list_all_relays(&mut self) {
        println!("Controller {} (Dec. {})", to_hex(self.address), self.address);
        for i in 0..16 {
            println!("Relay {}: {:?}", i, self.get_relay(i));
        }
    }
}



// Master start bytes
const MA0: &str = "55";
const MA1: &str = "aa";
// Master end byte
const MAE: &str = "77";

#[derive(Debug)]
pub struct Bytestring {
    pub data: Vec<u8>,
}

impl Bytestring {
    /// Returns a new Bytestring from a Vec<u8> of data bytes
    pub fn from(bytes: Vec<u8>) -> Bytestring {
        Bytestring {
            data: bytes
        }
    }

    /// Converts a u8 to a 2-character hex String
    pub fn to_hex(val: u8) -> String {
        let hex = format!("{:x}", val);
        if hex.len() == 1 {
            return format!("0{}", hex);
        }
        hex
    }

    /// Converts a 2 character hex String to a u8
    pub fn to_u8(hex: &str) -> Option<u8> {
        if hex.len() > 2 {
            return None;
        }

        match hex::decode(hex) {
            Ok(val) => Some(val[0]),
            Err(_) => None,
        }
    }

    pub fn checksum_as_hex(&self) -> String {
        let sum = self.data.iter().map(|&val| val as i32 ).sum::<i32>();
        let hex_string = format!("{:x}", sum);
        if hex_string.len() == 1 {
            format!("0{}", hex_string)
        } else {
            hex_string[hex_string.len() - 2..].to_string()
        }
    }

    /// Returns a string of all bytes as hex
    pub fn full(&self) -> String {
        let data_strings = self.data.iter().map(|&val| Bytestring::to_hex(val) ).collect::<Vec<String>>();
        format!("{}{}{}{}{}", MA0, MA1, data_strings.join(""), self.checksum_as_hex(), MAE)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        hex::decode(&self.full()).unwrap_or(vec![])
    }
}

impl std::fmt::Display for Bytestring {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.full())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_relay_status() {
        let mut board = Str1xx::new(254);
        board.set_relay(0, State::Off);

        assert_eq!(State::Off, board.get_relay(0));

        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));
    }

    #[test]
    fn bytestring_hex_to_u8() {
        assert_eq!(Some(254), Bytestring::to_u8("fe"));
        assert_eq!(Some(0),   Bytestring::to_u8("00"));
        assert_eq!(Some(16),  Bytestring::to_u8("10"));
        assert_eq!(None,      Bytestring::to_u8("0"));
        assert_eq!(None,      Bytestring::to_u8("fefe"));
    }

    #[test]
    fn full_bytestring() {
        assert_eq!("55aafeff01030177", Bytestring::from(vec![254, 255, 1, 3]).full());
        assert_eq!("55aafefe77", Bytestring::from(vec![254]).full());
        assert_eq!("55aa010177", Bytestring::from(vec![1]).full());
        assert_eq!("55aa0077", Bytestring::from(vec![]).full());
    }

    #[test]
    fn checksum_as_hex() {
        let bs = Bytestring::from(vec![5, 5, 10]);
        assert_eq!("14", bs.checksum_as_hex());
    }

}
