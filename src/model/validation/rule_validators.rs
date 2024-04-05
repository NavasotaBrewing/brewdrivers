use std::collections::HashSet;

use log::{debug, trace, warn};

use crate::{
    error::Error,
    model::{
        conditions::ConditionCollection,
        rules::{Rule, RuleSet},
        RTU,
    },
    Result,
};

fn fail(rule_id: &str, why: &str) -> Result<()> {
    Err(Error::ValidationError(format!(
        "rule validation failed on component `{rule_id}` because {why}",
    )))
}

pub fn all(
    rules: &RuleSet,
    conditions: &ConditionCollection,
    rtu: &mut RTU,
) -> std::result::Result<(), Vec<Error>> {
    let mut errors: Vec<Error> = Vec::new();

    if let Err(e) = all_used_conditions_exist(&rules.0, conditions) {
        errors.push(e);
    }

    if let Err(e) = all_used_devices_exist(&rules.0, rtu) {
        errors.push(e);
    }

    if let Err(e) = state_sets_are_correct_for_device_type(&rules.0, rtu) {
        errors.push(e);
    }

    if let Err(e) = rules_do_not_have_conflicting_state_sets(&rules.0) {
        errors.push(e);
    }

    if let Err(e) = rules_cannot_set_state_of_device_that_triggered_them(&rules.0, conditions) {
        errors.push(e);
    }

    if errors.len() == 0 {
        return Ok(());
    }
    Err(errors)
}

pub fn all_used_conditions_exist(
    rules: &Vec<Rule>,
    conditions: &ConditionCollection,
) -> Result<()> {
    let condition_ids: Vec<String> = conditions.0.iter().map(|cond| cond.id.clone()).collect();

    for rule in rules {
        if !condition_ids.contains(&rule.condition_id) {
            debug!("The conditions I found were: {:#?}", condition_ids);
            debug!("and you called for: {}", rule.condition_id);
            return fail(
                &rule.id,
                &format!(
                    "the condition it uses (`{}`) was not found in the conditions file",
                    rule.condition_id
                ),
            );
        }
    }

    trace!("Rule validation check passed: all conditions used by rules exist");
    Ok(())
}

pub fn all_used_devices_exist(rules: &Vec<Rule>, rtu: &RTU) -> Result<()> {
    let device_ids: Vec<String> = rtu.devices.iter().map(|device| device.id.clone()).collect();

    for rule in rules {
        for state_set in &rule.set {
            if !device_ids.contains(&state_set.device_id) {
                return fail(
                    &rule.id,
                    &format!(
                        "it would set the state of a device (`{}`) that couldn't be found in the RTU configuration",
                        state_set.device_id
                    ),
                );
            }
        }
    }

    trace!("rule validation check passed: all devices used by rules exist");
    Ok(())
}

pub fn state_sets_are_correct_for_device_type(rules: &Vec<Rule>, rtu: &mut RTU) -> Result<()> {
    for rule in rules {
        for state_set in &rule.set {
            if let Some(device) = rtu.device(&state_set.device_id) {
                // check device type and state set type
                use crate::controllers::Controller::*;

                let ts = &state_set.target_state;

                match device.conn.controller {
                    STR1 | Waveshare | WaveshareV2 => {
                        // relays should have a relay_state.
                        // we'll warn if they try to set the sv or pv of a relay, but it's not an
                        // error
                        if ts.relay_state.is_none() {
                            // No relay state was provided, but the device is a relay
                            return fail(
                                &rule.id,
                                &format!(
                                    "this rule operates on a relay, but does not specify the target state when the rule triggers. relay_state should be set to On or Off in the target state for devices of type {}",
                                    device.conn.controller
                                    )
                                );
                        }

                        if ts.pv.is_some() || ts.sv.is_some() {
                            warn!("rule `{}` tries to set the SV or PV of the device `{}`. This is likely an error with the rule configuration, and will have no effect.", &rule.id, device.conn.controller);
                        }
                    }
                    CN7500 => {
                        // PIDs should not include a PV setting
                        if ts.pv.is_some() {
                            warn!("in rule `{}`, you are attempting to set the PV of a PID. This is not possible, as the process value is a readonly attribute", &rule.id);
                        }

                        // PIDs should include either a relay_state, sv, or both
                        if ts.sv.is_none() && ts.relay_state.is_none() {
                            return fail(
                                &rule.id,
                                &format!("this rule does not set the SV or relay state of a PID, so it has no effect. Either the SV or relay state should be set, or the rule should be removed")
                            );
                        }
                    }
                }
            } else {
                // Normally this would be an error; the device named in the state set doesn't
                // exist. However we already checked this in a previous validator, so we'll do nothing
            }
        }
    }

    trace!("rule validation check passed: all rules set the correct type of state for the device they operate on");
    Ok(())
}

pub fn rules_do_not_have_conflicting_state_sets(rules: &Vec<Rule>) -> Result<()> {
    // we should only be able to set the target of a device one time
    for rule in rules {
        let devices: Vec<&String> = rule
            .set
            .iter()
            .map(|state_set| &state_set.device_id)
            .collect();

        let mut uniq = HashSet::new();
        if !devices.into_iter().all(move |device| uniq.insert(device)) {
            return fail(
                &rule.id,
                "this rule attempts to set a device's state multiple times, which is not allowed.",
            );
        }
    }

    trace!("rule validation check passed: all rules set a devices state only once");
    Ok(())
}

pub fn rules_cannot_set_state_of_device_that_triggered_them(
    rules: &Vec<Rule>,
    conditions: &ConditionCollection,
) -> Result<()> {
    for rule in rules {
        // Find the condition that this rule uses
        if let Some(relevant_condition) = conditions
            .0
            .iter()
            .find(|cond| cond.id == rule.condition_id)
        {
            // Get a list of all IDs of the devices that will be enacted when this rule triggers,
            // ie. the resultant devices
            let resultant_device_ids: Vec<&String> = rule
                .set
                .iter()
                .map(|state_set| &state_set.device_id)
                .collect();

            // If the devices that triggers the rule (the one in the condition) is also a resultant
            // device, we're cooked.
            if resultant_device_ids.contains(&&relevant_condition.device_id) {
                return fail(
                    &rule.id,
                    &format!("this rule will change the state of device `{}`, which is the same device that triggers the rule", relevant_condition.device_id)
                );
            }
        }
    }

    trace!("rule validation check passed: no rules will change the state of the device that triggered them");
    Ok(())
}
