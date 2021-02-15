use brewdrivers::omega::{Degree, CN7500};

#[tokio::main]
async fn main() {
    let mut cn = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await;
    cn.set_degrees(Degree::Fahrenheit).await.unwrap();
}