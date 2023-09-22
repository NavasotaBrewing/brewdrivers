use async_trait::async_trait;

pub mod conditions;
pub mod rtu;
pub mod rules;

pub use rtu::connection::Connection;
pub use rtu::device::Device;
pub use rtu::RTU;

use crate::Result;

/// An abstraction of a field device that can be polled and set
///
/// it is passed a `Device`, which contains connection details. Any controller that wants
/// to be used as a device in the system must implement this.
#[async_trait]
pub trait SCADADevice {
    async fn update(device: &mut Device) -> Result<()>;
    async fn enact(device: &mut Device) -> Result<()>;
}
