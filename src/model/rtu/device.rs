//! This model is a high level abstraction of a device. It is serializable and meant to be
//! sent through the network between web servers. It contains an implementation to talk with the hardware
//! through the drivers also provided by this crate.
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::controllers::*;
use crate::defaults::{default_command_retries, default_retry_delay};
use crate::logging_utils::device_info;
use crate::model::Connection;
use crate::model::SCADADevice;
use crate::state::DeviceState;
use crate::Result;

// use super::conditions::Condition;

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
    /// This should in the range [0, 5]
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
    /// Reads the state from the actual device and updates this structs internal state to match.
    ///
    /// If reading the state fails, this will retry a few time, as many as is configured in the RTU
    /// configuration.
    pub async fn update(&mut self) -> Result<()> {
        // We'll try to update a few times, in case there's a collision or the hardware fucks up
        let total_attempts = self.command_retries + 1;

        for i in 1..=total_attempts {
            device_info!(
                &self,
                &format!("updating (attempt {i} of {})", total_attempts)
            );

            let result = match self.conn.controller {
                Controller::STR1 => STR1::update(self).await,
                Controller::CN7500 => CN7500::update(self).await,
                Controller::Waveshare => Waveshare::update(self).await,
                Controller::WaveshareV2 => WaveshareV2::update(self).await,
            };

            if result.is_ok() {
                // If we get a good update, then return happy
                return Ok(());
            } else {
                // Otherwise, this update failed.
                // If we're on the last iteration of the loop
                // ie. the last retry and we still fail, then return the error
                if i == total_attempts {
                    // return the err
                    return result;
                }
                // Otherwise, log a message and sleep for a bit
                device_info!(&self, &format!("updating failed, but attempts remain. Waiting for retry_delay = {} ms before trying again.", self.retry_delay));
                std::thread::sleep(Duration::from_millis(self.retry_delay));
            }
        }

        panic!("Reached some code that shouldn't be reachable. Ran through all iterations of a device update loop without Ok() or Err()");
    }

    /// Attempts to write the internal state of this struct to the actual device. The inverse of
    /// `Device::update()`
    ///
    /// Will retry a few times, just as `update()` does.
    pub async fn enact(&mut self) -> Result<()> {
        let total_attempts = self.command_retries + 1;

        for i in 1..=total_attempts {
            device_info!(
                &self,
                &format!("enacting (attempt {i} of {})", total_attempts)
            );

            let result = match self.conn.controller {
                Controller::STR1 => STR1::enact(self).await,
                Controller::CN7500 => CN7500::enact(self).await,
                Controller::Waveshare => Waveshare::enact(self).await,
                Controller::WaveshareV2 => WaveshareV2::enact(self).await,
            };

            if result.is_ok() {
                return Ok(());
            } else {
                // Enaction failed
                // If we're on the last iteration of the loop
                // ie. the last retry and we still fail, then return the error
                if i == total_attempts {
                    return result;
                }
                device_info!(&self, &format!("enacting failed, but attempts remain. Waiting for retry_delay = {} ms before trying again.", self.retry_delay));
                std::thread::sleep(Duration::from_millis(self.retry_delay));
            }
        }

        panic!("Reached some code that shouldn't be reachable. Ran through all iterations of a device enact loop without Ok() or Err()");
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::tests::test_device_from_type;

    use super::*;
    use pretty_assertions::assert_eq;

    // TODO: Write more tests for devices

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

    #[tokio::test]
    async fn test_device_update_and_enact() {
        let mut device = test_device_from_type(Controller::WaveshareV2);

        // state is empty when we first generate the RTU data from the config file
        assert!(device.state.is_empty());

        // Turn it off to start the test
        device.state.relay_state = Some(BinaryState::Off);
        assert!(device.enact().await.is_ok());

        // Now we'll update. We should have a relay state (not None), but no SV or PV since this is
        // a relay
        assert!(device.update().await.is_ok());

        assert!(device.state.relay_state.is_some());
        assert!(device.state.sv.is_none());
        assert!(device.state.pv.is_none());

        device.state.relay_state = Some(BinaryState::On);
        assert!(device.enact().await.is_ok());

        // We updated the internal state and enacted, so they should theoretically match
        assert_eq!(device.state.relay_state, Some(BinaryState::On));
        // We'll update again and make sure
        assert!(device.update().await.is_ok());
        assert_eq!(device.state.relay_state, Some(BinaryState::On));

        // Turn it off to end the test
        device.state.relay_state = Some(BinaryState::Off);
        assert!(device.enact().await.is_ok());
    }
}
