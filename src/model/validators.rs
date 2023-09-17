//! Validators for when the RTU is deserialized from the config file
//!
//! These are called on the RTU and return an Err([ModelError](crate::model::ModelError)) if
//! the RTU doesn't pass the test. It's another layer of validation on top of `serde_yaml`. This ensures
//! the values in the RTU are actually correct, not just that it's valid YAML syntax.
//!
//! `serde` takes care of making sure the proper values are present; only values in an `Option<>` or that provide a default can be missing.

use log::{error, info, warn};
use std::collections::HashMap;

use crate::controllers::Controller;

use crate::model::{ModelError, RTU};

// Note that when an RTU generates, if it recieves an error from one of these methods,
// it will call log::error!() on it, then bubble up the error.

pub fn all_validators(rtu: &RTU) -> Result<(), ModelError> {
    devices_have_unique_ids(&rtu)?;
    id_has_no_whitespace(&rtu)?;
    serial_port_is_valid(&rtu)?;
    controller_baudrate_is_valid(&rtu)?;
    timeout_valid(&rtu)?;
    command_retries_valid(&rtu)?;
    retry_delay_valid(&rtu)?;
    Ok(())
}

/// Returns `Ok(())` if each device in the RTU has a unique ID
pub fn devices_have_unique_ids(rtu: &RTU) -> Result<(), ModelError> {
    let mut seen: HashMap<&String, bool> = HashMap::new();
    for device in &rtu.devices {
        if seen.get(&device.id).is_some() {
            return Err(ModelError::validation_error(
                &device.id,
                ("id", device.id.as_str()),
                "devices must have unique IDs across all RTUs",
            ));
        }
        seen.insert(&device.id, true);
    }

    info!("RTU validation check passed: all device IDs are unique");
    Ok(())
}

/// Returns `Ok(())` if the RTU ID and every device ID does not contain whitespace
pub fn id_has_no_whitespace(rtu: &RTU) -> Result<(), ModelError> {
    if rtu.id.contains(char::is_whitespace) {
        return Err(ModelError::validation_error(
            "RTU",
            ("id", &rtu.id),
            "RTU ID cannot contain whitespace",
        ));
    }

    for dev in &rtu.devices {
        if dev.id.contains(char::is_whitespace) {
            return Err(ModelError::validation_error(
                &dev.id,
                ("id", &dev.id),
                "device ID cannot contain whitespace",
            ));
        }
    }

    info!("RTU validation check passed: all ID values provided are valid");
    Ok(())
}

