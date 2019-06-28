//! Bytestring tools
//!
//! This module handles Bytestrings. Bytestrings are the messages sent to the STR1 relay boards.
//!
//! You probably don't want to use this code directly. Instead, use the [`STR1`](struct.STR1.html) struct to interact with an STR1 board.
//!
//! # Data format
//! A Bytestring is a bytearray (vector of u8 integers) that is written to the board, after which a response can be read.
//! The [software commands guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) explains every command
//! the STR1 board accepts. Bytestrings consist of a string of bytes in hexadecimal. Each byte is 2 characters long. Commands start with the
//! master start bytes (`MA0` and `MA1`), and end with a checksum and master end byte (`CS` and `MAE`). All bytes in between these 4 bytes are data bytes.
//! Data bytes determine what command the board should run, and the data it should use.
//!
//! In short, every Bytestring has the following format
//! ```text
//! MA0 MA1 Data bytes... CS MAE
//!
//! // Example
//! // 55aa1e1405053c77
//! // | 55  | aa  | 1e  14 05 05 |    3c    | 77  |
//! // | MA0 | MA1 |  Data bytes  | checksum | MAE |
//! ```
//!
//! They are formatted without spaces or any kind of separator. `55aa1e1405053c77` is a typical bytestring.
//!
//!
//! ## Master start, end bytes
//! The master start bytes and the master end bytes can be changed, but generally stay the same (unless you change them of course).
//!
//! We use the defaults for the STR1 board:
//! ```text
//! MA0 = 55
//! MA1 = aa
//! MAE = 77
//! ```
//!
//! ## Checksum
//! The checksum is determined by the previous bytes. It is used to check the validity of the command, ensuring no data was lost or corrupted.
//! The checksum is found by converting each of the data bytes (separately, not including `MA0` and `MA1`) to decimal
//! and adding them together. The checksum is this sum, as hex. Example:
//! ```text
//! // This `Bytestring` struct will handle the dirty work of creating a bytestring.
//! // Just give it the databytes as a vector of u8, and it will add the master bytes and checksum
//! //
//! // In base 10, will be converted to hex for you
//! let data_bytes = vec![30, 20, 5, 5];
//!
//! // This creates a bytestring from the data bytes
//! let bytestring = Bytestring::from(data_bytes);
//! // | MA0 | MA1 |  Data bytes  | checksum | MAE |
//! // | 55  | aa  | 1e  14 05 05 |    3c    | 77  |
//! //    Base 10 -> 30  20  5  5
//!
//! // Full bytestring is in hex
//! assert_eq!("55aa1e1405053c77", bytestring.full());
//!
//! // The actual checksum is 60 in decimal (30 + 20 + 5 + 5 = 60). 60 in hex is 3c.
//! assert_eq!("3c", bytestring.checksum_as_hex());
//! ```
//!
//!
//!
//! ## Data bytes
//! **Note**: The main functions you'd want to use (`get_relay`, `set_relay`) are handled by the [`STR1`](struct.STR1.html) struct. It'll do this
//! stuff for you. This is for specific commands you may want to run, like changing the baudrate.
//!
//! Data bytes change depending on the command. This is where you'll use the [software guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf).
//!
//! As an example, let's look at the software guide for setting a relay. It's `0x17` on page 13. Here's a screenshot.
//!
//! ![Set relay](https://i.imgur.com/YRgBPwY.png)
//!
//! It says the data bytes to send are
//! ```text
//! MA0, MA1, 0x08, 0x17, CN, start number output, number of outputs, 0/1 (state), CS, MAE
//! ```
//!
//! `MA0`, `MA1`, `CS` and `MAE` are added for us by the `Bytestring` struct, so we can ignore those. Let's create a new bytestring from
//! the data bytes. Remember, those bytes are in hex, we need them in decimal. `08` in hex is `8` in decimal, coincedentally, but `17` in
//! hex is `23` in decimal.
//! ```rust
//! use brewdrivers::relays::*;
//!
//! // Could be any u8
//! let controller_num = 2;
//! // Could be any u8
//! let relay_num = 4;
//! // Turn it on
//! let new_state = 1;
//!
//! let bytestring = Bytestring::from(vec![8, 23, controller_num, relay_num, 1, new_state]);
//!
//! // Full bytestring with `bytestring.ful()`
//! println!("{}", bytestring.full())
//! ```
//!
//! There's the bytestring for that command!
//!
//! # Using Bytestrings
//! After constructing a bytestring with [`Bytesring::from`](struct.Bytestring.html#method.from), you can write it to the board with
//! `STR1::write` method, which accepts a `Bytestring` type.
//!
//! # Useful links
//! * [Software guide](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
//! * [Online hex to decimal converter](https://www.rapidtables.com/convert/number/hex-to-decimal.html)
//!
use hex;

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
