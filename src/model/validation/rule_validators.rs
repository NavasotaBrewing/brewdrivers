use log::{debug, trace};

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
    rtu: &RTU,
) -> std::result::Result<(), Vec<Error>> {
    let mut errors: Vec<Error> = Vec::new();

    if let Err(e) = all_used_conditions_exist(&rules.0, conditions) {
        errors.push(e);
    }

    if let Err(e) = all_used_devices_exist(&rules.0, rtu) {
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
