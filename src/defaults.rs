//! Default values for things

/// Default configuration file
///
/// You are strongly encouraged to use this file instead of any others
pub fn config_file() -> &'static str {
    "/etc/NavasotaBrewing/rtu_conf.yaml"
}

pub fn rules_file() -> &'static str {
    "/etc/NavasotaBrewing/rules.yaml"
}

pub fn conditions_file() -> &'static str {
    "/etc/NavasotaBrewing/conditions.yaml"
}

pub fn default_command_retries() -> u8 {
    1
}

pub fn default_retry_delay() -> u64 {
    150
}

pub fn default_condition_margin_above() -> f64 {
    3.0
}

pub fn default_condition_margin_below() -> f64 {
    0.0
}
