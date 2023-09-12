use crate::model::Device;
use crate::controllers::Controller;

/// Creates a string prefix to add to the log message containing the device id and states.
/// 
/// The `brewdrivers::logging_utils` overloads the default log::* macros and adds this prefix to them.
/// Ex:
/// ```text
/// [2023-09-12T20:09:29Z TRACE logging_test] [`pump` -> relay_state: Some(true)] Device said hello!
/// ```
pub fn format_log_prefix(device: &Device) -> String {
    let mut states_string = String::new();
    match device.conn.controller {
        Controller::CN7500 => {
            states_string.push_str(&format!("pv: {:?}, sv: {:?}, relay: {:?}", device.state.pv, device.state.sv, device.state.relay_state));
        }
        Controller::Waveshare
        | Controller::WaveshareV2
        | Controller::STR1 => {
            states_string.push_str(&format!(
                "relay_state: {:?}", device.state.relay_state
            ))
        }
    }
    return format!("[`{}` -> {}]", device.id, states_string);
}

#[macro_export]
macro_rules! trace {
    ($device:expr) => {
        trace!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::trace!("{} {}", $crate::logging_utils::format_log_prefix(&$device), $msg);
    };
}

#[macro_export]
macro_rules! debug {
    ($device:expr) => {
        debug!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::debug!("{} {}", $crate::logging_utils::format_log_prefix(&$device), $msg);
    };
}

#[macro_export]
macro_rules! info {
    ($device:expr) => {
        info!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::info!("{} {}", $crate::logging_utils::format_log_prefix(&$device), $msg);
    };
}

#[macro_export]
macro_rules! warn {
    ($device:expr) => {
        warn!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::warn!("{} {}", $crate::logging_utils::format_log_prefix(&$device), $msg);
    };
}

#[macro_export]
macro_rules! error {
    ($device:expr) => {
        error!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::error!("{} {}", $crate::logging_utils::format_log_prefix(&$device), $msg);
    };
}

// We technically don't need crate:: for all of these
// except warn, which conflicts with the #[warn] builtin
pub use crate::trace;
pub use crate::debug;
pub use crate::info;
pub use crate::warn;
pub use crate::error;