//! An implementation of `ModbusInstrument` for the OMEGA CN7500.
//!
//! The [OMEGA CN7500](https://www.omega.com/en-us/control-monitoring/controllers/pid-controllers/p/CN7200-7500-7600-7800)
//! is a PID that we use to regulate temperatures within the BCS. This module provides a driver implementation for it, based on the
//! [`ModbusInstrument`](crate::drivers::ModbusInstrument) driver.
//!
//! Note: you can set the temperature units (`F` or `C`) of the board with [`CN7500::set_degrees`](crate::controllers::CN7500::set_degrees).
//! All units returned from the board or sent to it (when setting the setpoint value) will use the unit that the board is configured to at the time.
use async_trait::async_trait;
use log::trace;
use serde::{Deserialize, Serialize};

use crate::controllers::device_types::PID;
use crate::drivers::modbus::ModbusInstrument;
use crate::drivers::{InstrumentError, Result};
use crate::controllers::SCADADevice;

use crate::model::Device;
use crate::state::{BinaryState, DeviceState};

#[derive(Debug, Clone)]
pub enum Degree {
    Fahrenheit,
    Celsius,
}


/// The state components of the CN7500
/// 
/// This is what is returned when polling the device, and what should be written
/// when controlling the device.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct CN7500State {
    pub relay_state: BinaryState,
    pub pv: f64,
    pub sv: f64
}

/// A CN7500 PID Controller.
///
/// ```rust,no_run
/// use brewdrivers::controllers::{CN7500, PID};
///
/// #[tokio::main]
/// async fn main() {
///     let mut cn = CN7500::connect(0x16, "/dev/ttyUSB0").await.expect("Couldn't get device");
///
///     match cn.get_pv().await {
///         Ok(pv) => println!("CN7500 PV: {}", pv),
///         Err(e) => eprintln!("Error! {}", e)
///     }
///
///     cn.set_sv(145.6).await.expect("Couldn't set sv");
///     assert_eq!(145.6, cn.get_sv().await.unwrap());
/// }
/// ```
#[derive(Debug)]
pub struct CN7500(ModbusInstrument);


#[async_trait]
impl SCADADevice for CN7500 {
    async fn update(device: Device) -> Result<DeviceState, InstrumentError> {
        let mut dev_state = DeviceState::default();
        let cn = Self::connect(device.controller_addr, &device.port).await?;

        
        Ok(dev_state)
    }
    async fn enact(&mut self) -> Result<(), InstrumentError> {

    }
}

#[async_trait]
impl PID<CN7500> for CN7500 {
    /// Connects to a CN7500 board
    ///
    /// ```rust,no_run
    /// use brewdrivers::controllers::{CN7500, PID};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cn = CN7500::connect(0x16, "/dev/ttyUSB0").await.unwrap();
    /// }
    /// ```
    async fn connect(slave_addr: u8, port_path: &str) -> Result<Self> {
        trace!("(CN7500 {}) connected", slave_addr);
        let mut cn = CN7500(ModbusInstrument::new(slave_addr, port_path, 19200).await?);
        cn.connected().await.map_err(|instr_err|
            InstrumentError::modbusError(
                format!("CN7500 connection failed, likely busy. Error: {}", instr_err),
                Some(slave_addr)
            )
        )?;
        Ok(cn)
    }

    /// Returns `Ok(())` if the instrument is connected, `Err(InstrumentError)` otherwise.
    async fn connected(&mut self) -> Result<()> {
        // Try to read a coil, this could really be anything
        // Note: 'running' means the relay is on, not that the CN7500 is on
        self.software_revision().await?;
        Ok(())
    }

    /// Sets the setpoint value (target) of the CN7500. Should be a decimal between 1.0-999.0.
    async fn set_sv(&mut self, new_sv: f64) -> Result<()> {
        trace!("(CN7500 {}) Setting sv: {new_sv}", self.0.slave_addr);
        self.0.write_register(0x1001, (new_sv * 10.0) as u16).await
    }

