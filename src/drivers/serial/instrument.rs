//! Drivers for serial instruments
//!
//! See the [`controllers`](crate::controllers) for the controllers that implement this
//! 
//! see the [hardware guides](https://github.com/NavasotaBrewing/readme/tree/master/hardware) for more information.

// std uses
use std::time::Duration;
use std::io::{Read, Write};

// ext uses
use serialport::{DataBits, FlowControl, Parity, StopBits, TTYPort};

// use crate::drivers::serial::BoardError;
use crate::drivers::{Result, InstrumentError};


/// The state of a relay. This can be 'On' or 'Off'.
///
/// If the `network` feature is enabled, this enum will be serializable with `serde`.
///
/// This enum is mainly here for compatability with the javascript front end.
/// Javascript is pretty fast and loose with it's types, and this ensures we get an explicit
/// 'On' or 'Off' instead of `true`/`false`, `0`/`1`, `null`, etc.
#[cfg_attr(features = "network", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum State {
    On,
    Off
}

impl State {
    /// Converts a `bool` to a `State`
    ///
    /// ```rust
    /// use brewdrivers::drivers::serial::State;
    ///
    /// assert_eq!(State::from(true),  State::On);
    /// assert_eq!(State::from(false), State::Off);
    /// ```
    pub fn from(state: bool) -> State {
        match state {
            true  => State::On,
            false => State::Off
        }
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::On => write!(f, "on"),
            State::Off => write!(f, "off"),
        }
    }
}


/// A generic relay board.
///
/// This is mostly used as a base for other implementations. It can connect to a board
/// through a TTYPort (`serial` crate), and write a message and get a response.
///
/// Addresses are 0-255 (`u8`), Baudrates are standard baudrates for modbus/rs-485 communication, we use
/// 9600 because it's the default for our board. 
///
/// ```no_run
/// use brewdrivers::drivers::SerialInstrument;
/// 
/// let mut board = Board::new(0x01, "/dev/ttyUSB0", 9600).unwrap();
/// ```
#[derive(Debug)]
pub struct SerialInstrument {
    address: u8,
    port: TTYPort,
    baudrate: usize
}

impl SerialInstrument {
    /// Returns the board address
    pub fn address(&self) -> u8 {
        self.address
    }

    // Updated the stored address
    pub fn set_address(&mut self, new_addr: u8) {
        self.address = new_addr;
    }

    /// Returns the borrowed TTYPort
    pub fn port(&self) -> &TTYPort {
        &self.port
    }

    #[allow(unused)]
    /// Yields the TTYPort, consuming the Board struct
    pub fn owned_port(self) -> TTYPort {
        self.port
    }

    #[allow(unused)]
    /// Returns the baudrate
    pub fn baudrate(&self) -> usize {
        self.baudrate
    }

    /// Tries to connect to a Board at the given port and address
    /// ```rust,no_run
    /// use brewdrivers::drivers::SerialInstrument;
    /// 
    /// let mut si = SerialInstrument::new(0x01, "/dev/ttyUSB0", 9600).unwrap();
    /// ```
    pub fn new(address: u8, port_path: &str, baudrate: usize) -> Result<Self> {
        let port = SerialInstrument::open_port(port_path, baudrate).map_err(|err| InstrumentError::serialError(format!("{}", err), Some(address)))?;
        Ok(SerialInstrument {
            address,
            port,
            baudrate
        })
    }

    /// Opens a TTYPort. This is used in [`SerialInstrument::new()`](crate::drivers::SerialInstrument::new)
    fn open_port(port_path: &str, baudrate: usize) -> std::result::Result<TTYPort, serialport::Error> {
        serialport::new(port_path, baudrate as u32)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(45))
            .open_native()
    }

    /// Writes a vector of bytes to the device
    pub fn write_to_device(&mut self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        match self.port.write(&bytes) {
            Err(e) => return Err(
                InstrumentError::SerialError {
                    msg: format!("Error writing to board: {}", e),
                    addr: Some(self.address())
                }
            ),
            _ => {}
        };

        let mut output_buf: Vec<u8> = vec![];

        match self.port.read_to_end(&mut output_buf) {
            Ok(_) => {},
            Err(_) => {
                // timeout, expected
                // I'm pretty sure that the port never returns the number of bytes
                // to be read, and it just times out every time, even on successful writes.
                // It still reads successfully even after timeouts, so it's fine for now.
            }
        }

        Ok(output_buf)
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    // With a board connected at addr 1 and baud 9600 (Waveshare defaults)
    #[test]
    fn test_open_port() {
        let board = SerialInstrument::new(0x01, "/dev/ttyUSB0", 9600);
        assert!(board.is_ok());
    }

    #[test]
    fn test_write_bytes() {
        let mut board = SerialInstrument::new(0x01, "/dev/ttyUSB0", 9600).unwrap();
        // This is the waveshare cmd for getting all relays status
        let cmd: Vec<u8> = vec![0x01, 0x01, 0x00, 0xFF, 0x00, 0x01, 0xCD, 0xFA];
        // Write the command
        let resp = board.write_to_device(cmd);
        // Make sure we get a response
        assert!(resp.is_ok());
        assert!(resp.unwrap().len() > 0);
    }

}