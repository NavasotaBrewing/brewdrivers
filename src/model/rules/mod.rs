pub mod rule_error;

use std::fs;

use crate::{defaults::rules_file, state::DeviceState};
use log::*;
use serde::Deserialize;

use super::{conditions::ConditionCollection, Device, RTU};
pub use rule_error::RuleError;

#[derive(Debug, Deserialize)]
pub struct RuleSet(pub Vec<Rule>);

impl RuleSet {
    fn get_from_file() -> Result<Self, RuleError> {
        let file_path = rules_file();
        info!("Generating rules. Using config file: {:?}", file_path);

        // Get the contents of the config file
        let file_contents = fs::read_to_string(file_path).map_err(RuleError::IOError)?;

        // Deserialize the file. Return an Err if it doesn't succeed
        let rules = serde_yaml::from_str::<RuleSet>(&file_contents)
            .map_err(RuleError::SerdeParseError)?;

        Ok(rules)
    }

    pub fn get_all() -> Result<Self, RuleError> {
        let rules = RuleSet::get_from_file()?;
        rules.validate()?;
        Ok(rules)
    }

    /// Applies all rules in the rule set over the given devices
    pub async fn apply_all(&self, mut devices: Vec<Device>) -> Result<(), RuleError> {
        for rule in &self.0 {
            rule.apply(devices.iter_mut().collect()).await?;
        }
        Ok(())
    }

    /// Gets all rules and conditions from the rules/conditions files, and generates an RTU from
    /// the rtu file, then applies all rules to all devices.
    pub async fn apply_all_to_all_devices() -> Result<(), RuleError> {
        let rule_set = Self::get_all()?;
        let rtu = RTU::generate()?;
        rule_set.apply_all(rtu.devices).await?;
        Ok(())
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
    /// Searches through the provided devices for the proper device to evaluate a condition.
    /// If the condition is true, apply the target states to the resultant devices (the ones named
    /// in the rule definition).
    ///
    /// This calls `update()` on the dependant device, on the resultant device if the condition
    /// passes, and `enact()` on the resultant devices if they're state needs updating.
    pub async fn apply(&self, mut devices: Vec<&mut Device>) -> Result<(), RuleError> {
        // These error checks should be caught by validators when iris starts, but they're still
        // checked here because that's the spirit of Rust
        let mut condition = match ConditionCollection::get_by_id(&self.condition_id) {
            Some(cond) => cond,
            None => return Err(RuleError::ConditionNotFoundError(self.condition_id.clone())),
        };

        // Get the dependant device from the list. Return an error if it can't be found
        let dependant_device =
            match devices.iter_mut().find(|dev| dev.id == condition.device_id) {
                Some(dep) => dep,
                None => {
                    return Err(RuleError::DeviceNotFoundError {
                        device_id: condition.device_id,
                        rule_id: self.id.clone(),
                    })
                }
            };

        // Update the dependant device so that we have new values
        dependant_device.update().await?;
        // And evaluate the condition based on that device
        let condition_result = condition.evaluate_on(dependant_device).await;

        // Match on the result
        match condition_result {
            // If the condition isn't true, then we don't want to do anything. Just log some
            // messages.
            Ok(false) => {
                trace!("evaluated condition `{}` using device `{}` for rule `{}` and found result to be false (does not apply)", condition.id, dependant_device.id, self.id);
                // Do nothing
                Ok(())
            }
            // If the condition is true, then we want to potentially enact() some states.
            Ok(true) => {
                trace!("evaluated condition `{}` using device `{}` for rule `{}` and found result to be true (rule does apply)", condition.id, dependant_device.id, self.id);
                // Apply state sets
                self.apply_state_sets(devices).await?;
                Ok(())
            }
            Err(e) => {
                error!(
                    "rule `{}` encountered an error when evaluating condition `{}`: {}",
                    self.id, condition.id, e
                );
                Err(RuleError::ConditionError(e))
            }
        }
    }

    /// Applies the specific StateSets to the right devices. This will call update() on the
    /// resultant devices to evaluate if they need an enact() call based on their current state
    /// values.
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

            // Only call update on the resultant devices (the ones that get their state potentially
            // changed)
            found_device.update().await?;

            // If the device is already in that state, then don't enact
            if found_device.state != new_state.target_state {
                // Update the state and enact
                found_device.state = new_state.target_state.clone();

                info!(
                    "device `{}` state is being changed due to the rule `{}`: {:?}",
                    found_device.id, self.id, found_device.state
                );

                found_device.enact_without_applying_rules().await?;
            } else {
                trace!("device `{}` would be updated due a the rule `{}`, but it's current state already matched the target state", found_device.id, self.id);
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

        let rule = serde_yaml::from_str::<Rule>(source);
        assert_ok!(rule);
    }
}
