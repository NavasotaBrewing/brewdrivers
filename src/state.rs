//! Generalize states for controllers
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

/// A process value, alias to `f64`
pub type PV = f64;
/// A setpoint value, alias to `f64`
pub type SV = f64;

// TODO: maybe add an `extras` field here? It could be an Option<HashMap>

/// A generalized state that is attached to all `Device`s
///
/// Note that each controller uses a different set of these values. For example,
/// a relay board uses `relay_state` but won't ever touch `pv` or `sv`.
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct DeviceState {
    pub relay_state: Option<BinaryState>,
    pub pv: Option<PV>,
    pub sv: Option<SV>,
}

impl Default for DeviceState {
    /// Creates a default state. This is used when deserializing an RTU model from the
    /// configuration file. If the user doesn't provide state values in the config file (why would they?),
    /// then this will fill in the defaults.
    ///
    /// ```text
    /// DeviceState {
    ///     relay_state: Some(BinaryState::Off),
    ///     pv: Some(0.0),
    ///     sv: Some(0.0)
    /// }
    /// ```
    fn default() -> Self {
        Self {
            relay_state: Default::default(),
            pv: Default::default(),
            sv: Default::default(),
        }
    }
}

/// A general state error. This is mostly used when a bad state value is passed,
/// or the wrong type of state is given to a device.
#[derive(Debug, Error)]
pub enum StateError {
    #[error("Couldn't deserialize `{0}` into state variant")]
    Deserialize(String),
    #[error("Bad state values: {0:?}")]
    BadValue(DeviceState),
    #[error("State found to be null")]
    NullState,
}

/// A binary state, as used in a relay or similar. This can be 'On' or 'Off'.
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum BinaryState {
    On,
    Off,
}

impl FromStr for BinaryState {
    type Err = StateError;
    /// Converts from a string to a BinaryState
    ///
    /// ```rust
    /// use brewdrivers::state::BinaryState;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(BinaryState::from_str("on").unwrap(), BinaryState::On);
    /// assert_eq!(BinaryState::from_str("ON").unwrap(), BinaryState::On);
    /// assert_eq!(BinaryState::from_str("On").unwrap(), BinaryState::On);
    /// assert_eq!(BinaryState::from_str("off").unwrap(), BinaryState::Off);
    /// assert_eq!(BinaryState::from_str("OFF").unwrap(), BinaryState::Off);
    /// assert_eq!(BinaryState::from_str("Off").unwrap(), BinaryState::Off);
    ///
    /// // This doesn't work
    /// // we have to differentiate between stepped states and binary states
    /// assert!(BinaryState::from_str("1").is_err());
    /// assert!(BinaryState::from_str("0").is_err());
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "On" | "ON" | "on" => Ok(BinaryState::On),
            "Off" | "OFF" | "off" => Ok(BinaryState::Off),
            _ => Err(StateError::Deserialize(s.to_string())),
        }
    }
}

impl std::fmt::Display for BinaryState {
    /// ```rust
    /// # use brewdrivers::state::BinaryState;
    /// assert_eq!("On", format!("{}", BinaryState::On));
    /// assert_eq!("Off", format!("{}", BinaryState::Off));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BinaryState::On => write!(f, "On"),
            BinaryState::Off => write!(f, "Off"),
        }
    }
}

impl From<bool> for BinaryState {
    /// Converts from `bool` to `BinaryState`
    ///
    /// ```rust
    /// # use brewdrivers::state::BinaryState;
    /// assert_eq!(BinaryState::from(true), BinaryState::On);
    /// assert_eq!(BinaryState::from(false), BinaryState::Off);
    /// ```
    fn from(value: bool) -> Self {
        match value {
            true => BinaryState::On,
            false => BinaryState::Off,
        }
    }
}

impl Default for BinaryState {
    /// Defaults to `BinaryState::Off`
    fn default() -> Self {
        BinaryState::Off
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
        assert_eq!(
            serde_yaml::from_str::<BinaryState>("On").unwrap(),
            BinaryState::On
        );
        assert_eq!(
            serde_yaml::from_str::<BinaryState>("Off").unwrap(),
            BinaryState::Off
        );

        // To yaml string
        assert_eq!(
            serde_yaml::to_string(&BinaryState::On).unwrap().trim(),
            "On"
        );
        assert_eq!(
            serde_yaml::to_string(&BinaryState::Off).unwrap().trim(),
            "Off"
        );
    }
}
