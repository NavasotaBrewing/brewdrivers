//! A driver is the lowest level. It is a generic implementation of Modbus or a serial
//! instrument, or similar. [`Controllers`](crate::controllers) are implementations of these drivers.
//! If you're looking to interact with a controller, look there.


pub mod modbus;
pub mod serial;
pub mod instrument_error;

pub use instrument_error::InstrumentError;
pub use modbus::ModbusInstrument;
pub use serial::SerialInstrument;

pub type Result<T> = std::result::Result<T, InstrumentError>;