// use brewdrivers::omega::CN7500;

// This isn't working currently
// I'm trying to move away from tokio-modbus

use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::{io::{Read, Write}, time::Duration};

fn sleep(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

fn main() {
    // let omega = CN7500::new(0x16, "/dev/ttyUSB0", 19200);
    // println!("{}", omega.get_pv());

    // format:
    // (device addr) (function code) (register #) (register count) (data) (checksum)

    let mut port = serialport::new("/dev/ttyUSB0", 19_200)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(15))
            .open_native()
            .expect("Couldn't open serial prot");



    let mut data: Vec<u8> = vec![0x16, 0x08, 0x22, 0x33, 0x00, 0x00, 0xBE, 0xB8];


    // let mut checksum: u32 = 0;
    // for byte in data.iter() {
    //     checksum += *byte as u32;
    // }
    // checksum = checksum % 256;
    // data.push(checksum as u8);
            
    println!("Data to write: {:?}", data);
    port.write_all(&data).unwrap();

    sleep(100);

    let mut buffer: Vec<u8> = Vec::new();
    match port.read_to_end(&mut buffer) {
        Ok(_) => {},
        Err(_) => {}
    }

    let chars = buffer.iter().map(|val| *val as char ).collect::<String>();
    let hex = buffer.iter().map(|val| format!("{:0>2x}", val) ).collect::<Vec<String>>();

    println!("Raw:    {:?}", buffer);
    println!("As Hex: {:?}", hex);
    println!("As Chars: {:?}", chars);


}