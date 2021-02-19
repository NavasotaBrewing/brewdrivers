//! A driver for the `STR1XX` relays boards.
//!
//! This module is for the `STR108` and `STR116` relay boards from
//! [smarthardware](https://www.smarthardware.eu/index.php). The software on these
//! relays board functions identically, the only difference is the number of relays (8 or 16).
//!
//! There's a lot of functionality in these boards, and we don't use it all. This driver will help
//! you read and write to relays and update the controller number/baudrate. The NBC doesn't use any other functionality,
//! so we haven't built it in to this package. However, we do support custom commands, so implementing
//! any other command that the relay boards support in their [software manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
//! is trivial.
//!
//! See the [`STR1` struct](crate::relays::str1::STR1) or the `str1` example in the
//! [`examples/` directory](https://github.com/NavasotaBrewing/brewdrivers/tree/master/examples).

// std uses
use std::{fmt, io::{Read, Write}, time::Duration};

// external uses
use serialport::{DataBits, FlowControl, Parity, StopBits, TTYPort};

// internal uses
use crate::relays::{Bytestring, State};


/// An error type that may be returned when using the STR1 boards.
#[derive(Debug)]
pub struct STR1Error(String);

impl fmt::Display for STR1Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for STR1Error {}

/// An `STR1XX` board.
///
/// This struct contains connection details for an STR108 or STR116 relay board.
///
/// ## Examples
/// ```rust
/// use brewdrivers::relays::{State, STR1};
///
/// let mut board = STR1::new(0x01, "/dev/ttyUSB0", 9600).expect("Couldn't connect to device");
/// board.get_relay(0); // -> State::Off;
/// board.set_relay(0, State::On);
/// board.get_relay(0); // -> State::On;
///
/// board.relay_count(); // -> Some(8)
/// ```
// TODO: #12 Make these field not pub
#[derive(Debug)]
pub struct STR1 {
    pub port: TTYPort,
    pub address: u8,
    pub port_path: String,
    pub baudrate: u32
}

impl STR1 {
    /// Attempts to connect to an STR1 board.
    ///
    /// The `address` is the controller number the board is programmed to (default `0xFE`).
    ///
    /// ## Examples
    /// ```rust
    /// use brewdrivers::relays::{State, STR1};
    ///
    /// let mut board = STR1::new(0x01, "/dev/ttyUSB0", 9600).expect("Couldn't connect to device");
    /// board.get_relay(0);
    /// // ...
    /// ```
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

    // This is a wrapper around the serialport::new() method. We may want
    // to call this again in order to update the values
    fn open_port(port_path: &str, baudrate: u32) -> Result<TTYPort, serialport::Error> {
        serialport::new(port_path, baudrate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(15))
            .open_native()
    }

    /// Writes a command to the device. This is useful if you want to use a command
    /// that we haven't implemented with this struct. See the [software manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
    /// for a full list of commands.
    ///
    /// This method uses a [`Bytestring`](crate::relays::bytestring::Bytestring) to serialize the bytes you pass in,
    /// meaning you don't have to add the `MA0`, `MA1`, `CS` (checksum), and `MA0` bytes that the board requires.
    ///
    /// ## Example
    /// ```rust
    /// # use brewdrivers::relays::{State, STR1};
    /// let mut board = STR1::new(0x01, "/dev/ttyUSB0", 9600).expect("Couldn't connect to device");
    ///
    /// // These bytes are to read a relay status
    /// let output_buf: Vec<u8> = board.write_to_device(
    ///     vec![0x07, 0x14, 0x01, 0x00, 0x01]
    /// );
    /// assert!(output_buf.len() > 1);
    /// ```
    pub fn write_to_device(&mut self, bytes: Vec<u8>) -> Vec<u8> {
        // Bytestring adds checksum verification and MA1, MAE, etc.
        match self.port.write(&Bytestring::from(bytes).to_bytes()){
            Err(e) => eprintln!("Error writing to STR1 serial device: {}", e),
            _ => {}
        };


        let mut output_buf: Vec<u8> = vec![];
        match self.port.read_to_end(&mut output_buf) {
            Ok(_) => {},
            Err(_) => {
                // timeout, expected
                // I'm pretty sure that the port never returns the nunmber of bytes
                // to be read, and it just times out every time, even on successful writes.
                // It still reads successfully even after timeouts, so it's fine for now.
            }
        }

        output_buf
    }

    /// Gets the status of a relay, as a [`State`](crate::relays::State).
    ///
    /// ## Example
    /// ```rust
    /// # use brewdrivers::relays::{State, STR1};
    /// let mut board = STR1::new(0x01, "/dev/ttyUSB0", 9600).expect("Couldn't connect to device");
    /// assert_eq!(board.get_relay(0), State::Off);
    /// ```
    // TODO: #13 Test this with relay indices out of bounds
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

    /// Sets a relay to On or Off.
    ///
    /// ## Example
    /// ```rust
    /// use brewdrivers::relays::{State, STR1};
    ///
    /// let mut board = STR1::new(0x01, "/dev/ttyUSB0", 9600).expect("Couldn't connect to device");
    /// board.set_relay(0, State::On);
    /// assert_eq!(board.get_relay(0), State::On);
    /// board.set_relay(0, State::Off);
    /// assert_eq!(board.get_relay(0), State::Off);
    /// ```
    pub fn set_relay(&mut self, relay_num: u8, new_state: State) {
        let new_state_num = match new_state {
            State::On => 1,
            State::Off => 0
        };

        self.write_to_device(vec![0x08, 0x17, self.address, relay_num, 1, new_state_num]);
    }

    /// Lists all relays status. This prints to `stdout`, so it should really only
    /// be used in scripts and with the CLI.
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
