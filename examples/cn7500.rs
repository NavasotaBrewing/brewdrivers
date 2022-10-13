use brewdrivers::omega::CN7500;

#[tokio::main]
async fn main() {
    let mut cn = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await.unwrap();
    // cn.set_degrees(Degree::Fahrenheit).await.unwrap();
    if let Ok(pv) = cn.get_pv().await {
        println!("CN7500 PV: {}", pv);
    } else {
        println!("Couldn't get PV from device, check the connection details!");
    }

    // We could try to connect again, but our old connection is still active (cn)
    // Device is busy
    assert!(
        CN7500::new(0x16, "/dev/ttyUSB0", 19200)
            .await
            .is_err()
    );

    // Dropping the old connection will allow us to reconnect
    std::mem::drop(cn);

    let another = CN7500::new(0x16, "/dev/ttyUSB0", 19200).await;
    assert!(another.is_ok());
    println!("Another PV: {}", another.unwrap().get_pv().await.unwrap());
    
}
