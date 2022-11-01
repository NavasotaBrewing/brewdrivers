use std::io;
use thiserror::Error;

use crate::state::StateError;

/// A general purpose error that may be returned from Instrument interactions
#[derive(Error, Debug)]
pub enum InstrumentError {
    #[error("Timeout error: Modbus device on port {port}, slave addr {addr} timed out after request to register 0x{register:X}")]
    ModbusTimeoutError {
        port: String,
        addr: u8,
        register: u16,
    },
    /// [`std::io::Error`](std::io::Error) wrapper
    #[error("IO Error: {0}")]
    IOError(io::Error),
    /// General modbus error
    #[error("addr {addr:?}: {msg}")]
    ModbusError { msg: String, addr: Option<u8> },
    /// General serial board error
    #[error("addr {addr:?}: {msg}")]
    SerialError { msg: String, addr: Option<u8> },
    /// State error, when provided the wrong type of state
    #[error("State Error: {0:?}")]
    StateError(StateError)
}

impl InstrumentError {
    /// Creates a modbus timeout error, just a helper function
    pub fn modbusTimeoutError(port: &str, addr: u8, register: u16) -> Self {
        Self::ModbusTimeoutError { port: port.to_string(), addr, register }
    }

    /// creates a serial error, just a helper function
    pub fn serialError(msg: String, addr: Option<u8>) -> Self {
        Self::SerialError { msg, addr }
    }

    /// creates a modbus error, just a helper function
    pub fn modbusError(msg: String, addr: Option<u8>) -> Self {
        Self::ModbusError { msg, addr }
    }
}

impl From<io::Error> for InstrumentError {
    fn from(e: io::Error) -> Self { Self::IOError(e) }
}

