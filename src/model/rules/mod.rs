pub mod rule_error;

use std::fs;

use crate::{defaults::rules_file, state::DeviceState};
use log::*;
use serde::Deserialize;

use super::{conditions::ConditionCollection, Device};
pub use rule_error::RuleError;

#[derive(Debug, Deserialize)]
pub struct RuleSet(pub Vec<Rule>);

impl RuleSet {
    fn get_from_file() -> Result<Self, RuleError> {
        let file_path = rules_file();
        info!("Generating rules. Using config file: {:?}", file_path);

        // Get the contents of the config file
        let file_contents = fs::read_to_string(file_path).map_err(|err| RuleError::IOError(err))?;

        // Deserialize the file. Return an Err if it doesn't succeed
        let rules = serde_yaml::from_str::<RuleSet>(&file_contents)
            .map_err(|err| RuleError::SerdeParseError(err))?;

        Ok(rules)
    }

    pub fn get_all() -> Result<Self, RuleError> {
        let rules = RuleSet::get_from_file()?;
        rules.validate()?;
        Ok(rules)
    }

    pub fn validate(&self) -> Result<(), RuleError> {
        // For now, do nothing
        // TODO: write validators
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct StateSet {
    pub device_id: String,
    #[serde(rename = "to")]
    pub target_state: DeviceState,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    /// ID of the rule. Normal ID rules apply.
    pub id: String,
    /// Friendly name of the rule
    pub name: String,
    /// ID of the condition to check
    #[serde(rename = "when")]
    pub condition_id: String,
    /// The other states to set
    pub set: Vec<StateSet>,
}

impl Rule {
    /// Evaluates a rule and applies it's affects onto a set of devices.
    ///
    /// You must provide the device named in the rules condition, and the devices
    /// affected.
    ///
    /// This *does* call update on the dependant device, and enact if necessary on the resultant
    /// devices
    pub async fn apply(&self, mut devices: Vec<&mut Device>) -> Result<(), RuleError> {
        // These error checks should be caught by validators when iris starts, but they're still
        // checked here because that's the spirit of Rust
        debug!("About to look for condition");
        let mut condition = match ConditionCollection::get_by_id(&self.condition_id) {
            Some(cond) => cond,
            None => return Err(RuleError::ConditionNotFoundError(self.condition_id.clone())),
        };

        debug!("Found relevant condition `{}`", condition.id);

        let mut dependant_device =
            match devices.iter_mut().find(|dev| dev.id == condition.device_id) {
                Some(dep) => dep,
                None => {
                    return Err(RuleError::DeviceNotFoundError {
                        device_id: condition.device_id,
                        rule_id: self.id.clone(),
                    })
                }
            };

        dependant_device
            .update()
            .await
            .map_err(|e| RuleError::InstrumentError(e))?;

        let condition_result = condition.evaluate_on(&mut dependant_device).await;

        match condition_result {
            Ok(false) => {
                trace!("Evaluated condition `{}` using device `{}` for rule `{}` and found result to be false (does not apply)", condition.id, dependant_device.id, self.id);
                // Do nothing
                return Ok(());
            }
            Ok(true) => {
                trace!("Evaluated condition `{}` using device `{}` for rule `{}` and found result to be true (rule does apply)", condition.id, dependant_device.id, self.id);
                // Apply state sets
                self.apply_state_sets(devices).await?;
                return Ok(());
            }
            Err(e) => {
                error!(
                    "Rule `{}` encountered an error when evaluating condition `{}`: {}",
                    self.id, condition.id, e
                );
                return Err(RuleError::ConditionError(e));
            }
        }
    }

    /// Applies the specific StateSets to the right devices
    async fn apply_state_sets(&self, mut devices: Vec<&mut Device>) -> Result<(), RuleError> {
        // We've already checked the condition, so just apply all new state sets to all devices.
        for new_state in &self.set {
            let found_device = match devices.iter_mut().find(|dev| dev.id == new_state.device_id) {
                Some(dev) => dev,
                None => {
                    return Err(RuleError::DeviceNotFoundError {
                        device_id: new_state.device_id.clone(),
                        rule_id: self.id.clone(),
                    })
                }
            };

            // If the device is already in that state, then don't enact
            if found_device.state != new_state.target_state {
                // Update the state and enact
                found_device.state = new_state.target_state.clone();
                found_device
                    .enact()
                    .await
                    .map_err(|e| RuleError::InstrumentError(e))?
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::assert_ok;

    #[test]
    fn test_deserialize_rule() {
        let source = r#"
            id: my-rule
            name: My Rule
            when: condition1
            set:
              - device_id: omega1
                to:
                  sv: 192.0
                  relay_state: On
              - device_id: relay1
                to:
                  relay_state: On
            "#;

        let rule = serde_yaml::from_str::<Rule>(&source);
        assert_ok!(rule);
    }
}
