/// A binary state, as used in a relay or similar. This can be 'On' or 'Off'.
///
/// If the `network` feature is enabled, this enum will be serializable with `serde`. If
/// the network component isn't needed, we don't have to compile `serde`, saving some space.
///
/// This enum is mainly here for compatability with the javascript front end.
/// Javascript is pretty fast and loose with it's types, and this ensures we get an explicit
/// 'On' or 'Off' instead of `true`/`false`, `0`/`1`, `null`, etc.
#[cfg_attr(features = "network", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinaryState {
    On,
    Off
}

impl From<bool> for BinaryState {
    /// Converts a `bool` to a `State`
    ///
    /// ```rust
    /// use brewdrivers::controllers::BinaryState;
    ///
    /// assert_eq!(BinaryState::from(true), BinaryState::On);
    /// assert_eq!(BinaryState::from(false),BinaryState::Off);
    /// ```
    fn from(value: bool) -> Self {
        match value {
            true  => BinaryState::On,
            false => BinaryState::Off
        }
    }
}

impl From<u8> for BinaryState {
    /// Converts a u8 to a BinaryState
    /// 
    /// ```
    /// 1 => BinaryState::On
    /// 0 | (anything else) => BinaryState::Off
    /// ```
    fn from(value: u8) -> Self {
        match value {
            1 => Self::On,
            _ => Self::Off
        }
    }
}

impl From<String> for BinaryState {
    /// Same as `From<&str>`
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&str> for BinaryState {
    /// Converts a `&str` to a BinaryState
    /// 
    /// ```
    /// "1" | "on"  | "ON"  | "On"  => BinaryState::On,
    /// "0" | "off" | "OFF" | "Off" => Self::Off
    /// ```
    fn from(value: &str) -> Self {
        match value {
            "1" | "on" | "ON" | "On" => Self::On,
            "0" | "off" | "OFF" | "Off" => Self::Off,
            _ => Self::Off
        }
    }
}

impl Into<bool> for BinaryState {
    fn into(self) -> bool {
        match self {
            BinaryState::On => true,
            BinaryState::Off => false
        }
    }
}

impl Into<u8> for BinaryState {
    fn into(self) -> u8 {
        match self {
            BinaryState::On => 1,
            BinaryState::Off => 0
        }
    }
}

impl Into<String> for BinaryState {
    fn into(self) -> String {
        match self {
            BinaryState::On => String::from("On"),
            BinaryState::Off => String::from("Off")
        }
    }
}

impl Into<&str> for BinaryState {
    fn into(self) -> &'static str {
        match self {
            BinaryState::On => "On",
            BinaryState::Off => "Off"
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