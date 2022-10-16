//! A controller is a specific implementation of driver, made for one
//! specific instrument. 

pub mod cn7500;
pub mod str1;
pub mod waveshare;
pub mod states;

pub use cn7500::CN7500;
pub use str1::STR1;
pub use waveshare::Waveshare;
pub use states::BinaryState;