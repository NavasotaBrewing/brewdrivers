extern crate serialport;
extern crate hex;

pub mod str1;

use str1::{Str1, State::{On, Off}};

pub fn main() {
    let mut s = Str1::new(2);
    s.set_relay(1, On);
    s.set_relay(1, Off);
}
