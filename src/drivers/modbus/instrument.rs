//! A generic Modbus RTU instrument
//!
//! A number of devices run on the [Modbus RTU](https://en.wikipedia.org/wiki/Modbus)
//! protocol. This module provides tools for a generic Modbus device.
//!
//! This module uses the `tokio v0.2`, and `tokio v1.0` likely won't work.

// std uses

// external uses
use derivative::Derivative;
use log::{error, trace};
use tokio::time::{self, Duration};
use tokio_modbus::{
    client::{rtu, Context, Reader, Writer},
    prelude::Slave,
};

use crate::drivers::{InstrumentError, Result};

/// A generic async Modbus instrument.
///
/// Note: according to the Modbus spec, "coils" hold boolean values, while registers
/// hold `u16` values. This is reflected in the methods in this struct.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct ModbusInstrument {
    pub slave_addr: u8,
    pub port_path: String,
    pub baudrate: u64,
    // TODO: change this to a Duration
    pub timeout: Duration,
    #[derivative(Debug = "ignore")]
    pub ctx: Context,
}

impl ModbusInstrument {
    /// Creates a new `ModbusInstrument`. Opens a serial port on the given port path.
    ///
    /// This will *not* fail if the device is unresponsive, only if the port file (`/dev/ttyUSB0` or similar) doesn't exist.
    pub async fn new(
        slave_addr: u8,
        port_path: &str,
        baudrate: u64,
        timeout: Duration,
    ) -> Result<ModbusInstrument> {
        trace!("Setting up Modbus Instrument with details {{ slave_addr: 0x{:X} (dec {}), port_path: '{}', baudrate: {}, timeout: {:?} }}", slave_addr, slave_addr, port_path, baudrate, timeout);
        
        // Open a serial port with tokio_serial
        let builder = tokio_serial::new(port_path, baudrate as u32);
        let port = match tokio_serial::SerialStream::open(&builder) {
            Ok(port) => port,
            Err(serial_err) => {
                error!("Error when connecting to Modbus Instrument. There is likely no port location at `{}`", port_path);
                error!("Serial Error: {}", serial_err);
                return Err(InstrumentError::serialError(
                    format!("serial error: {}", serial_err),
                    Some(slave_addr),
                ));
            }
        };

        // Pass that serial port to tokio_modbus
        let ctx = rtu::connect_slave(port, Slave(slave_addr)).await?;

        // Make a new modbus instrument with the tokio_modbus content
        Ok(ModbusInstrument {
            port_path: String::from(port_path),
            slave_addr,
            baudrate,
            timeout,
            ctx,
        })
    }

    /// Asyncronously reads a number of registers.
    pub async fn read_registers(&mut self, register: u16, count: u16) -> Result<Vec<u16>> {
        let task = self.ctx.read_holding_registers(register, count);

        let timeout = time::timeout(self.timeout, task);

        match timeout.await {
            Ok(res) => return res.map_err(|err| InstrumentError::IOError(err)),
            Err(_) => {
                return Err(InstrumentError::modbusTimeoutError(
                    &self.port_path,
                    self.slave_addr,
                    register,
                ));
            }
        }
    }

    /// Writes to a register with the given `u16`. Returns `Ok(())` on success.
    pub async fn write_register(&mut self, register: u16, value: u16) -> Result<()> {
        let task = self.ctx.write_single_register(register, value);

        let timeout = time::timeout(self.timeout, task);

        match timeout.await {
            Ok(resp) => return resp.map_err(|ioerror| InstrumentError::IOError(ioerror)),
            Err(_) => {
                return Err(InstrumentError::modbusTimeoutError(
                    &self.port_path,
                    self.slave_addr,
                    register,
                ));
            }
        }
    }

    /// The same as [`read_registers()`](crate::drivers::ModbusInstrument::read_registers), but for coils
    pub async fn read_coils(&mut self, coil: u16, count: u16) -> Result<Vec<bool>> {
        let task = self.ctx.read_coils(coil, count);

        let timeout = time::timeout(self.timeout, task);

        match timeout.await {
            Ok(resp) => return resp.map_err(|ioerror| InstrumentError::IOError(ioerror)),
            Err(_) => {
                return Err(InstrumentError::modbusTimeoutError(
                    &self.port_path,
                    self.slave_addr,
                    coil,
                ));
            }
        }
    }

    /// The same as [`write_register()`](crate::drivers::ModbusInstrument::write_register), but for coils
    pub async fn write_coil(&mut self, coil: u16, value: bool) -> Result<()> {
        let task = self.ctx.write_single_coil(coil, value);

        let timeout = time::timeout(self.timeout, task);

        match timeout.await {
            Ok(resp) => return resp.map_err(|ioerror| InstrumentError::IOError(ioerror)),
            Err(_) => {
                return Err(InstrumentError::modbusTimeoutError(
                    &self.port_path,
                    self.slave_addr,
                    coil,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::test;

    async fn instr() -> ModbusInstrument {
        // We use a high timeout here because performance in tests doesn't matter too much
        ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200, Duration::from_millis(100))
            .await
            .unwrap()
    }

    #[test]

    async fn test_read_write_coil() {
        let mut instr = instr().await;
        let rsp1 = instr.write_coil(0x0814, true).await;
        assert!(rsp1.is_ok());
        let value1 = instr.read_coils(0x0814, 1).await;
        assert!(value1.is_ok());
        assert!(value1.unwrap()[0]);

        let rsp2 = instr.write_coil(0x0814, false).await;
        assert!(rsp2.is_ok());
        let value2 = instr.read_coils(0x0814, 1).await;
        assert!(value2.is_ok());
        assert!(!value2.unwrap()[0]);
    }

    #[test]

    async fn test_read_write_register() {
        let mut instr = instr().await;
        // Set SV in register 0x1001 to 1400
        let rsp = instr.write_register(0x1001, 1400).await;
        assert!(rsp.is_ok());

        // Read SV register, assert we get 1400
        let old_sv = instr.read_registers(0x1001, 1).await;
        assert!(old_sv.is_ok());
        assert!(old_sv.unwrap()[0] == 1400);

        // Set SV in register 0x1001 to 1500
        let rsp2 = instr.write_register(0x1001, 1500).await;
        assert!(rsp2.is_ok());

        // Read SV register again, assert we get 1500
        let new_sv = instr.read_registers(0x1001, 1).await;
        assert!(new_sv.is_ok());
        assert!(new_sv.unwrap()[0] == 1500);
    }
}
