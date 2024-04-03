use log::info;

use crate::{error::Error, model::conditions::ConditionCollection, Result};

use super::Rule;

fn fail(rule_id: &str, why: &str) -> Result<()> {
    Err(Error::ValidationError(format!(
        "rule validation failed on component `{rule_id}` because {why}",
    )))
}

pub fn all_validators(rules: &Vec<Rule>) -> Result<()> {
    all_used_conditions_exist(&rules)?;
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

    let condition_names: Vec<String> = conditions.0.iter().map(|cond| cond.id.clone()).collect();

    for rule in rules {
        if !condition_names.contains(&rule.condition_id) {
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
