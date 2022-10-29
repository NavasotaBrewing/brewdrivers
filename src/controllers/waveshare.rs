//! These boards are relatively cheap and (so far) reliable. They can be [found here](https://www.waveshare.com/modbus-rtu-relay.htm).
//! The [operation wiki](https://www.waveshare.com/wiki/Protocol_Manual_of_Modbus_RTU_Relay) explains how to use it, but you probably won't need that.
//!
//! See the `examples/` directory for a complete example of using this board. Here's a sneak peak
//! ```rust
//! use brewdrivers::controllers::*;
//!
//! let mut ws = Waveshare::connect(0x01, "/dev/ttyUSB0").unwrap();
//! ws.set_relay(3, BinaryState::On).unwrap(); // Turn on the 4th relay (indexed from 0)
//! println!("{:?}", ws.get_all_relays().unwrap());
//! ```

// ext uses
// Used for checksums
use crc::{Crc, CRC_16_MODBUS};

// internal uses
use crate::controllers::{BinaryState, RelayBoard};
use crate::drivers::serial::SerialInstrument;
use crate::drivers::{InstrumentError, Result};


// This is the checksum algorithm that the board uses
const CRC_MODBUS: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
// Hardcode the baud, we *probably* won't need to change it
pub const WAVESHARE_BAUD: usize = 9600;

#[allow(dead_code)]
/// The max index of a relay on the Waveshare board
pub const RELAY_MAX: u8 = 7;

/// A Waveshare board.
#[derive(Debug)]
pub struct Waveshare(SerialInstrument);

impl RelayBoard<Waveshare> for Waveshare {
    /// Connect to a board at the given address and port. This will fail if the port can't be opened,
    /// or if the board can't be communicated with. This method will poll the board for it's software
    /// version number and fail if it doesn't return one, returning an [`InstrumentError`](crate::drivers::InstrumentError).
    ///
    /// ```rust,no_run
    /// use brewdrivers::controllers::*;
    ///
    /// let mut ws = Waveshare::connect(0x01, "/dev/ttyUSB0").unwrap();
    /// ws.get_relay(0).unwrap();
    /// // ...
    /// ```
    fn connect(address: u8, port_path: &str) -> Result<Waveshare> {
        let mut ws = Waveshare(SerialInstrument::new(
            address,
            port_path,
            WAVESHARE_BAUD,
        )?);
        ws.connected()?;
        Ok(ws)
    }

    fn connected(&mut self) -> Result<()> {
        self.software_revision()?;
        Ok(())
    }

    /// Sets a relay to the given state. See the [`BinaryState`](crate::controllers::BinaryState) enum.
    ///
    /// ```rust,no_run
    /// use brewdrivers::controllers::*;
    ///
    /// let mut ws = Waveshare::connect(0x01, "/dev/ttyUSB0").unwrap();
    /// ws.set_relay(0, BinaryState::On).unwrap();
    /// assert_eq!(ws.get_relay(0).unwrap(),BinaryState::On);
    /// ws.set_relay(0, BinaryState::Off);
    /// ```
    fn set_relay(&mut self, relay_num: u8, state: BinaryState) -> Result<()> {
        // Example: 01 05 00 00 FF 00 8C 3A
        // 01       Device address	    0x00 is broadcast address；0x01-0xFF are device addresses
        // 05       05 Command	        Command for controlling Relay
        // 00 00	Address	            The register address of controlled Relay, 0x00 - 0x0008
        // FF 00	Command	            0xFF00：Open Replay;
        //                              0x0000：Close Relay;
        //                              0x5500：Flip Relay
        // 8C 3A	CRC16	            The CRC checksum of first six bytes.
        let mut bytes: Vec<u8> = vec![
            // Address
            self.0.address(),
            // Command to control a relay
            0x05,
            // Relay number (ours only has 8)
            0x00,
            relay_num,
        ];

        // Add on state
        match state {
            BinaryState::On => bytes.push(0xFF),
            BinaryState::Off => bytes.push(0x00),
        };

        // Add on 0x00, because the board needs it I guess
        bytes.push(0x00);

        Waveshare::append_checksum(&mut bytes).unwrap();

        self.0.write_to_device(bytes)?;
        Ok(())
    }

    /// Gets a relay state. See [`BinaryState`](crate::controllers::BinaryState).
    fn get_relay(&mut self, relay_num: u8) -> Result<BinaryState> {
        let statuses: Vec<BinaryState> = self.get_all_relays()?;

        if let Some(&state) = statuses.get(relay_num as usize) {
            return Ok(state);
        } else {
            return Err(
                InstrumentError::serialError(
                    format!(
                        "The board didn't return the proper amount of statuses, tried relay {}, found: {:?}",
                        relay_num,
                        statuses
                    ),
                    Some(self.0.address())
                )
            );
        }
    }
}

