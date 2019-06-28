use std::time::Duration;
use std::io::Write;
use std::path::Path;

use hex;
use serialport::prelude::*;
use serialport::posix::TTYPort;
use std::io::Read;


use crate::relays::State::{On, Off};
use crate::relays::Bytestring;


#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}

#[derive(Debug)]
pub enum BaudRate {
    Baud300,
    Baud600,
    Baud1200,
    Baud2400,
    Baud4800,
    Baud9600,
    Baud19200,
    Baud38400,
    Baud57600,
    Baud115200,
}


#[derive(Debug)]
pub struct Str1xx {
    pub address: u8,
    pub port: TTYPort,
}

impl Str1xx {
    pub fn new(address: u8) -> Str1xx {
        Str1xx {
            address,
            port: Str1xx::port()
        }
    }

    // Returns a port object to write to or read from
    fn port() -> TTYPort {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(20);
        settings.baud_rate = 9600;
        settings.data_bits = DataBits::Eight;
        settings.flow_control = FlowControl::None;
        settings.parity = Parity::None;
        settings.stop_bits = StopBits::One;


        TTYPort::open(&Path::new("/dev/ttyAMA0"), &settings).expect("Couldn't open port")
    }

    // Write to the device and return the bytearray it sends back
    pub fn write(&mut self, bytestring: Bytestring) -> Vec<u8> {
        match self.port.write(&bytestring.to_bytes()) {
            Ok(_) => {},
            Err(_) => println!("Error writing to serial device!"),
        };

        let mut output_buf: Vec<u8> = vec![];
        match self.port.read_to_end(&mut output_buf) {
            Ok(_) => {},
            Err(_) => { /* timeout, expected */ }
        }
        output_buf
    }

    pub fn set_relay(&mut self, relay_num: u8, state: State) {
        let new_state = match state {
            On => 1,
            Off => 0
        };

        // From the software guide
        // MA0, MA1, 0x08, 0x17, CN, start number output, number of outputs, 0/1, CS, MAE
        // MA0, MA1, CS, and MAE are added automatically
        let bytestring = Bytestring::from(vec![8, 23, self.address, relay_num, 1, new_state]);

        self.write(bytestring);
    }

    pub fn get_relay(&mut self, relay_num: u8) -> State {
        let bytestring = Bytestring::from(vec![7, 20, self.address, relay_num, 1]);
        let output_buf = self.write(bytestring);
        let result = hex::encode(output_buf);
        match result.chars().nth(7) {
            Some('1') => return On,
            _ => return Off,
        }
    }

    pub fn set_controller_num(&mut self, new_cn: u8) {
        // MA0, MA1, 0x06, 0x01, CN, newCN, CS, MAE
        let bytestring = Bytestring::from(vec![6, 1, self.address, new_cn]);
        println!("{:?}", self.write(bytestring));
        self.address = new_cn;
    }

    pub fn list_all_relays(&mut self) {
        println!("Controller {}", self.address);
        for i in 0..16 {
            std::thread::sleep(Duration::from_millis(10));
            println!("Relay {}: {:?}", i, self.get_relay(i));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_relay_status() {
        let mut board = Str1xx::new(2);
        board.set_relay(0, State::Off);

        assert_eq!(State::Off, board.get_relay(0));

        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));
    }
}
