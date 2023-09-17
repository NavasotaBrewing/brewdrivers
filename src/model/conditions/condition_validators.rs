use log::info;
use std::collections::HashMap;

use crate::{
    controllers::Controller,
    model::{
        conditions::{ConditionError, ConditionKind},
        RTU,
    },
};

use super::Conditions;

pub fn conditions_have_unique_ids(conditions: &Conditions) -> Result<(), ConditionError> {
    let mut seen: HashMap<&String, bool> = HashMap::new();
    for condition in conditions {
        if seen.get(&condition.id).is_some() {
            return Err(ConditionError::validation_error(
                &condition.id,
                format!("conditions must have unique IDs"),
            ));
        }
        seen.insert(&condition.id, true);
    }

    info!("Condition validation check passed: all condition IDs are unique");
    Ok(())
}

pub fn conditions_have_no_whitespace(conditions: &Conditions) -> Result<(), ConditionError> {
    for cond in conditions {
        if cond.id.contains(char::is_whitespace) {
            return Err(ConditionError::validation_error(
                &cond.id,
                format!("condition ID cannot contain whitespace"),
            ));
        }
    }

    info!("Condition validation check passed: no condition IDs contain whitespace");
    Ok(())
}

pub fn conditions_have_existing_device(conditions: &Conditions) -> Result<(), ConditionError> {
    let rtu = RTU::generate(None).unwrap();

    let device_ids = rtu
        .devices
        .iter()
        .map(|device| device.id.clone())
        .collect::<Vec<String>>();

    for cond_def in conditions {
        if !device_ids.contains(&cond_def.device_id) {
            return Err(
                ConditionError::validation_error(
                    &cond_def.id,
                    format!(
                        "conditions must be attached to a device that exists in the configuration. Could not find `{}`",
                        cond_def.device_id
                    )
                )
            );
        }
    }

    info!("Condition validation check passed: all conditions have an associated device that exists in the configuration");
    Ok(())
}

pub fn conditions_have_correct_device_type(conditions: &Conditions) -> Result<(), ConditionError> {
    let rtu = RTU::generate(None).unwrap();

    for cond in conditions {
        // First, get the device that's attached to it
        let device = rtu
            .devices
            .iter()
            .find(|device| device.id == cond.device_id);

        if device.is_none() {
            return Err(
                ConditionError::validation_error(
                    &cond.id,
                    format!(
                        "Couldn't find device `{}` attached to condition, even though I already checked for it. This shouldn't happen.", cond.device_id
                    )
                )
            );
        }

        let device_type = &device.unwrap().conn.controller;

        // Prebuild the error to keep the code a bit cleaner
        // This error is very complicated
        let error = ConditionError::validation_error(
                        &cond.id,
                        format!(
                            "Condition called '{}' (id `{}`) with kind `{:?}` is not applicable to device called {} (id `{}`) because the device has type `{}`",
                            cond.name,
                            cond.id,
                            cond.kind,
                            device.unwrap().name,
                            device.unwrap().id,
                            device_type
                        )
                    );

        match cond.kind {
            ConditionKind::RelayStateIs => match device_type {
                Controller::STR1 | Controller::Waveshare | Controller::WaveshareV2 => {}
                _ => return Err(error),
            },
            ConditionKind::PVIsAtLeast | ConditionKind::PVIsAround | ConditionKind::PVMeetsSV => {
                match device_type {
                    Controller::CN7500 => {}
                    _ => return Err(error),
                }
            }
        }
    }

    info!("Condition validation check passed: all conditions have the proper device type attached");
    Ok(())
}

#[cfg(test)]
mod tests {
    use tokio_test::{assert_err, assert_ok};

    use crate::model::conditions::Condition;

    use super::*;

    fn condition(input: &str) -> Condition {
        serde_yaml::from_str(input).unwrap()
    }

    #[test]
    fn test_condition_device_exists() {
        let condition_def = condition(&format!(
            r#"
            name: My Condition
            id: my-condition
            condition: RelayStateIs
            device_id: relay0
            state:
                relay_state: On
            "#,
        ));

        assert_ok!(conditions_have_existing_device(&vec![condition_def]));

        let condition_def2 = condition(
            r#"
            name: My Condition
            id: my-condition
            condition: RelayStateIs
            device_id: some-id-that-does-not-exist
            state:
                relay_state: On
            "#,
        );

        assert_err!(conditions_have_existing_device(&vec![condition_def2]));
    }
}
