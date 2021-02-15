//! This module contains traits that are common to all Modbus RS-485 devices

use serialport::TTYPort;



/// All Modbus Devices have a port (like /dev/ttyUSB0), an address (like 2, or 0xFE), and a baudrate (like 9600)
pub trait ModbusDevice {
    fn port(&self) -> &str;
    fn set_port(&mut self, new_path: &str);
    fn address(&self) -> u8;
    fn set_address(&mut self, new_addr: u8);
    fn baudrate(&self) -> u32;
    fn set_baudrate(&mut self, new_baud: u32);
    fn open(&self) -> TTYPort;
}
