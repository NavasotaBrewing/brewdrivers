use std::fs;
use std::net::Ipv4Addr;

use log::*;
use serde::{Deserialize, Serialize};

use crate::drivers::InstrumentError;

use super::{conditions::Condition, validators, Device, ModelError};

/// A digital representation of an RTU.
///
/// This is meant to be serialized from a configuration file. This is
/// also the data structure that is sent between the iris server and the front-end
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RTU {
    /// The RTU name, for display purposes
    pub name: String,
    /// The RTU id, must be unique among all RTUs and not contain whitespace
    pub id: String,
    /// The IP address of the RTU. Later, this may be generated for you, but
    /// for now it's manually set.
    pub ip_addr: Ipv4Addr,
    /// A list of devices connected to the RTU
    pub devices: Vec<Device>,
    /// A list of condition definitions
    //
    // We don't serialize, because there's really no reason to send these out
    #[serde(skip_serializing)]
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

impl RTU {
    /// This calls [`Device::enact`](crate::model::Device::enact) on each device in the RTU.
    /// Returns the first Err() encountered.
    ///
    /// This is not used very often, because it enacts every device.
    //
    // TODO: Maybe collect errors and return a list of errors, if any?
    pub async fn enact(&mut self) -> Result<(), InstrumentError> {
        info!("[RTU `{}`] enacting...", self.id);
        for dev in self.devices.iter_mut() {
            dev.enact().await?;
        }
        info!("[RTU `{}`] enacted.", self.id);
        Ok(())
    }

    /// This calls [`Device::update`](crate::model::Device::update) on each device in the RTU
    //
    // TODO: Same as above, return a list off all errors, if any
    pub async fn update(&mut self) -> Result<(), InstrumentError> {
        info!("[RTU `{}`] updating...", self.id);
        for dev in self.devices.iter_mut() {
            dev.update().await?;
        }
        info!("[RTU `{}`] updated.", self.id);
        Ok(())
    }

    /// Returns an optional mutable borrow to a `Device`
    pub fn device(&mut self, device_id: &str) -> Option<&mut Device> {
        self.devices.iter_mut().find(|dev| dev.id == device_id)
    }

    /// Reads the configuration file and builds an RTU from that. Note that while this method
    /// does take an optional file path, that's just used for testing purposes. You should pass
    /// `None` to this method and use the defualt configuration file at
    /// [crate::defaults](crate::defaults).
    ///
    /// This will fail if the RTU cannot be deserialized from the configuration file.
    ///
    /// This method calls [`RTU::validate()`](crate::model::RTU::validate) and returns an error if any of
    /// them don't succeed.
    pub fn generate(conf_path: Option<&str>) -> Result<RTU, ModelError> {
        let file_path = conf_path.or(Some(crate::defaults::config_file()));
        info!("Generating RTU. Using config file: {:?}", file_path);
        // TODO: Get IPv4 here programatically instead of writing it in the file

        // Get the contents of the config file
        let file_contents = fs::read_to_string(
            // this is safe
            file_path.unwrap(),
        )
        .map_err(|err| ModelError::IOError(err))?;

        // Deserialize the file. Return an Err if it doesn't succeed
        let rtu = serde_yaml::from_str::<RTU>(&file_contents)
            .map_err(|err| ModelError::SerdeParseError(err))?;

        info!("[RTU `{}`] generated.", rtu.id);
        rtu.validate()?;
        Ok(rtu)
    }

    /// Run all the [`validators`](crate::model::validators). Return an error if any of them don't succeed.
    pub fn validate(&self) -> Result<(), ModelError> {
        use validators::*;

        if let Err(e) = all_validators(&self) {
            error!("{e}");
            return Err(e);
        }

        info!("RTU passed all validators");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use tokio::test;

    #[test]
    async fn test_generate_rtu() {
        let rtu = RTU::generate(Some(crate::defaults::test_config_file()));
        assert!(rtu.is_ok());
        assert!(rtu.unwrap().devices.len() > 0);
    }
}
