//! Provides alternatives to the regular log::info, log::trace, etc. macros.
//! These work the same way, but you can provide a [Device](crate::model::Device) as the first
//! argument and it will print the id and state as a prefix.
//!
//! The state printed will be the current state stored on the Device struct, so be sure to
//! update the device first if you want accurate logging
use crate::controllers::Controller;
use crate::model::Device;

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
            states_string.push_str(&format!(
                "pv: {:?}, sv: {:?}, relay: {:?}",
                device.state.pv, device.state.sv, device.state.relay_state
            ));
        }
        Controller::Waveshare | Controller::WaveshareV2 | Controller::STR1 => {
            states_string.push_str(&format!("relay_state: {:?}", device.state.relay_state))
        }
    }
    return format!("[`{}` -> {}]", device.id, states_string);
}

#[macro_export]
macro_rules! device_trace {
    ($device:expr) => {
        device_trace!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::trace!(
            "{} {}",
            $crate::logging_utils::format_log_prefix(&$device),
            $msg
        );
    };
}

#[macro_export]
macro_rules! device_debug {
    ($device:expr) => {
        device_debug!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::debug!(
            "{} {}",
            $crate::logging_utils::format_log_prefix(&$device),
            $msg
        );
    };
}

#[macro_export]
macro_rules! device_info {
    ($device:expr) => {
        device_info!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::info!(
            "{} {}",
            $crate::logging_utils::format_log_prefix(&$device),
            $msg
        );
    };
}

#[macro_export]
macro_rules! device_warn {
    ($device:expr) => {
        device_warn!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::warn!(
            "{} {}",
            $crate::logging_utils::format_log_prefix(&$device),
            $msg
        );
    };
}

#[macro_export]
macro_rules! device_error {
    ($device:expr) => {
        device_error!($device, "")
    };
    ($device:expr, $msg:expr) => {
        log::error!(
            "{} {}",
            $crate::logging_utils::format_log_prefix(&$device),
            $msg
        );
    };
}

// We technically don't need crate:: for all of these
// except warn, which conflicts with the #[warn] builtin
pub use device_debug;
pub use device_error;
pub use device_info;
pub use device_trace;
pub use device_warn;

