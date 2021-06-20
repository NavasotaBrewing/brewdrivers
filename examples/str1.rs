#![allow(non_snake_case)]
use brewdrivers::relays::{STR1, State};


fn main() {
    let mut str1 = STR1::connect(0xFE, "/dev/ttyUSB0").unwrap();
    str1.set_relay(3, State::On).unwrap();
    
    str1.list_all_relays().unwrap();

    str1.set_relay(3, State::Off).unwrap();
}
