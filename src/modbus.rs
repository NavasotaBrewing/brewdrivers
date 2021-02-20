//! A generic Modbus RTU instrument
//!
//! A number of devices run on the [Modbus RTU](https://en.wikipedia.org/wiki/Modbus)
//! protocol. This module provides tools for a generic Modbus device.
//!
//! This module uses the `tokio v0.2`, and `tokio v1.0` likely won't work.

// std uses
use std::{
    error::Error,
    fmt,
    io
};

// external uses
use tokio::time::{self, Duration};
use tokio_modbus::{client::{Context, Reader, Writer, rtu}, prelude::Slave};
use tokio_serial::{Serial, SerialPortSettings};

/// A result type for Modbus device interaction
pub(crate) type Result<T> = std::result::Result<T, ModbusError>;

/// An error type when interacting with Modbus devices.
#[derive(Debug)]
pub enum ModbusError {
    /// Wraps a [`TimeoutError`](crate::modbus::TimeoutError)
    TimeoutError(TimeoutError),
    /// Wraps an [`io::Error`](std::io::Error)
    IOError(io::Error),
}

impl fmt::Display for ModbusError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            ModbusError::TimeoutError(e) => writeln!(f, "{}", e),
            ModbusError::IOError(e) => writeln!(f, "{}", e),
        }
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
            register
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

impl Error for TimeoutError {}


/// A generic async Modbus instrument.
///
/// Note: according to the Modbus spec, "coils" hold boolean values, while registers
/// hold `u16` values. This is reflected in the methods in this struct.
pub struct ModbusInstrument {
    pub port_path: String,
    pub slave_addr: u8,
    pub baudrate: u32,
    pub timeout: u64,
    pub ctx: Context
}



impl ModbusInstrument {
    /// Creates a new `ModbusInstrument`. Opens a serial port on the given port path.
    ///
    /// This will *not* fail if the device is unresponsive, only if the port file (`/dev/ttyUSB0` or similar) doesn't exist.
    ///
    /// ## Examples
    /// ```rust,nocompile
    /// use brewdrivers::modbus::ModbusInstrument;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let instr = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await;
    /// }
    /// ```
    pub async fn new(slave_addr: u8, port_path: &str, baudrate: u32) -> ModbusInstrument {
        let mut settings = SerialPortSettings::default();
        settings.baud_rate = 19200;
        let port = Serial::from_path(port_path, &settings).expect(&format!("Couldn't open serial port {}", port_path));
        let slave = Slave(slave_addr);

        let ctx = rtu::connect_slave(port, slave).await.unwrap();


        return ModbusInstrument {
            port_path: String::from(port_path),
            slave_addr,
            baudrate,
            timeout: 100,
            ctx,
        };
    }

    /// Asyncronously reads a number of registers.
    ///
    /// ```rust,nocompile
    /// use brewdrivers::modbus::ModbusInstrument;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut instr = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await;
    ///     // This is just an example of interaction with an OMEGA CN7500 PID.
    ///     let rsp = instr.read_registers(0x1000, 1).await;
    ///     assert!(rsp.is_ok());
    ///     assert!(rsp.unwrap().len() == 1);
    /// }
    /// ```
    pub async fn read_registers(&mut self, register: u16, count: u16) -> Result<Vec<u16>> {
        let task = self.ctx.read_holding_registers(register, count);

        let mut timeout = time::delay_for(Duration::from_millis(self.timeout));
        loop {
            tokio::select! {
                rsp = task => {
                    return rsp.map_err(|ioerror| ModbusError::IOError(ioerror) );
                }
                _ = &mut timeout => {
                    return Err(
                        ModbusError::TimeoutError(
                            TimeoutError::from_device(register, &self)
                        )
                    );
                },
            }
        }
    }

    /// Writes to a register with the given `u16`. Returns `Ok(())` on success.
    ///
    /// ## Examples
    /// ```rust,nocompile
    /// use brewdrivers::modbus::ModbusInstrument;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut instr = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await;
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

        let mut timeout = time::delay_for(Duration::from_millis(self.timeout));
        loop {
            tokio::select! {
                rsp = task => {
                    return rsp.map_err(|ioerror| ModbusError::IOError(ioerror) );
                }
                _ = &mut timeout => {
                    return Err(
                        ModbusError::TimeoutError(
                            TimeoutError::from_device(register, &self)
                        )
                    );
                },
            }
        }
    }


    /// The same as [`read_registers()`](crate::modbus::ModbusInstrument::read_registers), but for coils
    pub async fn read_coils(&mut self, coil: u16, count: u16) -> Result<Vec<bool>> {
        let task = self.ctx.read_coils(coil, count);

        let mut timeout = time::delay_for(Duration::from_millis(self.timeout));
        loop {
            tokio::select! {
                rsp = task => {
                    return rsp.map_err(|ioerror| ModbusError::IOError(ioerror) );
                }
                _ = &mut timeout => {
                    return Err(
                        ModbusError::TimeoutError(
                            TimeoutError::from_device(coil, &self)
                        )
                    );
                },
            }
        }
    }

    /// The same as [`write_register()`](crate::modbus::ModbusInstrument::write_register), but for coils
    pub async fn write_coil(&mut self, coil: u16, value: bool) -> Result<()> {
        let task = self.ctx.write_single_coil(coil, value);

        let mut timeout = time::delay_for(Duration::from_millis(self.timeout));
        loop {
            tokio::select! {
                rsp = task => {
                    return rsp.map_err(|ioerror| ModbusError::IOError(ioerror) );
                }
                _ = &mut timeout => {
                    return Err(
                        ModbusError::TimeoutError(
                            TimeoutError::from_device(coil, &self)
                        )
                    );
                },
            }
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    use serial_test::serial;

    async fn instr() -> ModbusInstrument {
        ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await
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