    /// Gets the setpoint value
    async fn get_sv(&mut self) -> Result<f64> {
        trace!("(CN7500 {}) getting sv", self.0.slave_addr);
        self.0
            .read_registers(0x1001, 1)
            .await
            .map(|vec| (vec[0] as f64) / 10.0)
    }

    /// Gets the process value
    async fn get_pv(&mut self) -> Result<f64> {
        trace!("(CN7500 {}) getting pv", self.0.slave_addr);
        self.0
            .read_registers(0x1000, 1)
            .await
            .map(|vec| (vec[0] as f64) / 10.0)
    }

    /// Returns `Ok(true)` if the relay is activated. The relay may or may not be on if it's activated,
    /// because the PID will control when to feather the relay on or off to control temperature. The relay
    /// will never be on if it's not active (ie. this method returns `Ok(false)`)
    async fn is_running(&mut self) -> Result<bool> {
        trace!("(CN7500 {}) polled is running", self.0.slave_addr);
        self.0.read_coils(0x0814, 1).await.map(|vals| vals[0])
    }

    /// Activates the relay
    async fn run(&mut self) -> Result<()> {
        trace!("(CN7500 {}) set to run", self.0.slave_addr);
        self.0.write_coil(0x0814, true).await
    }

    /// Deactivates the relay
    async fn stop(&mut self) -> Result<()> {
        trace!("(CN7500 {}) set to stop", self.0.slave_addr);
        self.0.write_coil(0x0814, false).await
    }
}

impl CN7500 {
    /// Sets the degree mode of the board to either Fahrenheit or Celsius
    pub async fn set_degrees(&mut self, degree_mode: Degree) -> Result<()> {
        trace!(
            "(CN7500 {}) setting degree mode to {:?}",
            self.0.slave_addr,
            degree_mode
        );
        match degree_mode {
            Degree::Celsius => self.0.write_coil(0x0811, true).await,
            Degree::Fahrenheit => self.0.write_coil(0x0811, false).await,
        }
    }

    pub async fn software_revision(&mut self) -> Result<Vec<u16>> {
        self.0.read_registers(0x102F, 1).await.map_err(|_|
            InstrumentError::SerialError {
                msg: format!("Software revision couldn't be retrieved, the controller likely isn't connected"),
                addr: Some(self.0.slave_addr)
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::controllers::Controller;

    use super::*;

    use tokio::test;

    async fn instr() -> CN7500 {
        let device = crate::tests::test_device_from_type(Controller::CN7500);
        CN7500::connect(device.controller_addr, &device.port).await.unwrap()
    }

    #[test]
    async fn test_new_cn7500() {
        let cn = instr().await;
        assert_eq!(cn.0.port_path, "/dev/ttyUSB0");
    }

    #[test]
    async fn test_set_sv() {
        let mut cn = instr().await;
        let rsp = cn.set_sv(123.4).await;
        assert!(rsp.is_ok());
    }

    #[test]
    async fn test_get_pv() {
        let mut cn = instr().await;
        assert!(cn.get_pv().await.unwrap() > 0.0);
    }

    #[test]
    async fn test_get_sv() {
        let mut cn = instr().await;
        assert!(cn.set_sv(145.7).await.is_ok());
        assert_eq!(cn.get_sv().await.unwrap(), 145.7);
    }

    #[test]
    async fn test_turn_on_relay() {
        let mut cn = instr().await;
        assert!(cn.run().await.is_ok());
        assert!(cn.is_running().await.unwrap());
        assert!(cn.stop().await.is_ok());
    }

    #[test]
    async fn test_cn7500_responds_when_connected() {
        let cn = CN7500::connect(0x16, "/dev/ttyUSB0").await;
        assert!(cn.is_ok());
        let cn2 = CN7500::connect(0x18, "/dev/ttyUSB0").await;
        assert!(cn2.is_err());
    }
}
