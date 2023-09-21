//! These boards are relatively cheap and (so far) reliable. They can be [found here](https://www.waveshare.com/modbus-rtu-relay.htm).
//! The [operation wiki](https://www.waveshare.com/wiki/Protocol_Manual_of_Modbus_RTU_Relay) explains how to use it, but you probably won't need that.
//!
//! See the `examples/` directory for a complete example of using this board.
use std::time::Duration;

use async_trait::async_trait;
// ext uses
// Used for checksums
use crc::{Crc, CRC_16_MODBUS};
use log::trace;

// internal uses
use crate::drivers::{serial::SerialInstrument, InstrumentError, Result};
use crate::logging_utils::device_trace;
use crate::model::Device;
use crate::state::{BinaryState, StateError};

use crate::model::SCADADevice;

// The baudrates that the Waveshare supports
// Same as version 2
pub const WAVESHARE_BAUDRATES: [usize; 8] =
    [4800, 9600, 19200, 38400, 57600, 115200, 128000, 256000];

// This is the checksum algorithm that the board uses
const CRC_MODBUS: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);

/// A Waveshare board.
#[derive(Debug)]
pub struct Waveshare(SerialInstrument);

#[async_trait]
impl SCADADevice for Waveshare {
    async fn update(device: &mut Device) -> Result<()> {
        device_trace!(device, "updating Waveshare device...");

        let mut board = Self::connect(
            device.conn.controller_addr,
            &device.conn.port(),
            // TODO: read these from the device once it's implemented
            device.conn.baudrate().clone(),
            device.conn.timeout(),
        )?;
        device.state.relay_state = Some(board.get_relay(device.conn.addr)?);

        device_trace!(device, "updated");
        Ok(())
    }

    async fn enact(device: &mut Device) -> Result<()> {
        device_trace!(device, "enacting Waveshare device...");

        let mut board = Self::connect(
            device.conn.controller_addr,
            &device.conn.port(),
            // TODO: read these from the device once it's implemented
            device.conn.baudrate().clone(),
            device.conn.timeout(),
        )?;

        match device.state.relay_state {
            Some(new_state) => board.set_relay(device.conn.addr(), new_state)?,
            None => {
                return Err(InstrumentError::StateError(StateError::BadValue(
                    device.state.clone(),
                )))
            }
        }

        device_trace!(device, "enacted");
        Ok(())
    }
}

impl Waveshare {
    /// Connect to a board at the given address and port. This will fail if the port can't be opened,
    /// or if the board can't be communicated with. This method will poll the board for it's software
    /// version number and fail if it doesn't return one, returning an [`InstrumentError`](crate::drivers::InstrumentError).
    pub fn connect(
        address: u8,
        port_path: &str,
        baudrate: usize,
        timeout: Duration,
    ) -> Result<Self> {
        if !WAVESHARE_BAUDRATES.contains(&baudrate) {
            return Err(InstrumentError::SerialError {
                msg: format!("Invalid baudrate `{baudrate}`"),
                addr: Some(address),
            });
        }

        let mut ws = Self(SerialInstrument::new(
            address, port_path, baudrate, timeout,
        )?);

        ws.connected().map_err(|instr_err| {
            InstrumentError::serialError(
                format!(
                    "Waveshare board connection failed, likely busy. Error: {}",
                    instr_err
                ),
                Some(address),
            )
        })?;
        trace!("[Waveshare addr: {}] connected", address);
        Ok(ws)
    }

    pub fn connected(&mut self) -> Result<()> {
        self.software_revision()?;
        Ok(())
    }

