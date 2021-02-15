use std::{io::{Read, Write}, time::Duration};

use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits, TTYPort};

use crate::modbus::ModbusDevice;
use crate::relays::{Bytestring, State};

#[derive(Debug)]
pub struct STR1 {
    port: TTYPort,
    address: u8,
    port_path: String,
    baudrate: u32
}

impl STR1 {
    pub fn new(address: u8, port_path: &str, baudrate: u32) -> Self {
        let port = STR1::open_port(port_path, baudrate);
        STR1 {
            port,
            address,
            port_path: String::from(port_path),
            baudrate
        }
    }

    /// This is a wrapper around the serialport::new() method. We may want 
    /// to call this again in order to update the values
    fn open_port(port_path: &str, baudrate: u32) -> TTYPort {
        serialport::new(port_path, baudrate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(15))
            .open_native()
            .expect(&format!("Couldn't open serial port at {}", port_path))
    }


    pub fn write_to_device(&mut self, bytestring: &Bytestring) -> Vec<u8> {
        match self.port.write(&bytestring.to_bytes()){
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
        let bytestring = Bytestring::from(vec![0x07, 20, self.address, relay_num, 1]);

        let output_buf: Vec<u8> = self.write_to_device(&bytestring);
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
        let bytestring = Bytestring::from(vec![0x08, 0x17, self.address, relay_num, 1, new_state_num]);

        self.write_to_device(&bytestring);
    }

    pub fn list_all_relays(&mut self) {
        // Leave that space there >:(
        println!(" Controller {} (0x{:X})", self.address, self.address);
        println!(
            "{0: >6} | {1: <6}",
            "Relay", "Status"
        );
        for i in 0..16 {
            println!("{0: >6} | {1: <6?}", i, self.get_relay(i));
        }
    }
}

impl ModbusDevice for STR1 {
    fn port(&self) -> &str {
        &self.port_path
    }

    fn set_port(&mut self, new_path: &str) {
        self.port_path = String::from(new_path);
        self.port = STR1::open_port(new_path, self.baudrate);
    }

    fn baudrate(&self) -> u32 {
        self.baudrate
    }

    fn set_baudrate(&mut self, new_baud: u32) {
        self.baudrate = new_baud;
        self.port = STR1::open_port(&self.port_path, new_baud);
    }

    fn address(&self) -> u8 {
        self.address
    }

    fn set_address(&mut self, new_addr: u8) {
        self.address = new_addr;
        // Don't need to reopen the port, addr is only used
        // in communication, not opening.
    }

    /// Opens the port with the configured settings. Does not change any fields of
    /// the STR1 board.
    fn open(&self) -> TTYPort {
        STR1::open_port(&self.port_path, self.baudrate)
    }
}
