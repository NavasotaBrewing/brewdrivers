use std::{io, fmt};

use crate::drivers::modbus::ModbusInstrument;

/// An error type when interacting with Modbus devices.
#[derive(Debug)]
pub enum ModbusError {
    /// Wraps a [`TimeoutError`](crate::modbus::TimeoutError)
    TimeoutError(TimeoutError),
    /// Wraps an [`io::Error`](std::io::Error)
    IOError(io::Error),
    /// Wraps a
    ConnectionError(io::Error),
}

impl fmt::Display for ModbusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ModbusError::TimeoutError(e) => writeln!(f, "{}", e),
            ModbusError::IOError(e) => writeln!(f, "{}", e),
            ModbusError::ConnectionError(e) => writeln!(f, "{}", e),
        }
    }
}

impl From<io::Error> for ModbusError {
    fn from(e: io::Error) -> Self {
        ModbusError::ConnectionError(e)
    }
}

/// Returned when a request times out
#[derive(Debug)]
pub struct TimeoutError {
    port: String,
    addr: u8,
    register: u16,
}

impl TimeoutError {
    /// This creates an error from a given register and Modbus instrument. This is typically
    /// used when a read or write times out. The error will contain the attempted register to
    /// read/write to, and the details of the device for printing.
    pub fn from_device(register: u16, device: &ModbusInstrument) -> TimeoutError {
        TimeoutError {
            port: String::from(&device.port_path),
            addr: device.slave_addr,
            register,
        }
    }
}

impl fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "Timeout error: Modbus device on port {}, slave addr {} timed out after request to register 0x{:X}",
            self.port,
            self.addr,
            self.register
        )
    }
}

impl std::error::Error for TimeoutError {}
