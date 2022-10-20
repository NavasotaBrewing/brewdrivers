//! A controller is a specific implementation of driver, made for one
//! specific instrument. 
use serde::{Serialize, Deserialize};

pub mod cn7500;
pub mod str1;
pub mod waveshare;
pub mod states;
pub mod device_types;

pub use cn7500::CN7500;
pub use str1::STR1;
pub use waveshare::Waveshare;
pub use states::{BinaryState, AnyState};
pub use device_types::{RelayBoard, PID};


/// These are the types of controllers that the BCS supports. This enum should reflect every
/// controller in `brewdrivers::controllers`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Controller {
    /// An STR1XX relay board. They come in STR116 (16-relay) or STR108 (8-relay).
    /// The driver is the same either way.
    STR1,
    /// An OMEGA Engineering PID. We use the CN7500, and haven't yet tested on others.
    CN7500,
    // The Waveshare relay board, similar in usage to the STR1
    Waveshare
}

impl std::fmt::Display for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CN7500 => write!(f, "CN7500"),
            Self::STR1 => write!(f, "STR1"),
            Self::Waveshare => write!(f, "Waveshare")
        }
    }
}

impl<T: AsRef<str>> From<T> for Controller {
    fn from(value: T) -> Self {
        match value.as_ref() {
            "STR1" => Self::STR1,
            "CN7500" => Self::CN7500,
            "Waveshare" => Self::Waveshare,
            _ => panic!("`{}` is not a valid controller name", value.as_ref())
        }
    }
}
