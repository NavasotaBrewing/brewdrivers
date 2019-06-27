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
// use std::thread::sleep;

use hex;
use serialport::prelude::*;
use crate::helpers::to_hex;

use State::{On, Off};


/// States of relays
///
/// Relays on these boards are binary, only on or off
#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}

#[derive(Debug)]
pub struct Str1xx {
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
    pub address: u8
}

impl Str1xx {
    pub fn new(address: u8) -> Str1xx {
        /// Returns a new Str1xx controller struct.
        ///
        /// Address should be the address previously programmed into the relay board.
        ///
        /// See the [commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for details on programming the number
        ///
        /// # Examples
        /// ```rust
        /// // Remember to use it
        /// use brewdrivers::relays::Str1xx;
        ///
        /// // It needs to be declared as mutable
        /// let mut str116 = Str1xx::new(2);
        ///
        /// str116.get_relay(2);
        /// // ...
        /// ```
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
    pub fn write(&self, bytestring: String) -> Vec<u8> {
        let mut port = Str1xx::port();
        match port.write(&hex::decode(&bytestring).expect("Couldn't decode hex")) {
            Ok(_) => {
                let mut output_buf: Vec<u8> = vec![];
                match port.read_to_end(&mut output_buf) {
                    _ => { /* let this fail (timeout) */ }
                };

                return output_buf;
            },
            Err(_) => println!("Could not write bytestring")
        };

        return vec![];
    }

    // Get a hex checksum on a str
    pub fn get_checksum(bytestring: &str) -> String {
        // Split by 2 chars
        let bs = bytestring.to_owned();
        let subs = bs.as_bytes().chunks(2).map(str::from_utf8).collect::<Result<Vec<&str>, _>>().unwrap();

        // Decode each pair of chars and add the sum
        let mut sum: u32 = 0;
        for piece in subs {
            sum += hex::decode(&piece).unwrap()[0] as u32;
        }

        // Sum back to hex
        let sum_string = format!("{:x}", sum);
        // Return only the last 2 chars
        sum_string[sum_string.len() - 2..].to_string()
    }

    pub fn set_relay(&mut self, relay_num: u8, state: State) {
        /// Sets a relay On or Off
        ///
        /// # Examples
        /// ```rust
        /// // Remember to bring in the State enum too
        /// use brewdrivers::relays::{Str1xx, State};
        ///
        /// let mut str116 = Str1xx::new(2);
        /// str116.set_relay(4, State::On); // it's on now
        /// ```
        let new_state = match state {
            On => 1,
            Off => 0
        };

        // From the software guide
        // MA0, MA1, 0x08, 0x17, CN, start number output, number of outputs, 0/1, CS, MAE
        // MA0, MA1, CS, and MAE are added automatically
        let bytestring = Bytestring::from(vec![8, 23, self.address, relay_num, 1, new_state]);

        self.write(bytestring.full());
    }

    pub fn get_relay(&mut self, relay_num: u8) -> State {
        /// Gets the status of a relay.
        ///
        /// To print the status of all relays, see [`list_all_relays`](struct.Str1xx.html#method.list_all_relays)
        ///
        /// # Examples
        /// ```rust
        /// let mut str116 = Str1xx::new(2);
        ///
        /// str116.get_relay(3);   // State::On or State::Off
        /// str116.get_relay(243); // State::Off (even though relay doesn't exist)
        /// ```

        // This bytstring is always the same, except for the address number
        let bytestring = format!("55aa0714{}0000102d77", to_hex(self.address));

        // Write to device and get output
        let output_buf = self.write(bytestring);

        println!("{:?}", output_buf);
        let relay_statuses = &hex::encode(output_buf)[6..38];

        let i: usize = (relay_num as usize) * 2;
        if &relay_statuses[i..(i + 2)] == "01" {
            On
        } else {
            Off
        }
    }

<<<<<<< HEAD
    /// Changes the controller number.
    ///
    /// Be careful with this. You need to know the current controller number to access the board, and
    /// to change the controller number, so don't forget it. The other option is to reset the board to factory defaults.
    ///
    /// # Examples
    /// ```rust
    /// // Address is 2 at the moment
    /// let mut str116 = Str1xx::new(2);
    ///
    /// str116.set_controller_num(3);
    /// // Address is now 3
    /// ```
=======
>>>>>>> 8a8c5554d7002a3b5b7cb54c19892e5c201fdc62
    pub fn set_controller_num(&mut self, new_cn: u8) {
        /// Changes the controller number.
        ///
        /// Be careful with this. You need to know the current controller number to access the board, and
        /// to change the controller number, so don't forget it.
        ///
        /// # Examples
        /// ```rust
        /// // Address is 2 at the moment
        /// let mut str116 = Str1xx::new(2);
        ///
        /// str116.set_controller_num(3);
        /// // Address is now 3
        /// ```
        let mut bytestring = format!("55aa0601{}{}checksum77", to_hex(self.address), &to_hex(new_cn));

        let checksum = Str1xx::get_checksum(&bytestring[4..12]);
        bytestring = bytestring.replace("checksum", &checksum);

        self.write(bytestring);
        self.address = new_cn;
    }

    pub fn list_all_relays(&mut self) {
        /// Prints the status off all the relays
        ///
        /// # Examples
        /// ```rust
        /// let mut str116 = Str1xx::new(2);
        ///
        /// str116.list_all_relays();
        /// ```
        /// Will print:
        /// ```
        /// Controller 02
        /// Relay 0: Off
        /// Relay 1: On
        /// Relay 2: Off
        /// Relay 3: Off
        /// Relay 4: On
        /// Relay 5: On
        /// Relay 6: On
        /// Relay 7: On
        /// Relay 8: Off
        /// Relay 9: Off
        /// Relay 10: Off
        /// Relay 11: Off
        /// Relay 12: Off
        /// Relay 13: Off
        /// Relay 14: Off
        /// Relay 15: Off
        /// ```
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
struct Bytestring {
    pub data: Vec<u8>,
}

impl Bytestring {
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

        println!("{:?}", hex);

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
}

impl std::fmt::Display for Bytestring {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.full())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn toggle_relays() {
    //     let mut s = Str1xx::new(254);
    //     s.set_relay(1, On);
    //     assert_eq!(s.get_relay(1), On);

    //     sleep(Duration::from_millis(500));

    //     s.set_relay(1, Off);
    //     assert_eq!(s.get_relay(1), Off);
    // }

    // #[test]
    // fn checksum() {
    //     assert_eq!("20", Str1xx::get_checksum("0817fe010101"));
    // }

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
