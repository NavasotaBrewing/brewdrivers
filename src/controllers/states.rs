use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Couldn't deserialize `{0}` into state variant")]
    Deserialize(String)
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

impl Default for BinaryState {
    fn default() -> Self { BinaryState::Off }
}


#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnyState {
    /// The string representation of this is "on" or "off", not 0 or 1
    BinaryState(BinaryState),
    /// 0-255
    SteppedState(u8)
}

impl FromStr for AnyState {
    type Err = StateError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to get a binary state
        if let Ok(binary) = s.parse::<BinaryState>() {
            return Ok(Self::BinaryState(binary));
        }

        // Try to get a stepped state
        if let Ok(stepped) = s.parse::<u8>() {
            return Ok(Self::SteppedState(stepped));
        }

        // if none of those are good, return an error
        Err(StateError::Deserialize(s.to_string()))
    }
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

    #[test]
    fn test_any_state() {
        let on = AnyState::BinaryState(BinaryState::On);
        let off = AnyState::BinaryState(BinaryState::Off);
        let stepped = AnyState::SteppedState(45);

        // From str
        assert_eq!("On".parse::<AnyState>().unwrap(), on);
        assert_eq!("Off".parse::<AnyState>().unwrap(), off);
        assert_eq!("45".parse::<AnyState>().unwrap(), stepped);

        // Yaml to string
        assert_eq!(serde_yaml::to_string(&on).unwrap().trim(), "On");
        assert_eq!(serde_yaml::to_string(&off).unwrap().trim(), "Off");
        assert_eq!(serde_yaml::to_string(&stepped).unwrap().trim(), "45");

        // Yaml from str
        assert_eq!(serde_yaml::from_str::<AnyState>("On").unwrap(), on);
        assert_eq!(serde_yaml::from_str::<AnyState>("Off").unwrap(), off);
        assert_eq!(serde_yaml::from_str::<AnyState>("45").unwrap(), stepped);
        assert_eq!(
            serde_yaml::from_str::<AnyState>("0").unwrap(),
            AnyState::SteppedState(0)
        );
    }


}