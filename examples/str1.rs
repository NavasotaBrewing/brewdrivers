#![allow(non_snake_case)]
use brewdrivers::controllers::RelayBoard;
use brewdrivers::controllers::STR1;
use brewdrivers::state::BinaryState;


fn main() {
    let mut str1 = STR1::connect(0xFE, "/dev/ttyUSB0").unwrap();
    str1.set_relay(3,BinaryState::On).unwrap();
    
    str1.list_all_relays().unwrap();

    str1.set_relay(3,BinaryState::Off).unwrap();
}
