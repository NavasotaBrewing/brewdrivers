use brewdrivers::controllers::{CN7500, PID};

#[tokio::main]
async fn main() {
    let mut cn = CN7500::connect(0x16, "/dev/ttyUSB0").await.expect("Couldn't get device");

    match cn.get_pv().await {
        Ok(pv) => println!("CN7500 PV: {}", pv),
        Err(e) => eprintln!("Error! {}", e)
    }
    
}
