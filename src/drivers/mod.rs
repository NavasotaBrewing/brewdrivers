/// A driver is the lowest level. It is a generic implementation of Modbus or a serial
/// instrument. These drivers are used by controllers.


pub mod modbus;
pub mod serial;
pub mod instrument_error;

pub use instrument_error::InstrumentError;

pub type Result<T> = std::result::Result<T, InstrumentError>;