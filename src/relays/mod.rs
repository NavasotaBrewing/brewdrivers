//! Drivers for relay boards.
//!
//! Relay boards we support:
//!  * [`STR1XX`](crate::relays::str1)
//!
//! see the [hardware guides](https://github.com/NavasotaBrewing/readme/tree/master/hardware) for more information.

pub mod str1;
pub mod bytestring;

pub use str1::{STR1, STR1Error};
pub use bytestring::Bytestring;

/// The state of a relay. This can be 'On' or 'Off'.
///
/// If the `network` feature is enabled, this enum will be serializable with `serde`.
///
/// This enum is mainly here for compatability with `brewkit`, the javascript front end.
/// Javascript is pretty fast and loose with it's types, and this ensures we get an explicit
/// 'On' or 'Off' instead of `true`/`false`, `0`/`1`, `null`, etc.
#[cfg_attr(features = "network", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub enum State {
    On,
    Off
}

impl State {
    /// Converts a `bool` to a `State`
    ///
    /// ```rust
    /// use brewdrivers::relays::State;
    ///
    /// assert_eq!(State::from(true),  State::On);
    /// assert_eq!(State::from(false), State::Off);
    /// ```
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
