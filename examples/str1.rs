#![allow(non_snake_case)]
use brewdrivers::relays::{State, str1::STR1};


fn main() {
    let mut board = STR1::new(0xFE, "/dev/ttyUSB0", 9600).unwrap();

    // Make sure we're connected
    assert!(board.connected());

    // Get the state of a relay
    assert_eq!(board.get_relay(0), State::Off);

    // Turn a relay on
    board.set_relay(0, State::On);
    assert_eq!(board.get_relay(0), State::On);


    // Set the controller number on the board
    board.set_controller_num(0x45);
    // I want to keep it as 0xFE (default)
    board.set_controller_num(0xFE);

    // Get the amount of relays on the board
    println!("{:?}", board.relay_count()) // Some(8)
}
