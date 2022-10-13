//! A `Bytestring`, which is used in the [`str1`](crate::relays::str1) module.
//!
//! A `Bytestring` is a vector of `u8` bytes, usually represented with hex numbers. This
//! bytestring is used to communicate with the STR1XX relay boards. You can read more about
//! then [here](https://github.com/NavasotaBrewing/readme/blob/master/hardware/STR1XX.md).
//!
//! ## Bytestring format
//! The bytestring format is as follows
//!
//! ```text
//! (MA0) (MA1) (BC) (CC) (CN) (Data)… (Data) (CS) (MAE)
//!
//! MA0 = master start 0
//! MA1 = master start 1
//! BC = bytecount from here to end
//! CC = command
//! CN = controller number
//! data = data bytes
//! CS = checksum
//! MAE = master end
//! ```
//!
//! The "master" bytes (`MA0`, `MA1`, `MAE`) are programmed into the board and *can* be changed, but we keep the defaults of
//! ```text
//! // master start bytes, in hex
//! MA0 = 0x55
//! MA1 = 0xAA
//! // master end byte, in hex
//! MAE = 0x77
//! ```
//!
//! This module keeps these as constants and they cannot be changed at this time.
//!
//! The command (`CC`) depends on the command you want to send. A full list can be seen in the
//! [software manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf).
//!
//! The controller number (`CN`) is the controller number programmed into the board. The default is
//! `0xFE`, but this should be changed. Every contorller should have a unique controller number.
//!
//! The data bytes are a variable number of bytes send to the board. They always start with
//! the controller number of the board, then more data bytes.
//!
//! The checksum is the sum of all the data bytes (excluding `MA0`, `MA1`, and `MAE`). The checksum
//! is always one byte. If the sum is higher than `0xFF`, then only the last byte of the number will
//! be kept. Example: `0x3DFA = 0xFA`.
//!
//! See the [software manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf) for more details and
//! a list of commands.
//!
//! ## Creating a `Bytestring`
//! The `MA0`, `MA1`, `CS`, and `MA0` are added automatically. You just need to provide the `BC`, `CC`, CN`, and
//! data bytes.
//!
//! ```rust
//! use brewdrivers::relays::Bytestring;
//!
//! // This command gets the status of relay 0x00 on controller 0x01
//! let bs = Bytestring::from(vec![0x07, 0x14, 0x01, 0x00, 0x01]);
//! ```

// Master start bytes
const MA0: u8 = 0x55;
const MA1: u8 = 0xAA;
// Master end byte
const MAE: u8 = 0x77;


/// The [`Bytestring`](crate::relays::bytestring) struct, representing a message to the STR1XX board.
#[derive(Debug)]
pub struct Bytestring {
    /// The bytes of a datastring, excluding the `MA0`, `MA1`, `CS`, and `MAE`
    pub data: Vec<u8>,
}

impl Bytestring {
    /// Returns a new Bytestring from a `Vec<u8>` of data bytes (`u8`)
    ///
    /// ## Examples
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// // This command gets the status of relay 0x00 on controller 0x01
    /// let bs = Bytestring::from(vec![0x07, 0x14, 0x01, 0x00, 0x01]);
    /// assert_eq!(bs.data[0], 0x07);
    /// assert_eq!(bs.data[1], 0x14);
    /// ```
    pub fn from(bytes: Vec<u8>) -> Bytestring {
        Bytestring {
            data: bytes
        }
    }


    /// Returns the checksum of the bytestring, as a single byte. The checksum
    /// is the sum of all the bytes, excluding the `MA0`, `MA1` and `MAE` bytes.
    ///
    /// ## Example
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// let bs = Bytestring::from(vec![0x07, 0x14, 0x01, 0x00, 0x01]);
    /// assert_eq!(bs.checksum_as_hex(), 0x1D);
    /// ```
    ///
    /// If the checksum is 2 bytes, then the low byte will be kept
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// let bs = Bytestring::from(vec![0xF3, 0xF3]);
    /// // 0xF3 + 0xF3 = 0x01E6
    /// assert_eq!(bs.checksum_as_hex(), 0xE6);
    /// ```
    pub fn checksum_as_hex(&self) -> u8 {
        let sum = self.data.iter().map(|&val| val as i32 ).sum::<i32>();
        return (sum % 0x100) as u8;
    }

    /// Returns a String of all bytes (including "master" bytes) as hex, padded to 2 spaces.
    ///
    /// ## Example
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// let bs = Bytestring::from(vec![0xF3, 0xF3]);
    /// assert_eq!(bs.full(), "55aaf3f3e677");
    /// ```
    pub fn full(&self) -> String {
        let data_strings = self.data.iter().map(|&val| format!("{:0>2}", format!("{:x}", val)) ).collect::<Vec<String>>();
        format!("{:0>2x}{:0>2x}{}{:0>2x}{:0>2x}", MA0, MA1, data_strings.join(""), self.checksum_as_hex(), MAE)
    }

    /// Consumes the Bytestring, returning the full bytestring
    /// with all bytes, as a `Vec<u8>`.
    ///
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// let bs = Bytestring::from(vec![0xF3, 0xF3]);
    /// assert_eq!(bs.to_bytes(), vec![0x55, 0xAA, 0xF3, 0xF3, 0xE6, 0x77]);
    /// ```
    pub fn to_bytes(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![MA0, MA1];
 
        for byte in &self.data {
            bytes.push(*byte);
        }
        
        bytes.push(self.checksum_as_hex());
        bytes.push(MAE);
        return bytes;
    }
}

impl std::fmt::Display for Bytestring {
    /// ```rust
    /// use brewdrivers::relays::Bytestring;
    ///
    /// let bs = Bytestring::from(vec![0xF3, 0xF3]);
    /// assert_eq!(
    ///     format!("{}", bs),
    ///     "55aaf3f3e677"
    /// );
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.full())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(0x14, bs.checksum_as_hex());
    }
}
