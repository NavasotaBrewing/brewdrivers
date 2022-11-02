//! A controller is a specific implementation of driver, made for one
//! specific instrument. This module also includes pieces of data like state enums 
//! that are used by the controller and above layer but not the driver layer.
use serde::{Serialize, Deserialize};

pub mod cn7500;
pub mod str1;
pub mod waveshare;
pub mod wavesharev2;

pub use cn7500::CN7500;
pub use str1::STR1;
pub use waveshare::Waveshare;
pub use wavesharev2::WaveshareV2;
pub use crate::state::BinaryState;

/// These are the types of controllers that the BCS supports. This enum should reflect every
/// controller in `brewdrivers::controllers`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Controller {
    /// An STR1XX relay board. They come in STR116 (16-relay) or STR108 (8-relay).
    /// The driver is the same either way.
    STR1,
    /// An OMEGA Engineering PID. We use the CN7500, and haven't yet tested on others.
    CN7500,
    /// The Waveshare relay board, similar in usage to the STR1
    Waveshare,
    /// Same as `Waveshare`, but software version 2.00
    WaveshareV2
}

impl std::fmt::Display for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CN7500 => write!(f, "CN7500"),
            Self::STR1 => write!(f, "STR1"),
            Self::Waveshare => write!(f, "Waveshare"),
            Self::WaveshareV2 => write!(f, "WaveshareV2")
        }
    }
}

impl<T: AsRef<str>> From<T> for Controller {
    fn from(value: T) -> Self {
        match value.as_ref() {
            "STR1" => Self::STR1,
            "CN7500" => Self::CN7500,
            "Waveshare" => Self::Waveshare,
            "WaveshareV2" => Self::WaveshareV2,
            _ => panic!("`{}` is not a valid controller name", value.as_ref())
        }
    }
}
