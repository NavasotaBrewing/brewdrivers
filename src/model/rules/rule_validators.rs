use log::{debug, info};

use crate::{
    error::Error,
    model::{conditions::ConditionCollection, RTU},
    Result,
};

use super::Rule;

fn fail(rule_id: &str, why: &str) -> Result<()> {
    Err(Error::ValidationError(format!(
        "rule validation failed on component `{rule_id}` because {why}",
    )))
}

pub fn all_validators(rules: &Vec<Rule>) -> Result<()> {
    all_used_conditions_exist(&rules)?;
    all_used_devices_exist(&rules)?;
    Ok(())
}

pub fn all_used_conditions_exist(rules: &Vec<Rule>) -> Result<()> {
    // We call get_from_file because that bypasses the conditions validation.
    // We don't want to revalidate all the conditions for each rule validation
    let conditions = match ConditionCollection::get_from_file() {
        Ok(conditions) => conditions,
        Err(e) => {
            return Err(Error::ValidationError(format!(
                "couldn't get list of conditions to use in rule validation. Probably an IO error: {e}"
            )));
        }
    };

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

    info!("Rule validation check passed: all conditions used by rules exist");
    Ok(())
}

pub fn all_used_devices_exist(rules: &Vec<Rule>) -> Result<()> {
    // TODO: this will validate the RTU when called. Maybe add a way to bypass?
    let rtu = match RTU::generate() {
        Ok(rtu) => rtu,
        Err(e) => {
            return Err(Error::ValidationError(format!(
                "Couldn't get device list when validating rules: {e}"
            )))
        }
    };

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

    info!("rule validation check passed: all devices used by rules exist");
    Ok(())
}
