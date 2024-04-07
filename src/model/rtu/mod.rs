pub mod connection;
pub mod device;

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

    /// Returns an owned Device, cloned from the original collection
    ///
    /// Probably should only use this for testing.
    #[allow(unused)]
    pub(crate) fn device_cloned(&mut self, device_id: &str) -> Option<Device> {
        self.devices
            .iter_mut()
            .find(|dev| dev.id == device_id)
            .cloned()
    }

    /// Deserializes an RTU from the conf file. This does *not* validate the RTU.
    ///
    /// You should probably call RTU::generate() rather than this. This is used when
    /// validating rules and conditions because in that context we only just need some
    /// info from the conf file.
    pub(crate) fn get_from_file() -> Result<RTU> {
        let file_path = config_file();
        info!("Generating RTU. Using config file: {:?}", file_path);
        // Get the contents of the config file
        let file_contents = fs::read_to_string(file_path).map_err(Error::IOError)?;
        // Deserialize the file
        serde_yaml::from_str::<RTU>(&file_contents).map_err(Error::YamlError)
    }

    /// Reads the configuration file and builds the representation of an RTU from that. It does not
    /// enact/update any devices, so if any state is stored in the RTU struct, it will be stale.
    ///
    /// This will fail if the RTU cannot be deserialized from the configuration file.
    ///
    /// This method calls [`RTU::validate()`](crate::model::RTU::validate) and returns an error if any of
    /// them don't succeed.
    pub fn generate() -> Result<Self> {
        let rtu = Self::get_from_file()?;
        info!("[RTU `{}`] generated.", rtu.id);
        Ok(rtu)
    }
}

#[cfg(test)]
mod tests {
    use crate::state::BinaryState;

    use super::*;

    use tokio::test;

    #[test]
    async fn test_generate_rtu() {
        let rtu = RTU::generate();
        assert!(rtu.is_ok());
        assert!(!rtu.unwrap().devices.is_empty());
    }

    #[test]
    async fn test_rtu_device_operations() {
        let mut rtu = RTU::generate().unwrap();

        let device = rtu.device("wsrelay0").unwrap();
        device.state.relay_state = Some(BinaryState::On);
        device.enact().await.unwrap();

        device.update().await.unwrap();
        assert_eq!(device.state.relay_state, Some(BinaryState::On));

        device.state.relay_state = Some(BinaryState::Off);
        device.enact().await.unwrap();
    }
}
