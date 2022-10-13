use std::error::Error;

use brewdrivers::{device_pool::{DevicePool, Device}, omega::CN7500, relays::STR1};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let str1 = STR1::connect(254, "/dev/ttyUSB0").unwrap();
    let cn7500 = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();

    let mut pool = DevicePool::create();
    pool.add("str1", Device::STR1(str1));
    pool.add("cn7500", Device::CN7500(cn7500));

    if let Device::CN7500(device) = pool.device("cn7500").unwrap() {
        assert!(device.set_sv(150.0).await.is_ok());
        assert_eq!(device.get_sv().await.unwrap(), 150.0);
    } else {
        assert!(false);
    }

    Ok(())
}
