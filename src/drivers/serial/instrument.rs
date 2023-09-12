//! Drivers for serial instruments
//!
//! See the [`controllers`](crate::controllers) for the controllers that implement this
//!
//! see the [hardware guides](https://github.com/NavasotaBrewing/readme/tree/master/hardware) for more information.

// std uses
use std::io::{Read, Write};
use std::time::Duration;

// ext uses
use serialport::{DataBits, FlowControl, Parity, StopBits, TTYPort};

use crate::drivers::{InstrumentError, Result};

/// A generic serial instrument.
#[derive(Debug)]
pub struct SerialInstrument {
    address: u8,
    port: TTYPort,
    baudrate: usize,
    timeout: Duration,
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
    /// Yields the TTYPort, consuming the `SerialInstrument` struct
    pub fn owned_port(self) -> TTYPort {
        self.port
    }

    #[allow(unused)]
    /// Returns the baudrate
    pub fn baudrate(&self) -> usize {
        self.baudrate
    }

    /// Sets the baudrate field on the struct. Does not set the baudrate on the controller.
    pub fn set_baudrate(&mut self, new_baudrate: usize) {
        self.baudrate = new_baudrate
    }

    /// Tries to connect to an instrument at the given port and address
    pub fn new(address: u8, port_path: &str, baudrate: usize, timeout: Duration) -> Result<Self> {
        match SerialInstrument::open_port(port_path, baudrate, timeout) {
            Ok(port) => {
                return Ok(SerialInstrument {
                    address,
                    port,
                    baudrate,
                    timeout,
                });
            }
            Err(e) => {
                return Err(InstrumentError::serialError(
                    format!("{}", e),
                    Some(address),
                ));
            }
        }
    }

    /// Opens a TTYPort. This is used in [`SerialInstrument::new()`](crate::drivers::SerialInstrument::new)
    fn open_port(
        port_path: &str,
        baudrate: usize,
        timeout: Duration,
    ) -> std::result::Result<TTYPort, serialport::Error> {
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
            Err(e) => {
                return Err(InstrumentError::serialError(
                    format!("Error writing to board: {}", e),
                    Some(self.address()),
                ))
            }
            _ => {}
        };

        let mut output_buf: Vec<u8> = vec![];

        match self.port.read_to_end(&mut output_buf) {
            Ok(_) => {}
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
    use crate::{controllers::Controller, drivers::serial::bytestring::Bytestring};

    use super::*;

    #[test]
    fn test_open_port() {
        let device = crate::tests::test_device_from_type(Controller::STR1);
        let c = device.conn;
        let board =
            SerialInstrument::new(c.controller_addr(), &c.port(), *c.baudrate(), c.timeout());
        assert!(board.is_ok());
    }

    #[test]
    fn test_write_bytes() {
        let device = crate::tests::test_device_from_type(Controller::WaveshareV2);
        let c = device.conn;
        let mut board =
            SerialInstrument::new(c.controller_addr(), &c.port(), *c.baudrate(), c.timeout())
                .unwrap();

        // This is the waveshare cmd for getting all relays status
        let cmd = Bytestring::from(vec![0x07, 0x14, 0xFE, 0x00, 0x01]);
        // Write the command
        let resp = board.write_to_device(cmd.to_bytes());
        // Make sure we get a response
        assert!(resp.is_ok());
        assert!(resp.unwrap().len() > 0);
    }
}

