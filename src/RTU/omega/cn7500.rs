use crate::RTU::omega::Instrument;



pub struct CN7500 {
    pub instrument: Instrument,
    addr: u8,
    port: String,
    baudrate: u32,
}

impl CN7500 {
    pub fn new(addr: u8, port: &str, baudrate: u32) -> CN7500 {
        let instrument = Instrument::new(addr, port, baudrate);
        CN7500 {
            instrument,
            addr,
            port: String::from(port),
            baudrate
        }
    }

    pub fn get_pv(&self) -> f64 {
        // Don't know of a better way :(
        let mut pv: f64 = 0.0;
        self.instrument.read_register(0x1000, 1, |response| {
            pv = (response[0] as f64) / 10.0;
        });
        pv
    }

    pub fn get_sv(&self) -> f64 {
        let mut sv: f64 = 0.0;
        self.instrument.read_register(0x1001, 1, |response| {
            sv = (response[0] as f64) / 10.0;
        });
        sv
    }

    pub fn set_sv(&self, temperature: f64) {
        self.instrument.write_register(0x1001, (temperature * 10.0) as u16);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cn7500() {
        let cn = CN7500::new(0x16, "/dev/ttyAMA0", 19200);
        assert_eq!(cn.addr, 0x16);
        assert_eq!(cn.port, String::from("/dev/ttyAMA0"));
        assert_eq!(cn.baudrate, 19200);
    }

    #[test]
    fn test_set_sv() {
        let cn = CN7500::new(0x16, "/dev/ttyAMA0", 19200);
        cn.set_sv(123.4);
    }

    #[test]
    fn test_get_pv() {
        let cn = CN7500::new(0x16, "/dev/ttyAMA0", 19200);
        assert!(cn.get_pv() > 0.0);
    }

    #[test]
    fn test_get_sv() {
        let cn = CN7500::new(0x16, "/dev/ttyAMA0", 19200);
        cn.set_sv(145.7);
        assert_eq!(cn.get_sv(), 145.7);
    }

    #[test]
    fn test_turn_on_relay() {
        let cn = CN7500::new(0x16, "/dev/ttyAMA0", 19200);
        cn.run();
        assert!(cn.is_running());
        cn.stop();
    }
}
