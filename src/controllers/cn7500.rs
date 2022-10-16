//! An implementation of `ModbusInstrument` for the OMEGA CN7500.
//!
//! The [OMEGA CN7500](https://www.omega.com/en-us/control-monitoring/controllers/pid-controllers/p/CN7200-7500-7600-7800)
//! is a PID that we use to regulate temperatures within the BCS. This module provides a driver implementation for it, based on the
//! [`ModbusInstrument`](crate::drivers::ModbusInstrument) driver.
//! 
//! Note: you can set the temperature units (`F` or `C`) of the board with [`CN7500::set_degrees`](crate::controllers::CN7500::set_degrees). 
//! All units returned from the board or sent to it (when setting the setpoint value) will use the unit that the board is configured to at the time.
use crate::drivers::modbus::ModbusInstrument;
use crate::drivers::Result;

#[derive(Debug, Clone)]
pub enum Degree {
    Fahrenheit,
    Celsius
}


/// A CN7500 PID Controller.
/// 
/// ```rust,no_run
/// use brewdrivers::controllers::CN7500;
/// 
/// #[tokio::main]
/// async fn main() {
///     let mut cn = CN7500::connect(0x16, "/dev/ttyUSB0").await.expect("Couldn't get device");
/// 
///     match cn.get_pv().await {
///         Ok(pv) => println!("CN7500 PV: {}", pv),
///         Err(e) => eprintln!("Error! {}", e)
///     }
/// 
///     cn.set_sv(145.6).await.expect("Couldn't set sv");
///     assert_eq!(145.6, cn.get_sv().await.unwrap());
/// }
/// ```
#[derive(Debug)]
pub struct CN7500(ModbusInstrument);

impl CN7500 {
    /// Connects to a CN7500 board
    /// 
    /// ```rust,no_run
    /// use brewdrivers::controllers::CN7500;
    /// 
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut cn = CN7500::connect(0x16, "/dev/ttyUSB0").await.unwrap();
    /// }
    /// ```
    pub async fn connect(slave_addr: u8, port_path: &str) -> Result<Self> {
        ModbusInstrument::new(slave_addr, port_path, 19200)
            .await
            // Wrap the instrument in a CN7500
            .map(|instr| CN7500(instr))
    }

    /// Sets the setpoint value (target) of the CN7500. Should be a decimal between 1.0-999.0.
    pub async fn set_sv(&mut self, new_sv: f64) -> Result<()> {
        self.0.write_register(0x1001, (new_sv * 10.0) as u16).await
    }

    /// Gets the setpoint value
    pub async fn get_sv(&mut self) -> Result<f64> {
        self.0
            .read_registers(0x1001, 1)
            .await
            .map(|vec| (vec[0] as f64) / 10.0)
    }

    /// Gets the process value
    pub async fn get_pv(&mut self) -> Result<f64> {
        self.0
            .read_registers(0x1000, 1)
            .await
            .map(|vec| (vec[0] as f64) / 10.0)
    }

    /// Returns `Ok(true)` if the relay is activated. The relay may or may not be on if it's activated,
    /// because the PID will control when to feather the relay on or off to control temperature. The relay
    /// will never be on if it's not active (ie. this method returns `Ok(false)`)
    pub async fn is_running(&mut self) -> Result<bool> {
        self.0.read_coils(0x0814, 1).await.map(|vals| vals[0] )
    }

    /// Activates the relay
    pub async fn run(&mut self) -> Result<()> {
        self.0.write_coil(0x0814, true).await
    }

    /// Deactivates the relay
    pub async fn stop(&mut self) -> Result<()> {
        self.0.write_coil(0x0814, false).await
    }

    /// Sets the degree mode of the board to either Fahrenheit or Celsius
    pub async fn set_degrees(&mut self, degree_mode: Degree) -> Result<()> {
        match degree_mode {
            Degree::Celsius => self.0.write_coil(0x0811, true).await,
            Degree::Fahrenheit => self.0.write_coil(0x0811, false).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tokio::test;

    async fn instr() -> CN7500 {
        CN7500::connect(0x16, "/dev/ttyUSB0").await.unwrap()
    }

    #[test]
    #[serial]
    async fn test_new_cn7500() {
        let cn = instr().await;
        assert_eq!(cn.0.port_path, "/dev/ttyUSB0");
    }

    #[test]
    #[serial]
    async fn test_set_sv() {
        let mut cn = instr().await;
        let rsp = cn.set_sv(123.4).await;
        assert!(rsp.is_ok());
    }

    #[test]
    #[serial]
    async fn test_get_pv() {
        let mut cn = instr().await;
        assert!(cn.get_pv().await.unwrap() > 0.0);
    }

    #[test]
    #[serial]
    async fn test_get_sv() {
        let mut cn = instr().await;
        assert!(cn.set_sv(145.7).await.is_ok());
        assert_eq!(cn.get_sv().await.unwrap(), 145.7);
    }

    #[test]
    #[serial]
    async fn test_turn_on_relay() {
        let mut cn = instr().await;
        assert!(cn.run().await.is_ok());
        assert!(cn.is_running().await.unwrap());
        assert!(cn.stop().await.is_ok());
    }
}
