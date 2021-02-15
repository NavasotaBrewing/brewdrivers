#![allow(non_snake_case)]
use brewdrivers::relays::{new_str1::STR1, State};


fn main() {
    let mut board = STR1::new(0xFE, "/dev/ttyUSB0", 9600);

    // Set 2 relays on
    board.set_relay(0, State::On);
    board.set_relay(5, State::On);

    board.list_all_relays();
    // You can also get a single relay State with
    // board.get_relay(i)

    // Turn them back off
    board.set_relay(0, State::Off);
    board.set_relay(5, State::Off);
}
