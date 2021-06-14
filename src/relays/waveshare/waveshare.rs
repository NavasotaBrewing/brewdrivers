#![allow(dead_code)]
use std::time::Duration;
use std::fmt;
use serialport::{DataBits, FlowControl, Parity, StopBits, TTYPort};

#[derive(Debug)]
struct WaveshareError(String);

impl fmt::Display for WaveshareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for WaveshareError {}



#[derive(Debug)]
struct Waveshare {
    address: u8,
    port: TTYPort,
}

impl Waveshare {
    // This is a wrapper around the serialport::new() method. We may want
    // to call this again in order to update the values. This is the same as the STR1::open_port method
    fn open_port(port_path: &str, baudrate: u32) -> Result<TTYPort, serialport::Error> {
        serialport::new(port_path, baudrate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(15))
            .open_native()
    }
    
    pub fn new(addr: u8, port_path: &str) -> Result<Waveshare, WaveshareError> {
        let port = Waveshare::open_port(port_path, 9600).map_err(|err| {
            WaveshareError(format!("Couldn't open serial port at {}: {}", port_path, err))
        });

        Ok(Waveshare {
            address: addr,
            port: port?
        })
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_connection() {
        let ws = Waveshare::new(0x01, "/dev/ttyUSB0");
        ws.unwrap();
        // assert!(ws.is_ok());
    }
}

