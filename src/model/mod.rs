use crate::drivers::InstrumentError;
use async_trait::async_trait;

pub mod conditions;
pub mod device;
pub mod model_error;
pub mod rtu;
mod validators;

pub use device::Device;
pub use model_error::ModelError;
pub use rtu::RTU;

/// An abstraction of a field device that can be polled and set
///
/// it is passed a `Device`, which contains connection details. Any controller that wants
/// to be used as a device in the system must implement this.
#[async_trait]
pub trait SCADADevice {
    async fn update(device: &mut Device) -> Result<(), InstrumentError>;
    async fn enact(device: &mut Device) -> Result<(), InstrumentError>;
}