impl Waveshare {
    // Calculates the CRC checksum for the data bytes to send to the board
    fn append_checksum(bytes: &mut Vec<u8>) -> Result<()> {
        let checksum = CRC_MODBUS.checksum(&bytes).to_le_bytes();
        bytes.push(checksum[0]);
        bytes.push(checksum[1]);
        Ok(())
    }

    /// Returns a `Vec<BinaryState>` of all 8 relays.
    pub fn get_all_relays(&mut self) -> Result<Vec<BinaryState>> {
        let mut bytes: Vec<u8> = vec![self.0.address(), 0x01, 0x00, 0xFF, 0x00, 0x01];
        Waveshare::append_checksum(&mut bytes)?;

        let resp = self.0.write_to_device(bytes)?;
        if let Some(status_number) = resp.get(3) {
            // this is a little cursed but i don't know how else to work with binary
            let binary = format!("{:08b}", status_number);
            let statuses: Vec<BinaryState> = binary
                .chars()
                .filter(|&ch| ch == '1' || ch == '0')
                .map(|ch| {
                    // Usually 0 and 1 are stepper states, not binary
                    // That's why theres no FromStr for BinaryState
                    match ch {
                        '1' => BinaryState::On,
                        '0' => BinaryState::Off,
                        _ => BinaryState::default()
                    }
                } )
                .rev()
                .collect();

            Ok(statuses)
        } else {
            Err(InstrumentError::serialError(
                format!(
                    "Board did not return the proper response, received {:?}",
                    resp
                ),
                Some(self.0.address()),
            ))
        }
    }

    /// Returns the software revision as a String like "v1.00"
    ///
    /// ```no_run
    /// use brewdrivers::controllers::*;
    ///
    /// let mut ws = Waveshare::connect(0x01, "/dev/ttyUSB0").unwrap();
    /// assert_eq!(ws.software_revision().unwrap(), "v1.00");
    /// ```
    pub fn software_revision(&mut self) -> Result<String> {
        let mut bytes: Vec<u8> = vec![self.0.address(), 0x03, 0x20, 0x00, 0x00, self.0.address()];

        Waveshare::append_checksum(&mut bytes)?;

        let resp = self.0.write_to_device(bytes)?;

        if let Some(&version_num) = resp.get(3) {
            Ok(format!("v{:.2}", (version_num as f64 / 100.0)))
        } else {
            Err(
                InstrumentError::serialError(
                    format!(
                        "The board didn't return it's software revision correctly. Possible connection issue. port: {:?}, response: {:?}",
                        self.0.port(),
                        resp
                    ),
                    Some(self.0.address())
                )
            )
        }
    }

    /// Attempts to find the address of connected boards in the RS-485 circuit.
    ///
    /// **Note:** The documentation on this is pretty unclear. Apparently, sending a certain message
    /// on the broadcast address (0x00) gets the address of one board on the circuit (returns it's address). This
    /// works for one board, but I'm not sure what will happen if there's multiple boards. I'm too poor to afford more than
    /// one at the moment. Call UTA about reducing my tuition if you want better documentation.
    ///
    /// ```no_run
    /// use brewdrivers::controllers::*;
    ///
    /// // address 0x00, the broadcast address
    /// let mut ws = Waveshare::connect(0x00, "/dev/ttyUSB0").unwrap();
    /// assert_eq!(ws.get_address().unwrap(), 0x01);
    /// ```
    pub fn get_address(&mut self) -> Result<u8> {
        let mut bytes: Vec<u8> = vec![0x00, 0x03, 0x40, 0x00, 0x0, 0x01];

        Waveshare::append_checksum(&mut bytes)?;

        let resp = self.0.write_to_device(bytes)?;
        resp.get(3)
            .ok_or(InstrumentError::serialError(
                format!(
                    "The board didn't return the proper response, recieved: {:?}",
                    resp
                ),
                Some(self.0.address()),
            ))
            .copied()
    }

