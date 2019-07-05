// std library
use std::io::{Write, Read};
use std::path::Path;
use std::time::Duration;

// extern crates
use hex;
use serialport::prelude::*;
use serialport::posix::TTYPort;
use retry::{retry, OperationResult};
use retry::delay::Fixed;

// this crate
use crate::relays::State::{On, Off};
use crate::relays::Bytestring;


#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}

#[derive(Debug)]
pub struct STR1 {
    pub address: u8,
    pub port: TTYPort,
}

impl STR1 {
    pub fn new(address: u8) -> STR1 {
        STR1 {
            address,
            port: STR1::port()
        }
    }

    fn port() -> TTYPort {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(20);
        settings.baud_rate = 9600;
        settings.data_bits = DataBits::Eight;
        settings.flow_control = FlowControl::None;
        settings.parity = Parity::None;
        settings.stop_bits = StopBits::One;


        let port = retry(Fixed::from_millis(10), || {
            match TTYPort::open(&Path::new("/dev/ttyAMA0"), &settings) {
                Ok(port) => OperationResult::Ok(port),
                Err(_) => OperationResult::Retry("Port busy")
            }
        });

        port.unwrap()
    }

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
        // Leave that space there >:(
        println!(" Controller {}", self.address);
        println!(
            "{0: >6} | {1: <6}",
            "Relay", "Status"
        );
        for i in 0..16 {
            println!("{0: >6} | {1: <6?}", i, self.get_relay(i));
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_relay_status() {
        let mut board = STR1::new(254);
        board.set_relay(0, State::Off);
        assert_eq!(State::Off, board.get_relay(0));

        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));
    }

    #[test]
    fn set_controller_number() {
        let mut board = STR1::new(254);

        // Checks communication
        board.set_relay(0, State::Off);
        assert_eq!(State::Off, board.get_relay(0));
        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));

        board.set_controller_num(253);

        // Checks communication again
        board.set_relay(0, State::Off);
        assert_eq!(State::Off, board.get_relay(0));
        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));

        // Set it back
        board.set_controller_num(254);

    }
}
