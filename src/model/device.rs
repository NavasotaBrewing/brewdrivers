//! This model is a high level abstraction of a device. It is serializable and meant to be
//! sent through the network between web servers. It contains an implementation to talk with the hardware
//! through the drivers also provided by this crate.
use serde::{Deserialize, Serialize};

use crate::controllers::*;
use crate::drivers::InstrumentError;
use crate::state::DeviceState;
use crate::model::SCADADevice;

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
    pub async fn update(&mut self) -> Result<()> {
        match self.controller {
            Controller::STR1 => STR1::update(self).await,
            Controller::CN7500 => CN7500::update(self).await,
            Controller::Waveshare => Waveshare::update(self).await
        }
    }
    
    pub async fn enact(&mut self) -> Result<()> {
        match self.controller {
            Controller::STR1 => STR1::enact(self).await,
            Controller::CN7500 => CN7500::enact(self).await,
            Controller::Waveshare => Waveshare::enact(self).await
        }
    }
}

