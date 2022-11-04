//! An implementation of `ModbusInstrument` for the OMEGA CN7500.
//!
//! The [OMEGA CN7500](https://www.omega.com/en-us/control-monitoring/controllers/pid-controllers/p/CN7200-7500-7600-7800)
//! is a PID that we use to regulate temperatures within the BCS. This module provides a driver implementation for it, based on the
//! [`ModbusInstrument`](crate::drivers::ModbusInstrument) driver.
//!
//! Note: you can set the temperature units (`F` or `C`) of the board with [`CN7500::set_degrees`](crate::controllers::CN7500::set_degrees).
//! All units returned from the board or sent to it (when setting the setpoint value) will use the unit that the board is configured to at the time.
use std::time::Duration;

use async_trait::async_trait;
use log::trace;
use crate::drivers::{
    modbus::ModbusInstrument,
    InstrumentError,
    Result
};
use crate::model::{SCADADevice, Device};
use crate::state::BinaryState;

pub const CN7500_BAUDRATES: [usize; 5] = [2400, 4800, 9600, 19200, 38400];

#[derive(Debug, Clone)]
pub enum Degree {
    Fahrenheit,
    Celsius,
}

/// A CN7500 PID Controller
#[derive(Debug)]
pub struct CN7500(ModbusInstrument);


#[async_trait]
impl SCADADevice for CN7500 {
    /// Updates the given device state using this controller
    async fn update(device: &mut Device) -> Result<()> {
        trace!("Updating CN7500 device `{}`", device.id);

        let mut cn = CN7500::connect(
            device.conn.controller_addr(),
            &device.conn.port(),
            *device.conn.baudrate() as u64,
            device.conn.timeout()
        ).await?;

        device.state.relay_state = Some(cn.is_running().await?.into());
        device.state.pv = Some(cn.get_pv().await?);
        device.state.sv = Some(cn.get_sv().await?);

        Ok(())
    }
    
    /// Writes the given device state to this controller
    async fn enact(device: &mut Device) -> Result<()> {
        trace!("Enacting CN7500 device `{}`", device.id);

        let mut cn = CN7500::connect(
            device.conn.controller_addr(),
            &device.conn.port(),
            *device.conn.baudrate() as u64,
            device.conn.timeout()
        ).await?;
    
        match device.state.relay_state {
            Some(BinaryState::On) => cn.run().await?,
            Some(BinaryState::Off) => cn.stop().await?,
            None => {}
        }

        if let Some(new_sv) = device.state.sv {
            cn.set_sv(new_sv).await?;
        }
    
        Ok(())
    }
}

impl CN7500 {
    /// Connects to a CN7500 board
    pub async fn connect(slave_addr: u8, port_path: &str, baudrate: u64, timeout: Duration) -> Result<Self> {
        trace!("(CN7500 {}) connected", slave_addr);
        let mut cn = CN7500(ModbusInstrument::new(slave_addr, port_path, baudrate, timeout).await?);
        cn.connected().await.map_err(|instr_err|
            InstrumentError::modbusError(
                format!("CN7500 connection failed, likely busy. Error: {}", instr_err),
                Some(slave_addr)
            )
        )?;
        Ok(cn)
    }

    /// Returns `Ok(())` if the instrument is connected, `Err(InstrumentError)` otherwise.
    pub async fn connected(&mut self) -> Result<()> {
        // Try to read a coil, this could really be anything
        // Note: 'running' means the relay is on, not that the CN7500 is on
        self.software_revision().await?;
        Ok(())
    }

    /// Sets the setpoint value (target) of the CN7500. Should be a decimal between 1.0-999.0.
    pub async fn set_sv(&mut self, new_sv: f64) -> Result<()> {
        trace!("(CN7500 {}) Setting sv: {new_sv}", self.0.slave_addr);
        self.0.write_register(0x1001, (new_sv * 10.0) as u16).await
    }

    /// Gets the setpoint value
    pub async fn get_sv(&mut self) -> Result<f64> {
        trace!("(CN7500 {}) getting sv", self.0.slave_addr);
        self.0
            .read_registers(0x1001, 1)
            .await
            .map(|vec| (vec[0] as f64) / 10.0)
    }

    /// Gets the process value
    pub async fn get_pv(&mut self) -> Result<f64> {
        trace!("(CN7500 {}) getting pv", self.0.slave_addr);
        self.0
            .read_registers(0x1000, 1)
            .await
            .map(|vec| (vec[0] as f64) / 10.0)
    }

    /// Returns `Ok(true)` if the relay is activated. The relay may or may not be on if it's activated,
    /// because the PID will control when to feather the relay on or off to control temperature. The relay
    /// will never be on if it's not active (ie. this method returns `Ok(false)`)
    pub async fn is_running(&mut self) -> Result<bool> {
        trace!("(CN7500 {}) polled is running", self.0.slave_addr);
        self.0.read_coils(0x0814, 1).await.map(|vals| vals[0])
    }

    /// Activates the relay
    pub async fn run(&mut self) -> Result<()> {
        trace!("(CN7500 {}) set to run", self.0.slave_addr);
        self.0.write_coil(0x0814, true).await
    }

    /// Deactivates the relay
    pub async fn stop(&mut self) -> Result<()> {
        trace!("(CN7500 {}) set to stop", self.0.slave_addr);
        self.0.write_coil(0x0814, false).await
    }

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
        trace!("(CN7500 {}) polled software revision", self.0.slave_addr);
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
        CN7500::connect(
            device.conn.controller_addr(),
            &device.conn.port(),
            *device.conn.baudrate() as u64,
            device.conn.timeout()
        ).await.unwrap()
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
    async fn test_cn7500_doesnt_respond_when_bad_conn() {
        let cn2 = CN7500::connect(0x18, "/dev/ttyUSB0", 9600, Duration::from_millis(100)).await;
        assert!(cn2.is_err());
    }
}
