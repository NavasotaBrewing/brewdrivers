//! A generic Modbus RTU instrument
//!
//! A number of devices run on the [Modbus RTU](https://en.wikipedia.org/wiki/Modbus)
//! protocol. This module provides tools for a generic Modbus device.
//!
//! This module uses the `tokio v0.2`, and `tokio v1.0` likely won't work.

// std uses

// external uses
use derivative::Derivative;
use tokio::time::{self, Duration};
use tokio_modbus::{
    client::{rtu, Context, Reader, Writer},
    prelude::Slave,
};

use crate::drivers::modbus::{ModbusError, TimeoutError, Result};

/// A generic async Modbus instrument.
///
/// Note: according to the Modbus spec, "coils" hold boolean values, while registers
/// hold `u16` values. This is reflected in the methods in this struct.
#[derive(Derivative)]
#[derivative(Debug)]
pub struct ModbusInstrument {
    pub port_path: String,
    pub slave_addr: u8,
    pub baudrate: u32,
    pub timeout: u64,
    #[derivative(Debug = "ignore")]
    pub ctx: Context,
}

impl ModbusInstrument {
    /// Creates a new `ModbusInstrument`. Opens a serial port on the given port path.
    ///
    /// This will *not* fail if the device is unresponsive, only if the port file (`/dev/ttyUSB0` or similar) doesn't exist.
    ///
    /// ## Examples
    /// ```rust,no_run
    /// use brewdrivers::modbus::ModbusInstrument;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let instr = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();
    /// }
    /// ```
    pub async fn new(slave_addr: u8, port_path: &str, baudrate: u32) -> Result<ModbusInstrument> {
        let builder = tokio_serial::new(port_path, 19200);
        let port = tokio_serial::SerialStream::open(&builder).unwrap();
        let slave = Slave(slave_addr);

        let ctx = rtu::connect_slave(port, slave).await?;

        Ok(ModbusInstrument {
            port_path: String::from(port_path),
            slave_addr,
            baudrate,
            timeout: 100,
            ctx,
        })
    }

    /// Asyncronously reads a number of registers.
    ///
    /// ```rust,no_run
    /// use brewdrivers::modbus::ModbusInstrument;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut instr = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();
    ///     // This is just an example of interaction with an OMEGA CN7500 PID.
    ///     let rsp = instr.read_registers(0x1000, 1).await;
    ///     assert!(rsp.is_ok());
    ///     assert!(rsp.unwrap().len() == 1);
    /// }
    /// ```
    pub async fn read_registers(&mut self, register: u16, count: u16) -> Result<Vec<u16>> {
        let task = self.ctx.read_holding_registers(register, count);

        // TODO: This code is used a lot, maybe abtract it?
        let timeout = time::timeout(Duration::from_millis(self.timeout), task);

        match timeout.await {
            Ok(res) => return res.map_err(|err| ModbusError::IOError(err)),
            Err(_) => {
                return Err(ModbusError::TimeoutError(TimeoutError::from_device(
                    register, &self,
                )));
            }
        }
    }

    /// Writes to a register with the given `u16`. Returns `Ok(())` on success.
    ///
    /// ## Examples
    /// ```rust,no_run
    /// use brewdrivers::modbus::ModbusInstrument;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut instr = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();
    ///     // This is just an example of interaction with an OMEGA CN7500 PID.
    ///     // This would set the CN7500 setpoint to 140.0
    ///     let rsp = instr.write_register(0x1001, 1400).await;
    ///     if rsp.is_ok() {
    ///         println!("Register written successfully!");
    ///     } else {
    ///         println!("Register couldn't be written to. Error: {}", rsp.unwrap_err());
    ///     }
    /// }
    /// ```
    pub async fn write_register(&mut self, register: u16, value: u16) -> Result<()> {
        let task = self.ctx.write_single_register(register, value);

        let timeout = time::timeout(Duration::from_millis(self.timeout), task);

        match timeout.await {
            Ok(resp) => return resp.map_err(|ioerror| ModbusError::IOError(ioerror)),
            Err(_) => {
                return Err(ModbusError::TimeoutError(TimeoutError::from_device(
                    register, &self,
                )));
            }
        }
    }

    /// The same as [`read_registers()`](crate::modbus::ModbusInstrument::read_registers), but for coils
    pub async fn read_coils(&mut self, coil: u16, count: u16) -> Result<Vec<bool>> {
        let task = self.ctx.read_coils(coil, count);

        let timeout = time::timeout(Duration::from_millis(self.timeout), task);

        match timeout.await {
            Ok(resp) => return resp.map_err(|ioerror| ModbusError::IOError(ioerror)),
            Err(_) => {
                return Err(ModbusError::TimeoutError(TimeoutError::from_device(
                    coil, &self,
                )));
            }
        }
    }

    /// The same as [`write_register()`](crate::modbus::ModbusInstrument::write_register), but for coils
    pub async fn write_coil(&mut self, coil: u16, value: bool) -> Result<()> {
        let task = self.ctx.write_single_coil(coil, value);

        let timeout = time::timeout(Duration::from_millis(self.timeout), task);

        match timeout.await {
            Ok(resp) => return resp.map_err(|ioerror| ModbusError::IOError(ioerror)),
            Err(_) => {
                return Err(ModbusError::TimeoutError(TimeoutError::from_device(
                    coil, &self,
                )));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tokio::test;

    async fn instr() -> ModbusInstrument {
        ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200)
            .await
            .unwrap()
    }

    #[test]
    #[serial]
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
    #[serial]
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
