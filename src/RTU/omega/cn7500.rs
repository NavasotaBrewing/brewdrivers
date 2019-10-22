use futures::future::Future;
use tokio_core::reactor::{Core, Handle};
use tokio_serial::{Serial, SerialPortSettings};
use tokio_modbus::prelude::*;


struct Omega {
    addr: u8,
    tty_addr: String,
    baudrate: u32
}

impl Omega {

    pub fn new(addr: u8, tty_addr: &str, baudrate: u32) -> Omega {
        Omega {
            addr,
            tty_addr: String::from(tty_addr),
            baudrate
        }
    }

    pub fn run<F>(&self, register: u16, cnt: u16, handler: F)
    where F: FnOnce(Vec<u16>) -> Result<(), std::io::Error> {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let slave = Slave(self.addr);

        let mut settings = SerialPortSettings::default();
        settings.baud_rate = self.baudrate;
        let port = Serial::from_path_with_handle(self.tty_addr.as_str(), &settings, &handle.new_tokio_handle()).unwrap();

        let task = rtu::connect_slave(&handle, port, slave).and_then(|ctx| {
            println!("Reading a sensor value");
            ctx
                .read_holding_registers(register, cnt)
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
        let o = Omega::new(0x16, "/dev/ttyAMA0", 19200);
        o.run(0x1000, 1, |rsp| {
            println!("{:?}", rsp);
            assert!(rsp[0] > 0);
            Ok(())
        });
    }
}
