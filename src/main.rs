use brewdrivers::RTU::omega::Instrument;

use std::thread::sleep;
use std::time::Duration;

fn main() {
    // brewdrivers::cli::parse_args();

    let instr = Instrument::new(0x16, "/dev/ttyAMA0", 19200);
    instr.read_coils(0x0814, 1, |response| {
        println!("{:?}", response);
    });
}
