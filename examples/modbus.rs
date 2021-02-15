use brewdrivers::modbus::ModbusInstrument;

#[tokio::main]
async fn main() {
    // Testing on my OMEGA CN7500 PID
    let mut inst = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await;

    // Read SV
    let pv1 = inst.read_registers(0x1001, 1).await.unwrap();
    println!("pv is now {:?}", pv1);
    
    // Set SV
    inst.write_register(0x1001, 1300).await.unwrap();
    
    // Read SV again
    let pv2 = inst.read_registers(0x1001, 1).await.unwrap();
    println!("pv is now {:?}", pv2);
    
    // Set PID to STOP
    inst.write_coil(0x0814, false).await.unwrap();
    // Read run/stop value
    let coil = inst.read_coils(0x0814, 1).await.unwrap();
    println!("Coil 0x0814 is {:?}", coil);

}
