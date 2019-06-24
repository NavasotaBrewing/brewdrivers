//! Driver for an `STR1XX` relay board
//!
//! **See the doc page for the `Str1` struct for examples**
//!
//! # About the board
//!
//! Relay boards contain relays, which can be on or off. Physical devices like valves and pumps
//! can be connected to relays in order to be toggled on and off.
//!
//! **Note:** Relay boards need to be programmed with an address before use.
//! See the [commands manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for details. This software provides
//! a way to change the controller number, see `Str1.set_controller_num`.
//!
//! We use the STR1 line of relay boards from `smart_hardware`, based in Bulgaria. You can buy
//! these relay boards on eBay. Two examples of boards we use are STR116 and STR008,
//! having 16 or 8 relays respectively. Software should work with either one. If you're using an STR008, you can still ask
//! for the status of a relay out of bounds, 12 for example. If the relay doesn't exist, it will silently fail or return `Off`.
//!
//! These relay boards are the most basic controller in our brewing rig. Check out [Adaptiman's brewing blog](https://adaptiman.com/category/brewing/)
//! for more information on our particular setup.
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
/// This is the main interface for an STR1XX board.
///
/// # Examples
///
/// ## Toggling some relays:
/// ```rust
/// use brewdrivers::str1::{Str1, State};
///
/// let mut board = Str1::new(2);
///
/// board.set_relay(1, State::On);  // Turn relay 1 on
/// board.set_relay(1, State::Off); // and back off
///
/// // Get the status of a relay
/// assert_eq!(board.get_relay(1), State::Off);
/// ```
///
/// # Setting the controller number:
/// ```rust
/// use brewdrivers::str1::Str1;
///
/// let mut board = Str1::new(2); // Use the current controller number (don't forget it!)
/// board.set_controller_num(4);  // Controller number is changed.
/// ```
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

    // Write to the device and return the bytearray it sends back
    fn write(&self, bytestring: String) -> Vec<u8> {
        let mut port = Str1::port();
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
    /// use brewdrivers::str1::{Str1, State};
    ///
    /// let mut str116 = Str1::new(2);
    /// str116.set_relay(4, State::On); // it's on now
    /// ```
    pub fn set_relay(&mut self, relay_num: u8, state: State) {
        let new_state = match state {
            On => "01",
            Off => "00"
        };

        let mut bytestring = format!("55aa0817{}{}01{}checksum77", self.address, zfill(relay_num), new_state);


        let checksum = Str1::get_checksum(&bytestring[4..16]);
        bytestring = bytestring.replace("checksum", &checksum);

        self.write(bytestring);
    }

    /// Gets the status of a relay
    ///
    /// ```rust
    /// let mut str116 = Str1::new(2);
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
    /// let mut str116 = Str1::new(2);
    /// str116.set_controller_num(3);
    /// // Address is now 3
    /// ```
    pub fn set_controller_num(&mut self, new_cn: u8) {
        let new_zfilled = zfill(new_cn);

        let mut bytestring = format!("55aa0601{}{}checksum77", self.address, &new_zfilled);

        let checksum = Str1::get_checksum(&bytestring[4..12]);
        bytestring = bytestring.replace("checksum", &checksum);

        self.write(bytestring);
        self.address = new_zfilled;
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
