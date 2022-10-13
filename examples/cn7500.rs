use brewdrivers::controllers::CN7500;

#[tokio::main]
async fn main() {
    let mut cn = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await.expect("Couldn't get device");
    // cn.set_degrees(Degree::Fahrenheit).await.unwrap();
    if let Ok(pv) = cn.get_pv().await {
        println!("CN7500 PV: {}", pv);
    } else {
        println!("Couldn't get PV from device, check the connection details!");
    }

}
