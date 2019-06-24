extern crate serialport;
extern crate hex;

pub mod str1;

use str1::Str1;

pub fn main() {
    let mut s = Str1::new(2);
    println!("{:?}", s.get_relay(1));
}