/// This will actually *not* fail if the serial port doesn't exist. Sometimes we disconnect
/// the cable and the port goes away, but it's still valid. Instead, it just checks that it's a
/// valid path in `/dev/`.
///
/// This will however print a `warn!()` statement if the port doesn't exist, if a logger is configured.
/// That will help if the brewer configures the wrong port or there's an electrical error.
pub fn serial_port_is_valid(rtu: &RTU) -> Result<(), ModelError> {
    for dev in &rtu.devices {
        // If they somehow pass an empty string
        // maybe with port: "" in the config file
        if dev.conn.port().len() == 0 {
            return Err(ModelError::validation_error(
                &dev.id,
                ("port", &dev.conn.port()),
                "serial port cannot be empty",
            ));
        }

        let path = &dev.conn.port;

        if !path.starts_with("/dev") {
            return Err(ModelError::validation_error(
                &dev.id,
                ("port", &dev.conn.port()),
                "port path must be in /dev/*",
            ));
        }

        match path.try_exists() {
            Ok(true) => {},
            Ok(false) => warn!("The serial port you configured is valid but does not currently exist. Are your cables plugged in?"),
            Err(e) => {
                error!("The port path you configured is hidden from me (or something similar). I can't determine if it exists or not.");
                error!("I'll let it slide this time since we're not using the serial port at this moment,
                        but maybe double check your serial port configuration");
                error!("{}", e);
            }
        }
    }

    info!("RTU validation check passed: all serial port values provided are valid");
    Ok(())
}

pub fn controller_baudrate_is_valid(rtu: &RTU) -> Result<(), ModelError> {
    use crate::controllers::{
        cn7500::CN7500_BAUDRATES, str1::STR1_BAUDRATES, wavesharev2::WAVESHAREV2_BAUDRATES,
    };
    for dev in &rtu.devices {
        match dev.conn.controller() {
            Controller::STR1 => {
                if !STR1_BAUDRATES.contains(dev.conn.baudrate()) {
                    return Err(ModelError::validation_error(
                        &dev.id,
                        ("baudrate", &format!("{}", dev.conn.baudrate())),
                        "invalid baudrate for STR1 controller",
                    ));
                }
            }
            Controller::CN7500 => {
                if !CN7500_BAUDRATES.contains(dev.conn.baudrate()) {
                    return Err(ModelError::validation_error(
                        &dev.id,
                        ("baudrate", &format!("{}", dev.conn.baudrate())),
                        "invalid baudrate for CN7500 controller",
                    ));
                }
            }
            Controller::Waveshare => {
                // This uses the same baudrates as Version 2
                if !WAVESHAREV2_BAUDRATES.contains(dev.conn.baudrate()) {
                    return Err(ModelError::validation_error(
                        &dev.id,
                        ("baudrate", &format!("{}", dev.conn.baudrate())),
                        "invalid baudrate for WaveshareV2 controller",
                    ));
                }
            }
            Controller::WaveshareV2 => {
                if !WAVESHAREV2_BAUDRATES.contains(dev.conn.baudrate()) {
                    return Err(ModelError::validation_error(
                        &dev.id,
                        ("baudrate", &format!("{}", dev.conn.baudrate())),
                        "invalid baudrate for WaveshareV2 controller",
                    ));
                }
            }
        }
    }

    info!("RTU validation check passed: all controller baudrates provided are valid");
    Ok(())
}

pub fn timeout_valid(rtu: &RTU) -> Result<(), ModelError> {
    for dev in &rtu.devices {
        match dev.conn.timeout {
            // Not allowed
            (0..=15) => {
                return Err(ModelError::validation_error(
                    &dev.id,
                    ("timeout", &format!("{}ms", dev.conn.timeout)),
                    "Timeout cannot be lower than 16 ms",
                ));
            }
            // Allowed, but warn the user
            (16..=35) => {
                // Temporarily disabling these for now
                // warn!(
                //     "Timeout for device `{}` with controller type `{}` is low. This *might* work,
                //     but you may experience device instability, especially under load.
                //     Consider raising your timeout to 30ms or 40ms",
                //     dev.name,
                //     dev.conn.controller()
                // );
                // info!("I've tested the following to be reasonbly stable:");
                // info!("STR1:\t\t17ms at baud 38400");
                // info!("CN7500:\t\t36ms at baud 19200");
                // info!("WaveshareV2:\t41ms at baud 38400");
            }
            _ => {}
        }
    }

    info!("RTU validation check passed: all timeout values provided are valid");
    Ok(())
}

pub fn command_retries_valid(rtu: &RTU) -> Result<(), ModelError> {
    for device in &rtu.devices {
        match device.command_retries {
            0..=5 => {}
            _ => {
                return Err(ModelError::validation_error(
                    &device.id,
                    ("command_retries", &format!("{}", device.command_retries)),
                    "command retries must be in range [0, 6]",
                ))
            }
        }
    }

    info!("RTU validation check passed: all command_retries values provided are valid");
    Ok(())
}

pub fn retry_delay_valid(rtu: &RTU) -> Result<(), ModelError> {
    for device in &rtu.devices {
        if device.retry_delay <= device.conn.timeout || device.retry_delay >= 2000 {
            return Err(ModelError::validation_error(
                &device.id,
                ("retry_delay", &format!("{}", device.retry_delay)),
                &format!(
                    "retry delay for this device must be in the range [{}, 2000] (units in ms)",
                    device.conn.timeout
                ),
            ));
        }
    }

    info!("RTU validation check passed: all relay_delay values provided are valid");
    Ok(())
}

#[cfg(test)]
mod test_validators {
    use super::*;

    use std::{net::Ipv4Addr, str::FromStr};
    use tokio_test::{assert_err, assert_ok};

    use crate::model::{Device, RTU};

    // Just quickly sets up an RTU for testing purposes
    fn rtu(name: &str, id: &str, devices: Vec<Device>) -> RTU {
        RTU {
            name: String::from(name),
            id: String::from(id),
            ip_addr: Ipv4Addr::from_str("0.0.0.0").unwrap(),
            devices,
        }
    }

    // Quickly builds a device for testing purposes
    fn device(input: &str) -> Device {
        serde_yaml::from_str(input).unwrap()
    }

    #[test]
    fn test_devices_have_unique_ids() {
        let devices = vec![
            device(
                r#"
                id: pump
                name: Pump
                conn:
                    port: /dev/ttyUSB0
                    baudrate: 9600
                    timeout: 100
                    controller: STR1
                    controller_addr: 254
                    addr: 0
            "#,
            ),
            device(
                r#"
                id: pump
                name: Other pump with same ID
                conn:
                    port: /dev/ttyUSB0
                    baudrate: 9600
                    timeout: 100
                    controller: STR1
                    controller_addr: 254
                    addr: 1
            "#,
            ),
            device(
                r#"
                id: pump2
                name: Other pump with different ID
                conn:
                    port: /dev/ttyUSB0
                    baudrate: 9600
                    timeout: 100
                    controller: STR1
                    controller_addr: 254
                    addr: 2
            "#,
            ),
        ];

        let mut rtu = rtu("Testing RTU", "testing-id", devices);

        assert_err!(devices_have_unique_ids(&rtu));
        rtu.devices.remove(1);
        assert_ok!(devices_have_unique_ids(&rtu));
    }

    #[test]
    fn test_id_has_no_whitespace() {
        let devices = vec![device(
            r#"
                id: pump id with whitespace
                name: Pump
                conn:
                    port: /dev/ttyUSB0
                    baudrate: 9600
                    timeout: 100
                    controller: STR1
                    controller_addr: 254
                    addr: 2
            "#,
        )];

        let mut rtu = rtu("Testing RTU", "testing id with whitespace", devices);

        assert_err!(id_has_no_whitespace(&rtu));
        rtu.devices[0].id = String::from("something-without-whitespace");
        // Still an error because the RTU id has whitespace
        assert_err!(id_has_no_whitespace(&rtu));
        rtu.id = String::from("no-whitespace");
        assert_ok!(id_has_no_whitespace(&rtu));
    }

    #[test]
    fn test_serial_port_is_valid() {
        // This port may or may not exist, but it's valid
        let devices = vec![device(
            r#"
                id: pump
                name: Pump
                conn:
                    port: /dev/ttyUSB0
                    baudrate: 9600
                    timeout: 100
                    controller: STR1
                    controller_addr: 254
                    addr: 2
            "#,
        )];

        let mut rtu = rtu("testing RTU", "test-id", devices);

        assert_ok!(serial_port_is_valid(&rtu));

        // This port definitely doesn't exist, but it's still valid
        rtu.devices.push(device(
            r#"
            id: pump
            name: Pump
            conn:
                port: /dev/peepee_poopoo
                baudrate: 9600
                timeout: 100
                controller: STR1
                controller_addr: 254
                addr: 2
        "#,
        ));

        assert_ok!(serial_port_is_valid(&rtu));

        // This port is not valid (not in /dev)
        rtu.devices.push(device(
            r#"
            id: pump
            name: Pump
            conn:
                port: /etc/different
                baudrate: 9600
                timeout: 100
                controller: STR1
                controller_addr: 254
                addr: 2
        "#,
        ));

        assert_err!(serial_port_is_valid(&rtu));
    }

    #[test]
    fn test_baudrate() {
        let devices = vec![device(
            r#"
                id: pump
                name: Pump
                conn:
                    port: /dev/ttyUSB0
                    baudrate: 9600
                    timeout: 100
                    controller: STR1
                    controller_addr: 254
                    addr: 2
            "#,
        )];

        let mut rtu = rtu("testing RTU", "test-id", devices);

        assert_ok!(controller_baudrate_is_valid(&rtu));

        rtu.devices.push(device(
            r#"
            id: pump
            name: Pump
            conn:
                port: /dev/ttyUSB0
                baudrate: 9601
                timeout: 100
                controller: STR1
                controller_addr: 254
                addr: 2
        "#,
        ));

        assert_err!(controller_baudrate_is_valid(&rtu));
    }

    #[test]
    fn test_timeout_valid() {
        let devices = vec![device(
            r#"
                id: pump
                name: Pump
                conn:
                    port: /dev/ttyUSB0
                    baudrate: 9600
                    timeout: 100
                    controller: STR1
                    controller_addr: 254
                    addr: 2
            "#,
        )];

        let mut rtu = rtu("testing RTU", "test-id", devices);

        assert_ok!(timeout_valid(&rtu));

        // Timeout less than 20
        rtu.devices.push(device(
            r#"
            id: pump
            name: Pump
            conn:
                port: /dev/ttyUSB0
                baudrate: 9600
                timeout: 15
                controller: STR1
                controller_addr: 254
                addr: 2
        "#,
        ));

        assert_err!(timeout_valid(&rtu));
    }

    #[test]
    fn test_command_retries_valid() {
        let valid_device = device(
            r#"
            id: pump
            name: pump
            command_retries: 0
            conn:
                port: /dev/ttyUSB0
                baudrate: 9600
                timeout: 15
                controller: STR1
                controller_addr: 254
                addr: 2
            "#,
        );

        let invalid_device = device(
            r#"
            id: pump
            name: pump
            command_retries: 6
            conn:
                port: /dev/ttyUSB0
                baudrate: 9600
                timeout: 15
                controller: STR1
                controller_addr: 254
                addr: 2
            "#,
        );

        let undeserializable_device = device(
            r#"
            id: pump
            name: pump
            command_retries: 6
            conn:
                port: /dev/ttyUSB0
                baudrate: 9600
                timeout: 15
                controller: STR1
                controller_addr: 254
                addr: 2
            "#,
        );

        let rtu1 = rtu("Valid RTU", "testing-id", vec![valid_device]);
        assert_ok!(command_retries_valid(&rtu1));

        let rtu2 = rtu("Invalid RTU", "testing-id", vec![invalid_device]);
        assert_err!(command_retries_valid(&rtu2));

        let rtu3 = rtu("Invalid RTU", "testing-id", vec![undeserializable_device]);
        assert_err!(command_retries_valid(&rtu3));
    }

    #[test]
    fn test_retry_delay_valid() {
        let valid_device = device(
            r#"
            id: pump
            name: pump
            retry_delay: 400
            conn:
                port: /dev/ttyUSB0
                baudrate: 9600
                timeout: 15
                controller: STR1
                controller_addr: 254
                addr: 2
            "#,
        );

        let invalid_device = device(
            r#"
            id: pump
            name: pump
            retry_delay: 5000
            conn:
                port: /dev/ttyUSB0
                baudrate: 9600
                timeout: 15
                controller: STR1
                controller_addr: 254
                addr: 2
            "#,
        );

        let rtu1 = rtu("Valid RTU", "testing-id", vec![valid_device]);
        assert_ok!(retry_delay_valid(&rtu1));

        let rtu2 = rtu("Invalid RTU", "testing-id", vec![invalid_device]);
        assert_err!(retry_delay_valid(&rtu2));
    }
}
