/// A controller is a specific implementation of driver, made for one
/// specific instrument. 


pub mod cn7500;
pub mod str1;
pub mod waveshare;
pub mod controller_pool;

pub use cn7500::instrument::CN7500;
pub use str1::STR1;
pub use waveshare::Waveshare;
pub use controller_pool::{ControllerPool, Controller};