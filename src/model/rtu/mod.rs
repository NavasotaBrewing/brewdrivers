pub mod connection;
pub mod device;
mod rtu_validators;

use std::fs;
use std::net::Ipv4Addr;

use log::*;
use serde::{Deserialize, Serialize};

use super::Device;
use crate::defaults::config_file;
use crate::{error::Error, Result};

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
}

impl RTU {
    /// This calls [`Device::enact`](crate::model::Device::enact) on each device in the RTU.
    /// Returns the first Err() encountered.
    ///
    /// This is not used very often, because it enacts every device.
    //
    // TODO: Maybe collect errors and return a list of errors, if any?
    pub async fn enact(&mut self) -> Result<()> {
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
    pub async fn update(&mut self) -> Result<()> {
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
    pub fn generate() -> Result<RTU> {
        let file_path = config_file();
        info!("Generating RTU. Using config file: {:?}", file_path);
        // TODO: Get IPv4 here programatically instead of writing it in the file

        // Get the contents of the config file
        let file_contents = fs::read_to_string(file_path).map_err(Error::IOError)?;

        // Deserialize the file. Return an Err if it doesn't succeed
        let rtu = serde_yaml::from_str::<RTU>(&file_contents).map_err(Error::YamlError)?;

        info!("[RTU `{}`] generated.", rtu.id);
        rtu.validate()?;
        Ok(rtu)
    }

    /// Run all the [`validators`](crate::model::validators). Return an error if any of them don't succeed.
    pub fn validate(&self) -> Result<()> {
        if let Err(e) = rtu_validators::all_validators(self) {
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
        let rtu = RTU::generate();
        assert!(rtu.is_ok());
        assert!(!rtu.unwrap().devices.is_empty());
    }
}