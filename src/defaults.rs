//! Default values for things

/// Default configuration file
///
/// You are strongly encouraged to use this file instead of any others
pub fn config_file() -> &'static str {
    "/etc/NavasotaBrewing/rtu_conf.yaml"
}

/// Testing configuration file
pub fn test_config_file() -> &'static str {
    "/etc/NavasotaBrewing/test_conf.yaml"
}
