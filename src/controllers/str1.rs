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
//! See the [`STR1` struct](crate::controllers::STR1) or the `str1` example in the
//! [`examples/` directory](https://github.com/NavasotaBrewing/brewdrivers/tree/master/examples).

use std::time::Duration;

// external uses
use async_trait::async_trait;
use log::trace;

// internal uses
use crate::drivers::{
    serial::{Bytestring, SerialInstrument},
    InstrumentError, Result,
};
use crate::model::{Device, SCADADevice};
use crate::state::{BinaryState, StateError};

pub const STR1_BAUDRATES: [usize; 10] = [
    300, 600, 1200, 2400, 4800, 9600, 19200, 38400, 57600, 115200,
];

/// An `STR1XX` board.
///
/// This struct contains connection details for an STR108 or STR116 relay board.
#[derive(Debug)]
pub struct STR1(SerialInstrument);

#[async_trait]
impl SCADADevice for STR1 {
    async fn update(device: &mut Device) -> Result<()> {
        trace!("Updating STR1 device `{}`", device.id);
        let mut board = STR1::connect(
            device.conn.controller_addr(),
            &device.conn.port(),
            *device.conn.baudrate(),
            device.conn.timeout()
        )?;
        device.state.relay_state = Some(board.get_relay(device.conn.addr())?);
        Ok(())
    }

    async fn enact(device: &mut Device) -> Result<()> {
        trace!("Enacting STR1 device `{}`", device.id);
        let mut board = STR1::connect(
            device.conn.controller_addr(),
            &device.conn.port(),
            *device.conn.baudrate(),
            device.conn.timeout()
        )?;

        match device.state.relay_state {
            Some(new_state) => board.set_relay(device.conn.addr(), new_state)?,
            None => {
                return Err(InstrumentError::StateError(StateError::BadValue(
                    device.state.clone(),
                )))
            }
        }
        Ok(())
    }
}

impl STR1 {
    /// Attempts to connect to an STR1 board.
    pub fn connect(address: u8, port_path: &str, baudrate: usize, timeout: Duration) -> Result<Self> {
        trace!("(STR1 {}) connected", address);
        let mut str1 = STR1(SerialInstrument::new(
            address,
            port_path,
            baudrate,
            timeout,
        )?);
        str1.connected().map_err(|instr_err| {
            InstrumentError::serialError(
                format!(
                    "STR1 board connection failed, likely busy. Error: {}",
                    instr_err
                ),
                Some(address),
            )
        })?;
        Ok(str1)
    }

    /// Attempts to communicate with the board, returning Ok(()) if it responds.
    pub fn connected(&mut self) -> Result<()> {
        trace!("(STR1 {}) connected", self.0.address());
        self.relay_count()?;
        Ok(())
    }

    /// Sets a relay to On or Off.
    pub fn set_relay(&mut self, relay_num: u8, new_state: BinaryState) -> Result<()> {
        trace!(
            "(STR1 {}) setting relay {relay_num}: {new_state}",
            self.0.address()
        );
        let new_state_num = match new_state {
            BinaryState::Off => 0,
            BinaryState::On => 1,
        };

        self.write_to_device(Bytestring::from(vec![
            0x08,
            0x17,
            self.0.address(),
            relay_num,
            0x01,
            new_state_num,
        ]))?;

        Ok(())
    }

    /// Gets the status of a relay, as a [`State`](crate::controllers::BinaryState).
    pub fn get_relay(&mut self, relay_num: u8) -> Result<BinaryState> {
        trace!("(STR1 {}) getting relay {relay_num}", self.0.address());
        let bytes = Bytestring::from(vec![0x07, 0x14, self.0.address(), relay_num, 0x01]);
        let output_buf: Vec<u8> = self.write_to_device(bytes)?;

        let result = hex::encode(output_buf);

        match result.chars().nth(7) {
            Some('1') => return Ok(BinaryState::On),
            _ => return Ok(BinaryState::Off),
        }
    }

    /// Writes a command to the device. This is useful if you want to use a command
    /// that we haven't implemented with this struct. See the [software manual](https://www.smarthardware.eu/manual/str1xxxxxx_com.pdf)
    /// for a full list of commands.
    ///
    /// This method uses a [`Bytestring`](crate::drivers::serial::Bytestring) to serialize the bytes you pass in,
    /// meaning you don't have to add the `MA0`, `MA1`, `CS` (checksum), and `MA0` bytes that the board requires.
    pub fn write_to_device(&mut self, bytestring: Bytestring) -> Result<Vec<u8>> {
        trace!("(STR1 {}) writing to device", self.0.address());
        self.0.write_to_device(bytestring.to_bytes())
    }

