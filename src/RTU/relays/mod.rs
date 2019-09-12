pub mod str1;
pub mod bytestring;

pub use str1::STR1;
pub use bytestring::Bytestring;

use serde::{Serialize, Deserialize, Deserializer, Serializer};

#[derive(Debug, PartialEq)]
pub enum State {
    On,
    Off
}

impl Serialize for State {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            State::On => serializer.serialize_unit_variant("on", 0, "On"),
            State::Off => serializer.serialize_unit_variant("off", 1, "Off"),
        }
    }
}

impl<'de> Deserialize<'de> for State {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "on" => State::On,
            "off" => State::Off,
            _ => State::Off,
        })
    }
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
