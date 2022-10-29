//! These traits represent different device types

use async_trait::async_trait;

use crate::drivers::InstrumentError;
use crate::controllers::BinaryState;

type Result<T> = std::result::Result<T, InstrumentError>;

// Used by the Waveshare and STR1 controllers
pub trait RelayBoard<T> {
    fn connect(address: u8, port_path: &str) -> Result<T>;
    fn set_relay(&mut self, relay_num: u8, state: BinaryState) -> Result<()>;
    fn get_relay(&mut self, relay_num: u8) -> Result<BinaryState>;
}

#[async_trait]
pub trait PID<T> {
    async fn connect(address: u8, port_path: &str) -> Result<T>;
    async fn get_pv(&mut self) -> Result<f64>;
    async fn get_sv(&mut self) -> Result<f64>;
    async fn set_sv(&mut self, new_sv: f64) -> Result<()>;
    async fn is_running(&mut self) -> Result<bool>;
    async fn run(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    async fn connected(&mut self) -> Result<()>;
}