    /// Sets the address of a board. You don't need to reconnect to the board
    /// after changing it. It's a good idea to remember the controller number in
    /// case it becomes inaccessible. Almost all communication requires the controller
    /// number. The documentation for this board is spotty.
    ///
    /// ```no_run
    /// use brewdrivers::controllers::*;
    ///
    /// let mut ws = Waveshare::connect(0x01, "/dev/ttyUSB0").unwrap();
    /// let mut unknown_board = Waveshare::connect(0x00, "/dev/ttyUSB0").unwrap();
    ///
    /// ws.set_address(0x07).unwrap();
    /// assert_eq!(unknown_board.get_address().unwrap(), 0x07);
    /// ```
    pub fn set_address(&mut self, new_addr: u8) -> Result<()> {
        let mut bytes: Vec<u8> = vec![self.0.address(), 0x06, 0x40, 0x00, 0x00, new_addr];

        Waveshare::append_checksum(&mut bytes)?;

        let _resp = self.0.write_to_device(bytes)?;
        self.0.set_address(new_addr);
        Ok(())
    }

    /// Sets all relays at once to the given state.
    pub fn set_all_relays(&mut self, state: BinaryState) -> Result<()> {
        let mut bytes: Vec<u8> = vec![
            self.0.address(),
            // These are all constant, reading all relays status
            0x05,
            0x00,
            0xFF,
        ];

        match state {
            BinaryState::On => {
                bytes.push(0xFF);
                bytes.push(0xFF);
            }
            BinaryState::Off => {
                bytes.push(0x00);
                bytes.push(0x00);
            }
        }

        Waveshare::append_checksum(&mut bytes)?;

        self.0.write_to_device(bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    use std::thread::sleep;
    use std::time::Duration;

    // Helper function
    fn ws() -> Waveshare {
        Waveshare::connect(0x01, "/dev/ttyUSB0").unwrap()
    }

    #[test]
    
    fn test_connect_to_waveshare() {
        let ws = Waveshare::connect(0x01, "/dev/ttyUSB0");
        assert!(ws.is_ok());
    }

    #[test]
    
    fn test_crc_16_checksum() {
        let checksum = CRC_MODBUS.checksum(&[0x01, 0x05, 0x00, 0x00, 0xFF, 0x00]);
        assert_eq!(checksum, 0x3A8C);

        // test swapping the bytes
        // from 0x3A8C we want [8C, 3A]
        assert_eq!([0x8C, 0x3A], checksum.to_le_bytes());
    }

    #[test]
    
    fn test_write_relay_state() {
        let mut ws = ws();

        assert!(ws.set_relay(0, BinaryState::On).is_ok());
        sleep(Duration::from_millis(200));
        assert!(ws.set_relay(0, BinaryState::Off).is_ok());
    }

    #[test]
    
    fn test_get_relay_status() {
        let mut ws = ws();

        ws.set_relay(0, BinaryState::On).unwrap();
        assert_eq!(ws.get_relay(0).unwrap(), BinaryState::On);

        ws.set_relay(0, BinaryState::Off).unwrap();
        assert_eq!(ws.get_relay(0).unwrap(), BinaryState::Off);
    }

    #[test]
    
    fn test_write_all_relays() {
        let mut ws = ws();

        ws.set_all_relays(BinaryState::On).unwrap();
        for i in 0..8 {
            assert_eq!(ws.get_relay(i).unwrap(), BinaryState::On);
        }
        sleep(Duration::from_millis(200));
        ws.set_all_relays(BinaryState::Off).unwrap();
    }

    #[test]
    
    fn test_get_all_relays_status() {
        let mut ws = ws();

        let expected = vec![
            BinaryState::On,
            BinaryState::Off,
            BinaryState::Off,
            BinaryState::Off,
            BinaryState::Off,
            BinaryState::Off,
            BinaryState::On,
            BinaryState::Off,
        ];

        ws.set_all_relays(BinaryState::Off).unwrap();
        ws.set_relay(0, BinaryState::On).unwrap();
        ws.set_relay(6, BinaryState::On).unwrap();
        assert_eq!(ws.get_all_relays().unwrap(), expected);
        sleep(Duration::from_millis(100));
        ws.set_all_relays(BinaryState::Off).unwrap();
    }

    #[test]
    
    fn test_software_revision() {
        let mut ws = ws();
        assert_eq!(ws.software_revision().unwrap(), "v1.00");
    }

    #[test]
    
    fn test_get_device_address() {
        let mut ws = ws();
        let addr = ws.get_address();
        assert_eq!(addr.unwrap(), 0x01);
    }

    #[test]
    
    fn test_set_device_address() {
        let mut ws = ws();

        assert_eq!(ws.get_address().unwrap(), 0x01);
        assert!(ws.set_address(0x05).is_ok());
        assert_eq!(ws.get_address().unwrap(), 0x05);
        assert!(ws.set_address(0x01).is_ok());
        assert_eq!(ws.get_address().unwrap(), 0x01);
    }
}
