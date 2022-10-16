use std::collections::HashMap;

use crate::controllers::Controller;

/// A simple controller pool to store device connections. This is useful when you want
/// to maintain one device connection throughout a program and only use it occasionally.;
pub struct ControllerPool {
    controllers: HashMap<String, Controller>
}

impl ControllerPool {
    /// Create an empty Pool
    pub fn create() -> Self {
        ControllerPool { controllers: HashMap::new() }
    }

    /// Returns a reference to all the controllers
    pub fn controllers(&self) -> &HashMap<String, Controller> {
        &self.controllers
    }

    /// Adds a controller to the pool under the key. The controller must be wrapped
    /// in the `Controller` enum.
    pub fn add(&mut self, key: &str, controller: Controller) {
        self.controllers.insert(String::from(key), controller);
    }

    /// Gives a mutable reference to the controller, if found
    pub fn controller(&mut self, key: &str) -> Option<&mut Controller> {
        return self.controllers.get_mut(key)
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::controllers::{STR1, CN7500};

    #[tokio::test]
    async fn test_device_pool() {
        let str1 = STR1::connect(254, "/dev/ttyUSB0").unwrap();
        let cn7500 = CN7500::connect(0x16, "/dev/ttyUSB0").await.unwrap();

        let mut pool = ControllerPool::create();
        pool.add("str1", Controller::STR1(str1));
        pool.add("cn7500", Controller::CN7500(cn7500));

        if let Controller::CN7500(device) = pool.controller("cn7500").unwrap() {
            assert!(device.set_sv(150.0).await.is_ok());
            assert_eq!(device.get_sv().await.unwrap(), 150.0);
        } else {
            assert!(false);
        }
    }
}