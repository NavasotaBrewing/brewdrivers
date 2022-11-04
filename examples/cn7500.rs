use std::time::Duration;

use brewdrivers::controllers::CN7500;

#[tokio::main]
async fn main() {
    let mut cn = CN7500::connect(0x16, "/dev/ttyUSB0", 19200, Duration::from_millis(100)).await.expect("Couldn't get device");

    match cn.get_pv().await {
        Ok(pv) => println!("CN7500 PV: {}", pv),
        Err(e) => eprintln!("Error! {}", e)
    }

    match cn.software_revision().await {
        Ok(s) => println!("Software revision: {:?}", s),
        Err(e) => eprintln!("{}", e)
    }
    
}
