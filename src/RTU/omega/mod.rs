use futures::future::Future;
use tokio_core::reactor::Core;
use tokio_serial::{Serial, SerialPortSettings};
use tokio_modbus::prelude::*;

pub mod cn7500;
pub use cn7500::CN7500;


#[derive(Debug)]
pub struct Instrument {
    addr: u8,
    tty_addr: String,
    baudrate: u32
}

impl Instrument {
    pub fn new(addr: u8, tty_addr: &str, baudrate: u32) -> Instrument {
        Instrument {
            addr,
            tty_addr: String::from(tty_addr),
            baudrate
        }
    }

    pub fn write_coil(&self, register: u16, data: bool) {
        // TODO: Add a timeout on wrong addr
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let slave = Slave(self.addr);

        let mut settings = SerialPortSettings::default();
        settings.baud_rate = self.baudrate;
        let port = Serial::from_path_with_handle(self.tty_addr.as_str(), &settings, &handle.new_tokio_handle()).unwrap();

        let task = rtu::connect_slave(&handle, port, slave).and_then(|ctx| {
            ctx.write_single_coil(register, data)
        });

        core.run(task).expect("Error running task write_single_coil");
    }

    pub fn read_coils<F>(&self, coil: u16, cnt: u16, handler: F)
    where F: FnOnce(Vec<bool>) {
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
                .read_coils(coil, cnt)
                // Then pass it to the handler
                .and_then(move |rsp| {
                    handler(rsp);
                    Ok(())
                })
        });

        core.run(task).expect("Error running task read_coils");
    }

    pub fn write_register(&self, register: u16, data: u16) {
        // TODO: Add a timeout on wrong addr
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let slave = Slave(self.addr);

        let mut settings = SerialPortSettings::default();
        settings.baud_rate = self.baudrate;
        let port = Serial::from_path_with_handle(self.tty_addr.as_str(), &settings, &handle.new_tokio_handle()).unwrap();

        let task = rtu::connect_slave(&handle, port, slave).and_then(|ctx| {
            ctx.write_single_register(register, data)
        });

        core.run(task).expect("Error running task write_single_register");
    }

    pub fn read_registers<F>(&self, register: u16, cnt: u16, handler: F)
    where F: FnOnce(Vec<u16>) {
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
                .and_then(move |rsp| {
                    handler(rsp);
                    Ok(())
                })
        });

        core.run(task).expect("Error running task read_holding_registers");
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_registers() {
        let inst = Instrument::new(0x16, "/dev/ttyAMA0", 19200);
        inst.read_registers(0x1000, 1, |response| {
            assert!(response[0] > 0);
        });
        inst.read_registers(0x1001, 1, |response| {
            assert!(response[0] > 0);
        });
    }

    #[test]
    fn test_write_register() {
        let inst = Instrument::new(0x16, "/dev/ttyAMA0", 19200);
        inst.write_register(0x1001, 1000);
        inst.read_registers(0x1001, 1, |response| {
            assert_eq!(response[0], 1000);
        });

        inst.write_register(0x1001, 1300);
        inst.read_registers(0x1001, 1, |response| {
            assert_eq!(response[0], 1300);
        });
    }

    #[test]
    fn test_write_coil() {
        let inst = Instrument::new(0x16, "/dev/ttyAMA0", 19200);
        inst.write_coil(0x0814, true);
        // Assert it's on
        inst.write_coil(0x0814, false);
    }
}
