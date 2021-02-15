use std::{fmt, io::{Read, Write}, time::Duration};

use serialport::{DataBits, FlowControl, Parity, StopBits, TTYPort};

use crate::relays::{Bytestring, State};


#[derive(Debug)]
pub struct STR1Error(String);

impl fmt::Display for STR1Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

impl std::error::Error for STR1Error {}

#[derive(Debug)]
pub struct STR1 {
    pub port: TTYPort,
    pub address: u8,
    pub port_path: String,
    pub baudrate: u32
}

impl STR1 {
    pub fn new(address: u8, port_path: &str, baudrate: u32) -> Result<Self, STR1Error> {
        let port = STR1::open_port(port_path, baudrate).map_err(|err| {
            STR1Error(format!("Couldn't open serial port at {}: {}", port_path, err))
        });

        
        if port.is_err() {
            return Err(port.unwrap_err());
        } else {
            Ok(STR1 {
                port: port.unwrap(),
                address,
                port_path: String::from(port_path),
                baudrate
            })
        }

    }

    /// This is a wrapper around the serialport::new() method. We may want 
    /// to call this again in order to update the values
    fn open_port(port_path: &str, baudrate: u32) -> Result<TTYPort, serialport::Error> {
        serialport::new(port_path, baudrate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(15))
            .open_native()
    }

    pub fn write_to_device(&mut self, bytes: Vec<u8>) -> Vec<u8> {
        // Bytestring adds checksum verification and MA1, MAE, etc.
        match self.port.write(&Bytestring::from(bytes).to_bytes()){
            Err(e) => eprintln!("Error writing to STR1 serial device: {}", e),
            _ => {}
        };

        
        let mut output_buf: Vec<u8> = vec![];
        match self.port.read_to_end(&mut output_buf) {
            Ok(_) => {},
            Err(_) => { /* timeout */ }
        }

        output_buf
    }

    pub fn get_relay(&mut self, relay_num: u8) -> State {
        let output_buf: Vec<u8> = self.write_to_device(
            vec![0x07, 0x14, self.address, relay_num, 1]
        );

        let result = hex::encode(output_buf);

        match result.chars().nth(7) {
            Some('1') => return State::On,
            _ => return State::Off,
        }
    }

    pub fn set_relay(&mut self, relay_num: u8, new_state: State) {
        let new_state_num = match new_state {
            State::On => 1,
            State::Off => 0
        };

        // From the software guide
        // MA0, MA1, 0x08, 0x17, CN, start number output, number of outputs, 0/1, CS, MAE
        // MA0, MA1, CS, and MAE are added automatically
        self.write_to_device(vec![0x08, 0x17, self.address, relay_num, 1, new_state_num]);
    }

    pub fn list_all_relays(&mut self) {
        // Leave that space there >:(
        println!(" Controller {} (0x{:X})", self.address, self.address);
        println!(
            "{0: >6} | {1: <6}",
            "Relay", "Status"
        );
        for i in 0..self.relay_count().unwrap() {
            println!("{0: >6} | {1: <6?}", i, self.get_relay(i));
        }
    }

    pub fn set_controller_num(&mut self, new_cn: u8) {
        self.write_to_device(vec![
            0x06, 0x01, self.address, new_cn
        ]);
        self.address = new_cn;
        self.port = STR1::open_port(&self.port_path, self.baudrate).expect("Couldn't open serial port");
    }

    pub fn set_baudrate(&mut self, new_baud: u32) -> Result<(), String> {
        // The boards expects a single digit value, 0 = 300, 1 = 600, etc.
        // Just the index of this vector.
        let acceptable = vec![300, 600, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200];
        let index = acceptable.iter().position(|&x| x == new_baud);

        if index.is_none() {
            return Err(format!("Unacceptable baudrate {}, options are {:?}", new_baud, acceptable));
        };

        self.write_to_device(vec![
            0x08, 0x33, self.address, 0xAA, 0x55, (index.unwrap() as u8)
        ]);
        self.port = STR1::open_port(&self.port_path, new_baud).expect(
            &format!("Couldn't reopen serial port after changing baudrate from {} to {}", self.baudrate, new_baud)
        );
        self.baudrate = new_baud;
        
        Ok(())
    }

    pub fn relay_count(&mut self) -> Option<u8> {
        let out = self.write_to_device(
            vec![0x05, 0x02, self.address]
        );
        // return:
        // SL0, SL1, 0x09, number of outputs,
        // number of inputs, number of analog inputs,
        // number of analog outputs, 0, 0, CS, SLE
        if out.len() < 4 {
            return None;
        } else {
            return Some(out[3]);
        }
    }

    pub fn connected(&mut self) -> bool {
        return self.relay_count().is_some()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn test_board() -> STR1 {
        let device = STR1::new(0xFE, "/dev/ttyUSB0", 9600);
        return device.unwrap();
    }


    #[test]
    #[serial]
    fn test_board_connected() {
        let mut board = test_board();
        assert!(board.connected());
        
        board.address = 4;
        
        assert!(!board.connected());
    }

    #[test]
    #[serial]
    fn test_write_to_device() {
        let mut board = test_board();
        // These bytes are the get the amount of relays on the board
        // Just a test
        let bytes = vec![0x05, 0x02, board.address];
        let output = board.write_to_device(bytes);
        assert!(output.len() > 0);
    }

    #[test]
    #[serial]
    fn set_get_relay_status() {
        let mut board = test_board();

        board.set_relay(0, State::On);
        assert_eq!(State::On, board.get_relay(0));

        board.set_relay(0, State::Off);
        assert_eq!(State::Off, board.get_relay(0));
    }

    #[test]
    #[serial]
    fn set_controller_number() {
        let mut board = test_board();
        
        assert!(board.connected());
        
        board.set_controller_num(253);
        
        assert!(board.connected());
        
        // Set it back
        board.set_controller_num(254);
        
        assert!(board.connected());
    }

    #[test]
    #[serial]
    fn test_change_baudrate() {
        // Not going to test this because I think the baudrate on the board
        // doesn't work properly. I was able to connect at 2 different baudrates and
        // both worked.
    }

    #[test]
    #[serial]
    fn test_relay_count() {
        let mut board = test_board();
        // I test on an STR108, so there should be 8. We may test on an STR116 with 16 relays
        // later though.
        let count = board.relay_count();
        assert!(count.is_some());
        assert!(count.unwrap() % 8 == 0);
    }
}