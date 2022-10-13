use std::collections::HashMap;
use std::fmt::Debug;

use crate::omega::CN7500;
use crate::relays::{STR1, Waveshare};

/// An enum containing all types of devices that we have drivers for

pub enum Device {
    STR1(STR1),
    Waveshare(Waveshare),
    CN7500(CN7500)
}

impl Debug for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // This could probably be refactored
        match self {
            Device::STR1(device) => {
                write!(f, "{:?}", device)
            },
            Device::Waveshare(device) => {
                write!(f, "{:?}", device)
            },
            Device::CN7500(device) => {
                write!(f, "{:?}", device)
            }
        }
    }
}

pub struct DevicePool {
    devices: HashMap<String, Device>
}

impl DevicePool {
    /// Create an empty Pool
    pub fn create() -> Self {
        DevicePool { devices: HashMap::new() }
    }

    pub fn devices(&self) -> &HashMap<String, Device> {
        &self.devices
    }

    /// Adds a device to the pool under the key. The device must be wrapped
    /// in the `Device` enum.
    /// 
    /// ```rust
    /// let cn7500 = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();
    /// let mut pool = DevicePool::create();
    /// 
    /// pool.add("omega_id", Device::CN7500(cn7500));
    /// ```
    pub fn add(&mut self, key: &str, device: Device) {
        self.devices.insert(String::from(key), device);
    }

    /// Gives a mutable reference to the device, if found
    /// 
    /// ```rust
    /// let cn7500 = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();
    /// let mut pool = DevicePool::create();
    /// 
    /// pool.add("omega_id", Device::CN7500(cn7500));
    /// if let Device::CN7500(device) = pool.device("omega_id").unwrap() {
    ///     assert!(device.set_sv(150.0).await.is_ok());
    /// }
    /// ```
    pub fn device(&mut self, key: &str) -> Option<&mut Device> {
        return self.devices.get_mut(key)
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_pool() {
        let str1 = STR1::connect(254, "/dev/ttyUSB0").unwrap();
        let cn7500 = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();

        let mut pool = DevicePool::create();
        pool.add("str1", Device::STR1(str1));
        pool.add("cn7500", Device::CN7500(cn7500));

        if let Device::CN7500(device) = pool.device("cn7500").unwrap() {
            assert!(device.set_sv(150.0).await.is_ok());
            assert_eq!(device.get_sv().await.unwrap(), 150.0);
        } else {
            assert!(false);
        }
    }
}