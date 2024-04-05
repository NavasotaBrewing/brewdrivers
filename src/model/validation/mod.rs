/// This module handles validation for all the model structs.
///
/// In a nutshell, it will take a generated RTU, Rule Set, Condition collection, etc, and make sure
/// that they are actually compatible with each other and valid. For example, if you define a
/// device with a port that doesn't exist, or you define a rule over a device that doesn't exist,
/// these validators will catch this and report the errors.
///
/// Note that the model structs will fail to properly deserialize if there is a YAML syntax error.
/// These validators do not check YAML syntax error, instead this happens earlier through serde.
use super::{conditions::ConditionCollection, rules::RuleSet, RTU};
use crate::Result;

pub mod condition_validators;
pub mod rtu_validators;
pub mod rule_validators;

pub fn validate_all(rtu: &RTU, conditions: &ConditionCollection, rules: &RuleSet) -> Result<()> {
    // TODO: Add logging of errors?
    // TODO: add error collection?
    // TODO: Change these to accept RuleSet and ConditionCollection
    condition_validators::all(&conditions.0)?;
    rtu_validators::all(rtu)?;
    rule_validators::all(&rules.0)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_validation() {
        let rtu = RTU::generate().unwrap();
        let conditions = ConditionCollection::generate().unwrap();
        let rules = RuleSet::generate().unwrap();

        assert!(validate_all(&rtu, &conditions, &rules).is_ok());
    }
}
