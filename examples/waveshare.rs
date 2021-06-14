use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use brewdrivers::relays::*;


fn main() {
    let mut board = STR1::new(1, "/dev/ttyUSB0", 9600).expect("Couldn't build a new device");

    let cmd: Vec<u8> = vec![1, 5, 0, 0, 255, 0, 140, 58];

    board.port.write_all(&cmd).unwrap();

    sleep(Duration::from_millis(500));

    let cmd2: Vec<u8> = vec![1, 5, 0, 0, 0, 0, 205, 202];

    board.port.write_all(&cmd2).unwrap();
}