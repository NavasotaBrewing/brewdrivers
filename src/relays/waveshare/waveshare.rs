use crc::{Crc, CRC_16_MODBUS};
use crate::relays::{Board, BoardError, State};

type Result<T> = std::result::Result<T, BoardError>;

const CRC_MODBUS: Crc<u16> = Crc::<u16>::new(&CRC_16_MODBUS);
const WAVESHARE_BAUD: usize = 9600;

#[derive(Debug)]
pub struct Waveshare(Board);

impl Waveshare {
    pub fn new(address: u8, port_path: &str) -> Result<Waveshare> {
        let port = Board::open_port(port_path, WAVESHARE_BAUD).map_err(|err| BoardError(format!("{}", err)) );

        Ok(
            Waveshare(Board {
                address,
                port: port?,
                baudrate: WAVESHARE_BAUD
            })
        )
    }

    pub fn append_checksum(bytes: &mut Vec<u8>) -> Result<()> {
        let checksum = CRC_MODBUS.checksum(&bytes).to_le_bytes();
        bytes.push(checksum[0]);
        bytes.push(checksum[1]);
        Ok(())
    }

    pub fn set_relay(&mut self, relay_num: u8, state: State) -> Result<()> {
        // Example: 01 05 00 00 FF 00 8C 3A
        // 01   	Device address	    0x00 is broadcast address；0x01-0xFF are device addresses
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
            0x00, relay_num,
        ];

        // Add on state
        match state {
            State::On => bytes.push(0xFF),
            State::Off => bytes.push(0x00)
        };

        // Add on 0x00, because the board needs it I guess
        bytes.push(0x00);

        Waveshare::append_checksum(&mut bytes).unwrap();

        self.0.write_to_device(bytes)?;
        Ok(())
    }


    pub fn get_relay(&mut self, relay_num: u8) -> Result<State> {
        let mut bytes: Vec<u8> = vec![
            self.0.address(),
            // These are all constant, reading all relays status
            0x01,
            0x00, 0xFF,
            0x00, 0x01
        ];

        Waveshare::append_checksum(&mut bytes).unwrap();

        let resp = self.0.write_to_device(bytes)?;

        if let Some(states_number) = resp.get(3) {
            // This is absolutely fucked
            // Get the number as binary, but as a String
            let s = format!("{:b}", states_number);
            // split each character (bit)
            let mut states_binary = s.split("").collect::<Vec<&str>>();
            // remove the first and last because split() includes blank spaces
            states_binary.remove(0);
            states_binary.remove(states_binary.len() - 1);

            // Match the nth character (relay number)
            match states_binary.get(relay_num as usize) {
                Some(&"1") => return Ok(State::On),
                Some(&"0") => return Ok(State::Off),
                _ => return Err(BoardError(format!("Something went wrong with the boards response: {:?}", states_binary)))
            }
        } else {
            return Err(BoardError(format!("Something went wrong, resp: {:?}", resp)));
        }
    }



    pub fn set_all_relays(&mut self, state: State) -> Result<()> {
        let mut bytes: Vec<u8> = vec![
            self.0.address(),
            // These are all constant, reading all relays status
            0x05,
            0x00, 0xFF,
        ];

        match state {
            State::On => {
                bytes.push(0xFF);
                bytes.push(0xFF);
            },
            State::Off => {
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
    use serial_test::serial;
    use std::thread::sleep;
    use std::time::Duration;

    // Helper function
    fn ws() -> Waveshare {
        Waveshare::new(0x01, "/dev/ttyUSB0").unwrap()
    }

    #[test]
    #[serial]
    fn test_connect_to_waveshare() {
        let ws = Waveshare::new(0x01, "/dev/ttyUSB0");
        assert!(ws.is_ok());
    }

    #[test]
    #[serial]
    fn test_crc_16_checksum() {
        let checksum = CRC_MODBUS.checksum(&[0x01, 0x05, 0x00, 0x00, 0xFF, 0x00]);
        assert_eq!(checksum, 0x3A8C);

        // test swapping the bytes
        // from 0x3A8C we want [8C, 3A]
        assert_eq!([0x8C, 0x3A], checksum.to_le_bytes());
    }


    #[test]
    #[serial]
    fn test_write_relay_state() {
        let mut ws = ws();

        assert!(ws.set_relay(0, State::On).is_ok());
        sleep(Duration::from_millis(200));
        assert!(ws.set_relay(0, State::Off).is_ok());
    }

    #[test]
    #[serial]
    fn test_get_relay_status() {
        let mut ws = ws();

        ws.set_relay(0, State::On).unwrap();
        assert_eq!(ws.get_relay(0).unwrap(), State::On);

        ws.set_relay(0, State::Off).unwrap();
        assert_eq!(ws.get_relay(0).unwrap(), State::Off);
    }

    #[test]
    #[serial]
    fn test_write_all_relays() {
        let mut ws = ws();

        ws.set_all_relays(State::On).unwrap();
        for i in 0..8 {
            assert_eq!(ws.get_relay(i).unwrap(), State::On);
        }
        sleep(Duration::from_millis(200));
        ws.set_all_relays(State::Off).unwrap();
    }
}