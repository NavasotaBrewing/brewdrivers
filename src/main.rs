use brewdrivers::RTU::omega::cn7500::CN7500;

fn main() {
    let omega = CN7500::new(0xff, "/dev/ttyAMA0", 19200);
    omega.read_register(0x1001, 1, |response| {
        println!("{:?}", response);

        Ok(())
    });
    // brewdrivers::cli::parse_args();
}