    /// Lists all relays status. This prints to `stdout`, so it should really only
    /// be used in scripts and with the CLI.
    pub fn list_all_relays(&mut self) -> Result<()> {
        trace!("(STR1 {}) listing all relays", self.0.address());
        // Leave that space there >:(
        println!(
            " Controller {} (0x{:X})",
            self.0.address(),
            self.0.address()
        );

        println!("{0: >6} | {1: <6}", "Relay", "Status");

        for i in 0..self.relay_count()? {
            // TODO: #14 Replace this with the command that gets all the relays status
            println!("{0: >6} | {1: <6?}", i, self.get_relay(i));
        }

        Ok(())
    }

    /// Programs the controller number of the board. Be careful with this, don't forget the number.
    /// The new controller number should be `0x00`-`0xFF`.
    pub fn set_controller_num(&mut self, new_cn: u8) -> Result<()> {
        trace!(
            "(STR1 {}) setting controller number to {new_cn}",
            self.0.address()
        );
        let bs = Bytestring::from(vec![0x06, 0x01, self.0.address(), new_cn]);

        self.write_to_device(bs)?;

        self.0.set_address(new_cn);
        Ok(())
    }

    /// Sets the baudrate of the board. See [`STR1_BAUDRATES`](crate::controllers::str1::STR1_BAUDRATES)
    pub fn set_baudrate(&mut self, new_baudrate: usize) -> Result<()> {
        trace!("Setting STR1 (addr {}) baudrate to {}", self.0.address(), new_baudrate);
        match STR1_BAUDRATES.iter().position(|&rate| rate == new_baudrate ) {
            Some(baud_code) => {
                let bs = Bytestring::from(vec![0x08, 0x33, self.0.address(), 0xAA, 0x55, baud_code as u8]);
                self.write_to_device(bs)?;
                self.0.set_baudrate(new_baudrate);
                return Ok(())
            },
            None => {
                return Err(InstrumentError::SerialError {
                    msg: format!("Bad baudrate for STR1 `{}`", new_baudrate),
                    addr: Some(self.0.address()),
                });
            }
        }
    }

    /// Gets the amount of relays on this board, if any
    pub fn relay_count(&mut self) -> Result<u8> {
        trace!("(STR1 {}) getting relay count", self.0.address());
        let out = self.write_to_device(Bytestring::from(vec![0x05, 0x02, self.0.address()]))?;
        // return:
        // SL0, SL1, 0x09, number of outputs,
        // number of inputs, number of analog inputs,
        // number of analog outputs, 0, 0, CS, SLE
        if out.len() < 4 {
            return Err(InstrumentError::serialError(
                format!(
                    "The STR1 board didn't return the correct response, recieved {:?}",
                    out
                ),
                Some(self.0.address()),
            ));
        } else {
            return Ok(out[3]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controllers::Controller;

    fn test_board() -> STR1 {
        let device = crate::tests::test_device_from_type(Controller::STR1);
        STR1::connect(
            device.conn.controller_addr(),
            &device.conn.port(),
            *device.conn.baudrate(),
            device.conn.timeout()
        ).unwrap()
    }

    #[test]
    fn test_error_if_details_are_wrong() {
        let dev = STR1::connect(0xDD, "/dev/ttyUSB0", 9600, Duration::from_millis(50));
        assert!(dev.is_err());

        let dev2 = STR1::connect(0xFE, "/dev/doesntexist", 9600, Duration::from_millis(50));
        assert!(dev2.is_err());
    }

    #[test]
    fn test_board_connected() {
        let mut board = test_board();
        assert!(board.connected().is_ok());
    }

    #[test]
    fn set_get_relay_status() {
        let mut board = test_board();

        board.set_relay(0, BinaryState::On).unwrap();
        assert_eq!(BinaryState::On, board.get_relay(0).unwrap());

        board.set_relay(0, BinaryState::Off).unwrap();
        assert_eq!(BinaryState::Off, board.get_relay(0).unwrap());
    }

    #[test]
    fn set_controller_number() {
        let mut board = test_board();

        assert!(board.connected().is_ok());

        board.set_controller_num(253).unwrap();

        assert!(board.connected().is_ok());

        // Set it back
        board.set_controller_num(254).unwrap();

        assert!(board.connected().is_ok());
    }

    #[test]
    fn test_all_relays() {
        let mut board = test_board();
        for i in 0..16 {
            board.set_relay(i, BinaryState::On).unwrap();
        }

        for i in 0..16 {
            board.set_relay(i, BinaryState::Off).unwrap();
        }
    }

    #[test]
    fn test_relay_count() {
        let mut board = test_board();
        // I test on an STR108, so there should be 8. We may test on an STR116 with 16 relays
        // later though.
        let count = board.relay_count();
        assert!(count.is_ok());
        assert!(count.unwrap() % 8 == 0);
    }
}
