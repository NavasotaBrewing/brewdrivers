// pub mod str1;
pub mod new_str1;
pub mod bytestring;

pub use new_str1::STR1;
pub use bytestring::Bytestring;

#[cfg_attr(features = "network", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum State {
    On,
    Off
}

impl State {
    pub fn from(state: bool) -> State {
        if state {
            return State::On;
        };
        State::Off
    }
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            State::On => write!(f, "on"),
            State::Off => write!(f, "off"),
        }
    }
}
