use std::error::Error;

use brewdrivers::{device_pool::DevicePool, relays::STR1, omega::CN7500};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // let str1 = STR1::connect(254, "/dev/ttyUSB0");
    // let cn7500 = CN7500::new(0x16, "dev/ttyUSB0", 19200);

    // let mut pool = DevicePool::create();
    // pool.add(&str1);
    // pool.add(&cn7500);


    Ok(())
}