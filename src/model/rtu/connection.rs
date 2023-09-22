use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::controllers::*;

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
