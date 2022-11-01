use std::str::FromStr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A generalized state that is attached to all `Device`s
/// 
/// Note that each controller uses a different set of these values.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct DeviceState {
    pub relay_state: Option<BinaryState>,
    pub pv: Option<f64>,
    pub sv: Option<f64>,
}

impl Default for DeviceState {
    fn default() -> Self {
        Self { relay_state: Default::default(), pv: Default::default(), sv: Default::default() }
    }
}

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Couldn't deserialize `{0}` into state variant")]
    Deserialize(String),
    #[error("Bad state values: {0:?}")]
    BadValue(DeviceState),
    #[error("State found to be null")]
    NullState
}

/// A binary state, as used in a relay or similar. This can be 'On' or 'Off'.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum BinaryState {
    On,
    Off
}

impl FromStr for BinaryState {
    type Err = StateError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "On" | "ON" | "on" => Ok(BinaryState::On),
            "Off" | "OFF" | "off" => Ok(BinaryState::Off),
            _ => Err(StateError::Deserialize(s.to_string()))
        }
    }
}

impl std::fmt::Display for BinaryState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BinaryState::On => write!(f, "On"),
            BinaryState::Off => write!(f, "Off"),
        }
    }
}

impl From<bool> for BinaryState {
    fn from(value: bool) -> Self {
        match value {
            true => BinaryState::On,
            false => BinaryState::Off
        }
    }
}

impl Default for BinaryState {
    fn default() -> Self { BinaryState::Off }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_state() {
        // From string
        assert_eq!("On".parse::<BinaryState>().unwrap(), BinaryState::On);
        assert_eq!("Off".parse::<BinaryState>().unwrap(), BinaryState::Off);
        assert!("145".parse::<BinaryState>().is_err());

        // From yaml string
        assert_eq!(serde_yaml::from_str::<BinaryState>("On").unwrap(), BinaryState::On);
        assert_eq!(serde_yaml::from_str::<BinaryState>("Off").unwrap(), BinaryState::Off);

        // To yaml string
        assert_eq!(serde_yaml::to_string(&BinaryState::On).unwrap().trim(), "On");
        assert_eq!(serde_yaml::to_string(&BinaryState::Off).unwrap().trim(), "Off");
    }
}