//! Drivers for relay boards.
//!
//! Relay boards we support:
//!  * [`STR1XX`](crate::relays::str1)
//!
//! see the [hardware guides](https://github.com/NavasotaBrewing/readme/tree/master/hardware) for more information.

use std::{error, fmt, time::Duration};
use std::io::{Read, Write};
use serialport::{DataBits, FlowControl, Parity, StopBits, TTYPort};


pub mod waveshare;

/// The state of a relay. This can be 'On' or 'Off'.
///
/// If the `network` feature is enabled, this enum will be serializable with `serde`.
///
/// This enum is mainly here for compatability with `brewkit`, the javascript front end.
/// Javascript is pretty fast and loose with it's types, and this ensures we get an explicit
/// 'On' or 'Off' instead of `true`/`false`, `0`/`1`, `null`, etc.
#[cfg_attr(features = "network", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum State {
    On,
    Off
}

impl State {
    /// Converts a `bool` to a `State`
    ///
    /// ```rust
    /// use brewdrivers::relays::State;
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



/// A generic board error. This is used when communication with any board is unsuccessful.
#[derive(Debug)]
pub struct BoardError(String);

impl fmt::Display for BoardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for BoardError {}


/// A generic relay board.
///
/// This is mostly used as a base for other implementations. It can connect to a board
/// through a TTYPort (`serial` crate), and write a message and get a response.
///
/// Addresses are 0-255 (`u8`), Baudrates are standard baudrates for modbus/rs-485 communication, we use
/// 9600 because it's the default for our board. 
///
/// ```no_run
/// use brewdrivers::relays::Board;
/// 
/// let mut board = Board::new(0x01, "/dev/ttyUSB0", 9600).unwrap();
/// ```
#[derive(Debug)]
pub struct Board {
    address: u8,
    port: TTYPort,
    baudrate: usize
}

impl Board {
    /// Returns the board address
    pub fn address(&self) -> u8 {
        self.address
    }

    /// Returns the borrowed TTYPort
    pub fn port(&self) -> &TTYPort {
        &self.port
    }

    /// Yields the TTYPort, consuming the Board struct
    pub fn owned_port(self) -> TTYPort {
        self.port
    }

    /// Returns the baudrate
    pub fn baudrate(&self) -> usize {
        self.baudrate
    }

    /// Tries to connect to a Board at the given port and address
    /// ```no_run
    /// use brewdrivers::relays::Board;
    /// 
    /// let mut board = Board::new(0x01, "/dev/ttyUSB0", 9600).unwrap();
    /// ```
    pub fn new(address: u8, port_path: &str, baudrate: usize) -> Result<Self, BoardError> {
        let port = Board::open_port(port_path, baudrate).map_err(|err| BoardError(format!("{}", err)) );

        Ok(Board {
            address,
            port: port?,
            baudrate
        })
    }

    /// Opens a TTYPort. This is used in Board::new()
    fn open_port(port_path: &str, baudrate: usize) -> Result<TTYPort, serialport::Error> {
        serialport::new(port_path, baudrate as u32)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(45))
            .open_native()
    }

    pub fn write_to_device(&mut self, bytes: Vec<u8>) -> Result<Vec<u8>, BoardError> {
        match self.port.write(&bytes) {
            Err(e) => return Err(BoardError(format!("Error writing to board: {}", e))),
            _ => {}
        };

        let mut output_buf: Vec<u8> = vec![];

        match self.port.read_to_end(&mut output_buf) {
            Ok(_) => {},
            Err(_) => {
                // timeout, expected
                // I'm pretty sure that the port never returns the nunmber of bytes
                // to be read, and it just times out every time, even on successful writes.
                // It still reads successfully even after timeouts, so it's fine for now.
            }
        }

        Ok(output_buf)
    }
}




// TODO: Make these tests better once you know how to use the Waveshare board
#[cfg(test)]
mod tests {
    use super::*;

    // With a board connected at addr 1 and baud 9600 (Waveshare defaults)
    #[test]
    fn test_open_port() {
        let board = Board::new(0x01, "/dev/ttyUSB0", 9600);
        assert!(board.is_ok());
    }

    #[test]
    fn test_write_bytes() {
        let mut board = Board::new(0x01, "/dev/ttyUSB0", 9600).unwrap();
        // This is the waveshare cmd for getting all relays status
        let cmd: Vec<u8> = vec![0x01, 0x01, 0x00, 0xFF, 0x00, 0x01, 0xCD, 0xFA];
        // Write the command
        let resp = board.write_to_device(cmd);
        // Make sure we get a response
        assert!(resp.is_ok());
        assert!(resp.unwrap().len() > 0);
    }
}