//! A driver is the lowest level. It is a generic implementation of Modbus or a serial
//! instrument, or similar. [`Controllers`](crate::controllers) are implementations of these drivers.
//! If you're looking to interact with a controller, look there.
//!
//! Devices that communicate through Modbus RTU should provide registers and coils to read and write from.
//! You can see the Modbus RTU Spec on [modbus.org](https://modbus.org/). Some devices don't do this,
//! and instead provide a list of commands as a string of bytes. For the former, see [`ModbusInstrument`](crate::drivers::ModbusInstrument).
//! For the later, see [`SerialInstrument`](crate::drivers::SerialInstrument).
//!
//! Note that technically the devices that don't provide register and coil addresses are still using Modbus RTU. I don't care.

pub mod modbus;
pub mod serial;

pub use modbus::ModbusInstrument;
pub use serial::instrument::SerialInstrument;
