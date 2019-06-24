//! Driver for an STR1XX relay board
//!
//! # Examples
//!
//! ```rust
//! use Str1::{Str1, States};
//!
//! // Takes the address
//! let mut board = Str1::new(2);
//! // Turn relay 1 on
//! board.set_relay(1, States::On);
//! // and back off
//! board.set_relay(1, States::Off);
//! ```
//!
//! # About the board
//!
//! Relay boards contains relays, which can be on or off. Physical devices like valves and pumps
//! can be connected to relays in order to be toggled on and off.
//!
//! **Note:** Relay boards need to be programmed with an address before use.
//! See the [commands manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for details.
//!
//! We use the STR1 line of relay boards from smart_hardware, based in Bulgaria. You can buy
//! these relay boards on eBay. Two examples of boards we use are STR116 and STR008,
//! having 16 or 8 relays respectively. Software should work with either one.
//!
//!
//! # Links:
//!
//! * [STR1XX board description on eBay](https://bit.ly/31PUi8W)
//! * [Commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
//!
use std::time::Duration;

use hex;
use serialport::prelude::*;
use State::{On, Off};

use std::io::Write;

// Like zfill in python :)
fn zfill(val: u8) -> String {
    format!("{:02}", val)
}

/// States of relays
///
/// These relays are binary, only on or off
#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}

/// Representation of an STR1 board
///
/// This is the main interface for an STR1XX board. See the str1 docs for examples.
#[derive(Debug)]
pub struct Str1 {
    pub address: String
}

impl Str1 {
    /// Returns a new Str1 controller struct.
    ///
    /// Address should be the address previously programmed into the relay board.
    ///
    /// See the [commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for details on programming the number
    pub fn new(address: u8) -> Str1 {
        Str1 {
            address: zfill(address)
        }
    }

    // Returns a port object to write to or read from
    fn port() -> Box<SerialPort> {
        let port_name = String::from("/dev/ttyAMA0");
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(100);
        settings.baud_rate = 19200;
        settings.data_bits = DataBits::Eight;
        settings.flow_control = FlowControl::None;
        settings.parity = Parity::None;
        settings.stop_bits = StopBits::One;
        settings.timeout = Duration::from_millis(15);

        serialport::open_with_settings(&port_name, &settings).expect("Couldnt open port")
    }

    fn get_checksum(bytestring: &str) -> String {
        let to_check = bytestring.to_string();
        let raw_sum: u8 = hex::decode(&to_check).unwrap().iter().sum();
        String::from(hex::encode([raw_sum]))
    }

    /// Sets a relay On or Off
    ///
    /// `relay_num` should be the address of a relay on the board.
    ///
    /// If the provided relay number is out of range, this will silently fail.
    pub fn set_relay(&mut self, relay_num: u8, state: State) {
        let bytestring = match state {
            On => self.get_write_bytestring(relay_num, 1),
            Off => self.get_write_bytestring(relay_num, 0)
        };
        println!("{:?}", &bytestring);

        match Str1::port().write(&bytestring) {
            Ok(_) => {},
            Err(_) => println!("Could not set relay!"),
        };
    }

    pub fn get_relay(&mut self, relay_num: u8) -> State {
        // This bytstring is always the same
        let bytestring = r#"55aa07140200102d77"#;
        let mut port = Str1::port();

        match port.write(&hex::decode(bytestring).expect("Couldn't decode hex")) {
            Ok(_) => {},
            Err(_) => println!("Could not write relay status bytestring")
        };

        let mut output_buf: Vec<u8> = vec![];
        match port.read_to_end(&mut output_buf) {
            _ => { /* let this fail (timeout) */ }
        };

        let relay_statuses = &hex::encode(output_buf)[6..38];

        let i: usize = (relay_num as usize) * 2;
        if &relay_statuses[i..(i + 2)] == "01" {
            On
        } else {
            Off
        }
    }

    fn get_write_bytestring(&self, relay_number: u8, state: u8) -> Vec<u8> {
        let address = zfill(self.address.parse::<u8>().expect("Couldnt parse address number to u8"));
        let relay_num = zfill(relay_number);
        let new_state = zfill(state);

        // This is from adaptibrew
        //
        // MA0, MA1, 0x08, 0x17, CN, start number output (relaynumber), \
        // number of outputs (usually 0x01), 00/01 (off/on), CS (calculated), MAE
        // need to do a checksum on 0x08, 0x17, CN, relaynumber, 0x01, 0x01
        let mut bytestring = format!(r#"55aa0817{}{}01{}checksum77"#, address, relay_num, new_state);

        let checksum = Str1::get_checksum(&bytestring[4..16]);
        bytestring = bytestring.replace("checksum", &checksum);

        hex::decode(&bytestring).expect("Couldn't decode bytestring!")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zfill() {
        assert_eq!("02", zfill(2));
        assert_eq!("05", zfill(5));
        assert_eq!("14", zfill(14));
        assert_eq!("145", zfill(145));

        let s = Str1::new(2);
        assert_eq!("02", s.address);
        assert_ne!("2", s.address);
    }

    #[test]
    fn get_relay_status() {
        let mut s = Str1::new(2);
        s.set_relay(1, On);
        assert_eq!(s.get_relay(1), On);

        s.set_relay(1, Off);
        assert_eq!(s.get_relay(1), Off);
    }
}
