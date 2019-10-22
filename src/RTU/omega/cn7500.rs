use futures::future::Future;
use tokio_core::reactor::{Core, Handle};
use tokio_serial::{Serial, SerialPortSettings};
use tokio_modbus::prelude::*;


pub struct CN7500 {
    addr: u8,
    tty_addr: String,
    baudrate: u32
}

impl CN7500 {

    pub fn new(addr: u8, tty_addr: &str, baudrate: u32) -> CN7500 {
        CN7500 {
            addr,
            tty_addr: String::from(tty_addr),
            baudrate
        }
    }

    pub fn read_register<F>(&self, register: u16, cnt: u16, handler: F)
    where F: FnOnce(Vec<u16>) -> Result<(), std::io::Error> {
        // TODO: Add a timeout on wrong addr
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let slave = Slave(self.addr);

        let mut settings = SerialPortSettings::default();
        settings.baud_rate = self.baudrate;
        let port = Serial::from_path_with_handle(self.tty_addr.as_str(), &settings, &handle.new_tokio_handle()).unwrap();

        let task = rtu::connect_slave(&handle, port, slave).and_then(|ctx| {
            ctx
                // Read sensor value
                .read_holding_registers(register, cnt)
                // Then pass it to the handler
                .and_then(move |rsp| { handler(rsp) })
        });

        core.run(task).unwrap();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_pv() {
        let o = CN7500::new(0x16, "/dev/ttyAMA0", 19200);
        o.read_register(0x1000, 1, |rsp| {
            println!("{:?}", rsp);
            assert!(rsp[0] > 0);
            Ok(())
        });
    }
}
