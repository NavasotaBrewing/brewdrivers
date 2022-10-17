use std::io;
use thiserror::Error;

use crate::controllers::AnyState;

#[derive(Error, Debug)]
pub enum InstrumentError {
    #[error("Timeout error: Modbus device on port {port}, slave addr {addr} timed out after request to register 0x{register:X}")]
    ModbusTimeoutError {
        port: String,
        addr: u8,
        register: u16,
    },
    #[error("IO Error: {0}")]
    IOError(io::Error),
    #[error("Board 0x{addr:X?}: {msg}")]
    SerialError { msg: String, addr: Option<u8> },
    #[error("State Error: incorrect state type")]
    StateError(AnyState)
}

impl InstrumentError {
    pub fn modbusTimeoutError(port: &str, addr: u8, register: u16) -> Self {
        Self::ModbusTimeoutError { port: port.to_string(), addr, register }
    }

    pub fn serialError(msg: String, addr: Option<u8>) -> Self {
        Self::SerialError { msg, addr }
    }
}

impl From<io::Error> for InstrumentError {
    fn from(e: io::Error) -> Self { Self::IOError(e) }
}

