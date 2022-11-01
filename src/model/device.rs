//! This model is a high level abstraction of a device. It is serializable and meant to be
//! sent through the network between web servers. It contains an implementation to talk with the hardware
//! through the drivers also provided by this crate.
use log::trace;
use serde::{Deserialize, Serialize};

use crate::controllers::*;
use crate::drivers::InstrumentError;
use crate::state::{DeviceState, StateError, BinaryState};

type Result<T> = std::result::Result<T, InstrumentError>;

/// A digital represenation of a device
///
/// Devices are not controllers. They operate on controllers, and sometimes there is 1 device for 1 controllers.
/// And example is that each relay on a relay board is it's own device, so 1 controller -> 8 devices (or similar).
/// Or we could have 1 PID controller that controls 1 Thermometer device.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Device {
    /// The ID of the device, must be unique among all devices on all RTUs
    pub id: String,
    /// A pretty name, for display purposes
    pub name: String,
    /// The serial port the device runs on.
    ///
    /// This will probably be `/dev/ttyUSB0`
    pub port: String,
    /// The devices specific address (ie. relay number, etc.)
    ///
    /// If the device has no specific address within the controller, set to 0
    pub addr: u8,
    /// The type of controller the device runs on
    pub controller: Controller,
    /// The address of the controller on the RS485 bus
    pub controller_addr: u8,
    /// The state of the device. Different devices use different types of state.
    pub state: Option<DeviceState>,
}

impl Device {
    /// Polls a device for it's state and updates `self` to match
    pub async fn update(&mut self) -> Result<()> {
        trace!("Updating device `{}`", self.id);
        match self.controller {
            Controller::STR1 => self.handle_str1_update().await?,
            Controller::Waveshare => self.handle_waveshare_update().await?,
            Controller::CN7500 => self.handle_cn7500_update().await?,
        }
        Ok(())
    }

    /// Writes `self`'s state to the controller
    pub async fn enact(&mut self) -> Result<()> {
        trace!("Enacting device `{}`", self.id);

        match self.controller {
            Controller::STR1 => self.handle_str1_enact().await?,
            Controller::Waveshare => self.handle_waveshare_enact().await?,
            Controller::CN7500 => self.handle_cn7500_enact().await?,
        }
        Ok(())
    }

    async fn handle_str1_update(&mut self) -> Result<()> {
        trace!("Updating STR1 device `{}`", self.id);

        let mut board = STR1::connect(self.controller_addr, &self.port)?;

        if self.state.is_none() {
            self.state = Some(DeviceState::default())
        }

        if let Some(state) = self.state.as_mut() {
            state.relay_state = Some(board.get_relay(self.addr)?);
        }

        Ok(())
    }

    async fn handle_waveshare_update(&mut self) -> Result<()> {
        trace!("Updating Waveshare device `{}`", self.id);

        let mut board = Waveshare::connect(self.controller_addr, &self.port)?;

        if self.state.is_none() {
            self.state = Some(DeviceState::default())
        }

        if let Some(state) = self.state.as_mut() {
            state.relay_state = Some(board.get_relay(self.addr)?);
        }
        Ok(())
    }

    async fn handle_cn7500_update(&mut self) -> Result<()> {
        trace!("Updating Waveshare device `{}`", self.id);

        let mut cn = CN7500::connect(self.controller_addr, &self.port).await?;

        if self.state.is_none() {
            self.state = Some(DeviceState::default())
        }

        // this is guaranteed
        if let Some(state) = self.state.as_mut() {
            state.relay_state = Some(cn.is_running().await?.into());
            state.pv = Some(cn.get_pv().await?);
            state.sv = Some(cn.get_sv().await?);
        }

        Ok(())
    }

    async fn handle_str1_enact(&mut self) -> Result<()> {
        trace!("Enacting STR1 device `{}`", self.id);
        let mut board = STR1::connect(self.controller_addr, &self.port)?;

        

        if let Some(new_state) = &self.state {
            let new_rs = new_state.relay_state.ok_or(InstrumentError::StateError(
                StateError::BadValue(new_state.clone())
            ))?;
            board.set_relay(self.addr, new_rs)?;
        } else {
            return Err(
                InstrumentError::StateError(StateError::NullState)
            )
        }

        Ok(())
    }

    async fn handle_waveshare_enact(&mut self) -> Result<()> {
        trace!("Enacting Waveshare device `{}`", self.id);
        let mut board = Waveshare::connect(self.controller_addr, &self.port)?;
        
        if let Some(new_state) = &self.state {
            let new_rs = new_state.relay_state.ok_or(InstrumentError::StateError(
                StateError::BadValue(new_state.clone())
            ))?;
            board.set_relay(self.addr, new_rs)?;
        } else {
            return Err(
                InstrumentError::StateError(StateError::NullState)
            )
        }

        Ok(())
    }

    async fn handle_cn7500_enact(&mut self) -> Result<()> {
        trace!("Enacting STR1 device `{}`", self.id);
        let mut cn = CN7500::connect(self.controller_addr, &self.port).await?;
        

        if self.state.is_none() {
            return Err(
                InstrumentError::StateError(
                    StateError::NullState
                )
            )
        }

        let new_state = self.state.as_ref().unwrap();

        match new_state.relay_state {
            Some(BinaryState::On) => cn.run().await?,
            Some(BinaryState::Off) => cn.stop().await?,
            None => {}
        }

        if let Some(new_sv) = new_state.sv {
            cn.set_sv(new_sv).await?;
        }

        Ok(())
    }
}

