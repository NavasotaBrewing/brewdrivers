#![allow(non_snake_case)]
use brewdrivers::relays::{State, str1::STR1};



fn main() {
    let mut board = STR1::new(0xFE, "/dev/ttyUSB0", 9600).unwrap();
}
