use std::error::Error;
use std::time::Duration;
use std::thread::sleep;

use brewdrivers::controllers::{Waveshare, RelayBoard};
use brewdrivers::state::BinaryState;

fn main() -> Result<(), Box<dyn Error>> {
    let mut ws = Waveshare::connect(0x01, "/dev/ttyUSB0")?;

    // getting the software revision is a smoke test
    println!("Board software revision: {:?}", ws.software_revision());

    // Set a relay on or off
    ws.set_relay(0, BinaryState::On)?;
    ws.set_relay(2, BinaryState::On)?;

    // Get all the relays statuses as a Vec<BinaryState>
    let statuses = ws.get_all_relays()?;
    // just print the statuses
    for i in 0..8 {
        println!("Relay {}: {}", i, statuses[i]);
    }

    // Wait a bit
    sleep(Duration::from_millis(100));
    // Make sure they're all off at the end
    ws.set_all_relays(BinaryState::Off)?;



    // Now let's set the controller number to something else (don't forget it. 0x01 is the default)
    println!("{:X?}", ws.get_address());

    println!("Setting the address to 0x07...");
    ws.set_address(0x07)?;
    println!("Address is now 0x{:02X?}", ws.get_address()?);

    println!("Now setting it back to 0x01");
    ws.set_address(0x01)?;

    Ok(())
}