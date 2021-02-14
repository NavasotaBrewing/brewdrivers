pub mod str1;
pub mod bytestring;

pub use str1::STR1;
pub use bytestring::Bytestring;

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


pub trait Board {
    fn with_address(addr: u8) -> Self;
    fn set_relay(&mut self, relay_num: u8, new_state: State);
    fn get_relay(&mut self, relay_num: u8) -> State;
}
