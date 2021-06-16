use std::error::Error;
use std::time::Duration;
use std::thread::sleep;
use brewdrivers::relays::{Waveshare, State};

fn main() -> Result<(), Box<dyn Error>> {
    let mut ws = Waveshare::connect(0x01, "/dev/ttyUSB0")?;

    ws.set_relay(0, State::On)?;
    ws.set_relay(2, State::On)?;

    let statuses = ws.get_all_relays()?;
    for i in 0..8 {
        println!("Relay {}: {}", i, statuses[i]);
    }

    sleep(Duration::from_millis(100));

    sleep(Duration::from_millis(100));
    ws.set_all_relays(State::Off)?;


    Ok(())
}