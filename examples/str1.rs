#![allow(non_snake_case)]
use std::time::Duration;

use brewdrivers::controllers::STR1;
use brewdrivers::state::BinaryState;
use brewdrivers::Result;

fn main() -> Result<()> {
    let mut str1 = STR1::connect(0xFE, "/dev/ttyUSB0", 38400, Duration::from_millis(16)).unwrap();
    str1.set_relay(3, BinaryState::On).unwrap();
    str1.list_all_relays().unwrap();
    str1.set_relay(3, BinaryState::Off).unwrap();
    Ok(())
}
