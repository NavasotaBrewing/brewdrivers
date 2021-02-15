use brewdrivers::modbus::ModbusInstrument;
use tokio_modbus::client::Reader;

#[tokio::main]
async fn main() {
    let mut inst = ModbusInstrument::new(0x16, "/dev/ttyUSB0", 19200).await;
    inst.write_coil(0x0814, true).await.unwrap();


}