    /// Sets a relay to the given state. See the [`BinaryState`](crate::controllers::BinaryState) enum.
    pub fn set_relay(&mut self, relay_num: u8, state: BinaryState) -> Result<()> {
        // Example: 01 05 00 00 FF 00 8C 3A
        // 01       Device address	    0x00 is broadcast address；0x01-0xFF are device addresses
        // 05       05 Command	        Command for controlling Relay
        // 00 00	Address	            The register address of controlled Relay, 0x00 - 0x0008
        // FF 00	Command	            0xFF00：Open Replay;
        //                              0x0000：Close Relay;
        //                              0x5500：Flip Relay
        // 8C 3A	CRC16	            The CRC checksum of first six bytes.
        trace!(
            "[Waveshare addr: {}] setting relay {} to {}",
            self.0.address(),
            relay_num,
            state
        );
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
    pub fn get_relay(&mut self, relay_num: u8) -> Result<BinaryState> {
        trace!(
            "[Waveshare addr: {}] getting relay {}",
            self.0.address(),
            relay_num
        );
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

    // Calculates the CRC checksum for the data bytes to send to the board
    fn append_checksum(bytes: &mut Vec<u8>) -> Result<()> {
        let checksum = CRC_MODBUS.checksum(&bytes).to_le_bytes();
        bytes.push(checksum[0]);
        bytes.push(checksum[1]);
        Ok(())
    }

    /// Returns a `Vec<BinaryState>` of all 8 relays.
    pub fn get_all_relays(&mut self) -> Result<Vec<BinaryState>> {
        trace!("[Waveshare addr: {}] getting all relays", self.0.address());
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
                        _ => BinaryState::default(),
                    }
                })
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
    pub fn software_revision(&mut self) -> Result<String> {
        let mut bytes: Vec<u8> = vec![self.0.address(), 0x03, 0x80, 0x00, 0x00, 0x01];

        Waveshare::append_checksum(&mut bytes)?;

        let resp = self.0.write_to_device(bytes)?;

        if let Some(&version_num) = resp.get(4) {
            Ok(format!("v{:.2}", (version_num as f64 / 100.0)))
        } else {
            Err(
                InstrumentError::serialError(
                    format!(
                        "The board didn't return it's software revision correctly. Possible connection issue. port: {:?}, response: {:?}",
                        self.0,
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
    pub fn get_address(&mut self) -> Result<u8> {
        trace!("[Waveshare addr: {}] getting address", self.0.address());
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
    pub fn set_address(&mut self, new_addr: u8) -> Result<()> {
        trace!(
            "[Waveshare addr: {}] setting address to {}",
            self.0.address(),
            new_addr
        );
        let mut bytes: Vec<u8> = vec![self.0.address(), 0x06, 0x40, 0x00, 0x00, new_addr];

        Waveshare::append_checksum(&mut bytes)?;

        let _resp = self.0.write_to_device(bytes)?;
        self.0.set_address(new_addr);
        Ok(())
    }

    /// Sets all relays at once to the given state.
    pub fn set_all_relays(&mut self, state: BinaryState) -> Result<()> {
        trace!(
            "[Waveshare addr: {}] setting all relays to {}",
            self.0.address(),
            state
        );
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

/// Creates a controller connection from a Device
impl TryFrom<&Device> for Waveshare {
    type Error = InstrumentError;
    fn try_from(device: &Device) -> std::result::Result<Self, Self::Error> {
        Self::connect(
            device.conn.controller_addr(),
            &device.conn.port(),
            device.conn.baudrate().clone(),
            device.conn.timeout(),
        )
    }
}

// These are disabled because I don't have a waveshare v1 and cargo doesn't
// allow me to skip these, because wavesharev2 is a similar name
#[cfg(False)]
#[cfg(test)]
mod tests {
    use crate::controllers::Controller;

    use super::*;

    use std::thread::sleep;
    use std::time::Duration;

    // Helper function
    fn ws() -> Waveshare {
        let device = crate::tests::test_device_from_type(Controller::Waveshare);
        Waveshare::connect(device.conn.controller_addr(), &device.conn.port()).unwrap()
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
        assert_eq!(ws.software_revision().unwrap(), "v2.00");
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
