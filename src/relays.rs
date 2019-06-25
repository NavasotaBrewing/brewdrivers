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
//! # Links:
//!
//! * [STR1XX board description on eBay](https://bit.ly/31PUi8W)
//! * [Commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
//!
use std::time::Duration;
use std::io::Write;

use hex;
use serialport::prelude::*;
use crate::helpers::zfill;

use State::{On, Off};


/// States of relays
///
/// Relays on these boards are binary, only on or off
#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}

/// Representation of an STR1XX board
///
/// This is the main interface for an STR1XX board.
///
/// # Examples
///
/// ## Toggling some relays:
/// ```rust
/// use brewdrivers::relays::{Str1xx, State};
///
/// let mut board = Str1xx::new(2);
///
/// board.set_relay(1, State::On);  // Turn relay 1 on
/// board.set_relay(1, State::Off); // and back off
///
/// // Get the status of a relay
/// assert_eq!(board.get_relay(1), State::Off);
/// ```
///
/// ## Setting the controller number:
/// ```rust
/// use brewdrivers::relays::Str1xx;
///
/// let mut board = Str1xx::new(2); // Use the current controller number (don't forget it!)
/// board.set_controller_num(4);  // Controller number is changed.
/// ```
#[derive(Debug)]
pub struct Str1xx {
    pub address: String
}

impl Str1xx {
    /// Returns a new Str1xx controller struct.
    ///
    /// Address should be the address previously programmed into the relay board.
    ///
    /// See the [commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for details on programming the number
    pub fn new(address: u8) -> Str1xx {
        Str1xx {
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

    // Write to the device and return the bytearray it sends back
    fn write(&self, bytestring: String) -> Vec<u8> {
        let mut port = Str1xx::port();
        match port.write(&hex::decode(&bytestring).expect("Couldn't decode hex")) {
            Ok(_) => {},
            Err(_) => println!("Could not write bytestring")
        };

        let mut output_buf: Vec<u8> = vec![];
        match port.read_to_end(&mut output_buf) {
            _ => { /* let this fail (timeout) */ }
        };

        output_buf
    }

    // Get a hex checksum on a str
    fn get_checksum(bytestring: &str) -> String {
        let to_check = bytestring.to_string();
        let raw_sum: u8 = hex::decode(&to_check).unwrap().iter().sum();
        String::from(hex::encode([raw_sum]))
    }

    /// Sets a relay On or Off
    ///
    /// ```rust
    /// // Remember to bring in the State enum
    /// use brewdrivers::relays::{Str1xx, State};
    ///
    /// let mut str116 = Str1xx::new(2);
    /// str116.set_relay(4, State::On); // it's on now
    /// ```
    pub fn set_relay(&mut self, relay_num: u8, state: State) {
        let new_state = match state {
            On => "01",
            Off => "00"
        };

        let mut bytestring = format!("55aa0817{}{}01{}checksum77", self.address, zfill(relay_num), new_state);


        let checksum = Str1xx::get_checksum(&bytestring[4..16]);
        bytestring = bytestring.replace("checksum", &checksum);

        self.write(bytestring);
    }

    /// Gets the status of a relay
    ///
    /// ```rust
    /// let mut str116 = Str1xx::new(2);
    ///
    /// str116.get_relay(3); // State::On or State::Off
    /// str116.get_relay(243); // State::Off (relay doesn't exist)
    /// ```
    pub fn get_relay(&mut self, relay_num: u8) -> State {
        if relay_num > 15 {
            return Off;
        }

        // This bytstring is always the same, except for the address number
        let bytestring = format!("55aa0714{}00102d77", self.address);

        // Write to device and get output
        let output_buf = self.write(bytestring);

        let relay_statuses = &hex::encode(output_buf)[6..38];

        let i: usize = (relay_num as usize) * 2;
        if &relay_statuses[i..(i + 2)] == "01" {
            On
        } else {
            Off
        }
    }

    /// Changes the controller number.
    ///
    /// Be careful with this. You need to know the current controller number to access the board, and
    /// to change the controller number, so don't forget it.
    /// ```rust
    /// // Address is 2 at the moment
    /// let mut str116 = Str1xx::new(2);
    ///
    /// str116.set_controller_num(3);
    /// // Address is now 3
    /// ```
    pub fn set_controller_num(&mut self, new_cn: u8) {
        let new_zfilled = zfill(new_cn);

        let mut bytestring = format!("55aa0601{}{}checksum77", self.address, &new_zfilled);

        let checksum = Str1xx::get_checksum(&bytestring[4..12]);
        bytestring = bytestring.replace("checksum", &checksum);

        self.write(bytestring);
        self.address = new_zfilled;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_relays() {
        let mut s = Str1xx::new(2);
        s.set_relay(1, On);
        assert_eq!(s.get_relay(1), On);

        s.set_relay(1, Off);
        assert_eq!(s.get_relay(1), Off);
    }
}
