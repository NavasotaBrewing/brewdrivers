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
/// use std::time::Duration;
/// 
/// let mut board = SerialInstrument::new(0x01, "/dev/ttyUSB0", 9600, Duration::from_millis(45)).unwrap();
/// ```
#[derive(Debug)]
pub struct SerialInstrument {
    address: u8,
    port: TTYPort,
    baudrate: usize,
    timeout: Duration
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

    /// Returns the timeout
    pub fn timout(&self) -> &Duration {
        &self.timeout
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

    pub fn set_baudrate(&mut self, new_baudrate: usize) {
        self.baudrate = new_baudrate
    }

    /// Tries to connect to a Board at the given port and address
    pub fn new(address: u8, port_path: &str, baudrate: usize, timeout: Duration) -> Result<Self> {
        let port = SerialInstrument::open_port(port_path, baudrate, timeout).map_err(|err| InstrumentError::serialError(format!("{}", err), Some(address)))?;
        Ok(SerialInstrument {
            address,
            port,
            baudrate,
            timeout,
        })
    }

    /// Opens a TTYPort. This is used in [`SerialInstrument::new()`](crate::drivers::SerialInstrument::new)
    fn open_port(port_path: &str, baudrate: usize, timeout: Duration) -> std::result::Result<TTYPort, serialport::Error> {
        serialport::new(port_path, baudrate as u32)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(timeout)
            .open_native()
    }

    /// Writes a vector of bytes to the device
    pub fn write_to_device(&mut self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        match self.port.write(&bytes) {
            Err(e) => return Err(
                InstrumentError::serialError(format!("Error writing to board: {}", e), Some(self.address()))
            ),
            _ => {}
        };

        let mut output_buf: Vec<u8> = vec![];

        // TODO: I think this is only relevant for the STR1 board because it's stupid.
        // Try to handle the error when communicating with the Waveshare
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
    use crate::drivers::serial::Bytestring;

    use super::*;

    #[test]
    fn test_open_port() {
        // The STR1 board, default address 0xFE (254)
        let board = SerialInstrument::new(0xFE, "/dev/ttyUSB0", 9600, Duration::from_millis(100));
        assert!(board.is_ok());
    }

    #[test]
    fn test_write_bytes() {
        // The STR1 board, default address 0xFE (254)
        let mut board = SerialInstrument::new(0xFE, "/dev/ttyUSB0", 9600, Duration::from_millis(100)).unwrap();

        // This is the waveshare cmd for getting all relays status
        let cmd = Bytestring::from(vec![0x07, 0x14, 0xFE, 0x00, 0x01]);
        // Write the command
        let resp = board.write_to_device(cmd.to_bytes());
        // Make sure we get a response
        assert!(resp.is_ok());
        assert!(resp.unwrap().len() > 0);
    }

}