pub mod instrument;
pub mod error;

pub use error::{ModbusError, TimeoutError};
pub use instrument::ModbusInstrument;

/// A result type for Modbus device interaction
pub(crate) type Result<T> = std::result::Result<T, ModbusError>;
