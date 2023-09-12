//! This model is a high level abstraction of a device. It is serializable and meant to be
//! sent through the network between web servers. It contains an implementation to talk with the hardware
//! through the drivers also provided by this crate.
use std::path::PathBuf;
use std::time::Duration;

use log::*;
use serde::{Deserialize, Serialize};

use crate::controllers::*;
use crate::defaults::{default_command_retries, default_retry_delay};
use crate::drivers::InstrumentError;
use crate::model::SCADADevice;
use crate::state::DeviceState;

type Result<T> = std::result::Result<T, InstrumentError>;

/// Holds the connection details for a device
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Connection {
    /// The serial port the device runs on.
    ///
    /// This will probably be `/dev/ttyUSB0`
    pub port: PathBuf,
    pub baudrate: usize,
    pub timeout: u64,
    /// The devices specific address (ie. relay number, etc.)
    ///
    /// If the device has no specific address within the controller, set to 0
    #[serde(default)]
    pub addr: u8,
    /// The address of the controller on the RS485 bus
    pub controller_addr: u8,
    /// The type of controller the device runs on
    pub controller: Controller,
}

impl Connection {
    /// Gets the port as a `&str`
    pub fn port(&self) -> String {
        // TODO: This is bad
        self.port.as_path().to_str().unwrap().to_string()
    }

    /// Gets the device address
    pub fn addr(&self) -> u8 {
        self.addr
    }

    /// Gets the controller address
    pub fn controller_addr(&self) -> u8 {
        self.controller_addr
    }

    /// Gets the controller type
    pub fn controller(&self) -> &Controller {
        &self.controller
    }

    /// Gets the baudrate
    pub fn baudrate(&self) -> &usize {
        &self.baudrate
    }

    pub fn timeout(&self) -> Duration {
        Duration::from_millis(self.timeout)
    }
}

/// A digital representation of a device
///
/// Devices are not controllers. They belong to controllers, and sometimes there is 1 device for 1 controller.
/// And example is that each relay on a relay board is it's own device, so 1 controller -> 8 devices (or similar).
/// Or we could have 1 PID controller that controls 1 Thermometer device.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Device {
    /// The ID of the device, must be unique among all devices on all RTUs
    pub id: String,
    /// A pretty name, for display purposes
    pub name: String,
    /// Amount of times to retry an update/enact if it fails.
    /// This should in the range [0, 6]
    #[serde(default = "default_command_retries")]
    pub command_retries: u8,
    /// Delay (ms) between retries if there's a failure.
    /// Should be less than 2000, and >= the devices timeout
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    /// Connection details for the device
    pub conn: Connection,
    /// The state of the device. Different devices use different types of state.
    ///
    /// Default deserialization is used here so we don't have to specify state
    /// in the config file
    #[serde(default)]
    pub state: DeviceState,
}

impl Device {
    pub async fn update(&mut self) -> Result<()> {
        let total_attempts = self.command_retries + 1;
        for i in 1..=total_attempts {
            info!(
                "Updating device `{}` (Attempt {i} of {})",
                self.id, total_attempts
            );

            let result = match self.conn.controller {
                Controller::STR1 => STR1::update(self).await,
                Controller::CN7500 => CN7500::update(self).await,
                Controller::Waveshare => Waveshare::update(self).await,
                Controller::WaveshareV2 => WaveshareV2::update(self).await,
            };

            match result {
                Ok(_) => return Ok(()),
                Err(e) => {
                    // If we're on the last iteration of the loop
                    // ie. the last retry and we still fail, then return the error
                    if i == total_attempts {
                        return Err(e);
                    }
                    info!("Updating device `{}` failed, but attempts remain. Waiting for retry_delay = {} ms before trying again.", self.id, self.retry_delay);
                    std::thread::sleep(Duration::from_millis(self.retry_delay));
                }
            }
        }

        panic!("Reached some code that shouldn't be reachable. Ran through all iterations of a device update loop without Ok() or Err()");
    }

    pub async fn enact(&mut self) -> Result<()> {
        let total_attempts = self.command_retries + 1;
        for i in 1..=total_attempts {
            info!(
                "Enacting device `{}` (Attempt {i} of {})",
                self.id, total_attempts
            );

            let result = match self.conn.controller {
                Controller::STR1 => STR1::enact(self).await,
                Controller::CN7500 => CN7500::enact(self).await,
                Controller::Waveshare => Waveshare::enact(self).await,
                Controller::WaveshareV2 => WaveshareV2::enact(self).await,
            };

            match result {
                Ok(_) => return Ok(()),
                Err(e) => {
                    // If we're on the last iteration of the loop
                    // ie. the last retry and we still fail, then return the error
                    if i == total_attempts {
                        return Err(e);
                    }
                    info!("Enacting device `{}` failed, but attempts remain. Waiting for retry_delay = {} ms before trying again.", self.id, self.retry_delay);
                    std::thread::sleep(Duration::from_millis(self.retry_delay));
                }
            }
        }

        panic!("Reached some code that shouldn't be reachable. Ran through all iterations of a device enact loop without Ok() or Err()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_connection_port() {
        let conn = Connection {
            port: PathBuf::from("/dev/ttyUSB0"),
            baudrate: 19200,
            timeout: 200,
            controller: Controller::CN7500,
            addr: 0,
            controller_addr: 22,
        };

        assert_eq!("/dev/ttyUSB0", conn.port());
        assert_ne!(r#""/dev/ttyUSB0""#, conn.port());
    }
}
