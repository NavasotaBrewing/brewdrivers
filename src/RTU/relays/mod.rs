pub mod str1;
pub mod bytestring;

pub use str1::STR1;
pub use bytestring::Bytestring;

use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
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

    // pub fn to_u8(&self) -> u8 {
    //     match self {
    //         State::On => return 1,
    //         State::Off => return 0,
    //     }
    // }
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
