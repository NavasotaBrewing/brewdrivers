use brewdrivers::omega::CN7500;

fn main() {

    let omega = CN7500::new(0x16, "/dev/ttyUSB0", 19200);

    println!("{}", omega.get_pv());
}