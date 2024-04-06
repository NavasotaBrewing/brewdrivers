use std::fs;

use crate::{defaults::rules_file, state::DeviceState};
use log::*;
use serde::Deserialize;

use super::{conditions::ConditionCollection, Device, RTU};
use crate::{error::Error, Result};

#[derive(Debug, Deserialize)]
pub struct RuleSet(pub Vec<Rule>);

impl RuleSet {
    pub fn generate() -> Result<Self> {
        let file_path = rules_file();
        info!("Generating rules. Using config file: {:?}", file_path);

        // Get the contents of the config file
        let file_contents = fs::read_to_string(file_path).map_err(Error::IOError)?;

        // Deserialize the file. Return an Err if it doesn't succeed
        let rules = serde_yaml::from_str::<RuleSet>(&file_contents).map_err(Error::YamlError)?;

        Ok(rules)
    }

    /// Applies all rules in the rule set over the given devices
    pub async fn apply_all(&self, mut devices: Vec<Device>) -> Result<()> {
        for rule in &self.0 {
            rule.apply(devices.iter_mut().collect()).await?;
        }
        Ok(())
    }

    /// Gets all rules and conditions from the rules/conditions files, and generates an RTU from
    /// the rtu file, then applies all rules to all devices.
    pub async fn apply_all_to_all_devices() -> Result<()> {
        let rule_set = Self::generate()?;
        let rtu = RTU::generate()?;
        rule_set.apply_all(rtu.devices).await?;
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
    pub async fn apply(&self, mut devices: Vec<&mut Device>) -> Result<()> {
        // These error checks should be caught by validators when iris starts, but they're still
        // checked here because that's the spirit of Rust
        let mut condition = match ConditionCollection::get_by_id(&self.condition_id) {
            Some(cond) => cond,
            None => {
                return Err(Error::RuleError(format!(
                    "condition `{}` was not found in the condition definitions file",
                    self.condition_id
                )))
            }
        };

        // Get the dependant device from the list. Return an error if it can't be found
        let dependant_device = match devices.iter_mut().find(|dev| dev.id == condition.device_id) {
            Some(dep) => dep,
            None => {
                return Err(Error::RuleError(format!(
                    "device `{}` was not found in the collection, and it is needed to evaluate condition `{}`",
                    condition.device_id,
                    condition.id,
                )));
            }
        };

        // Update the dependant device so that we have new values
        dependant_device.update_without_applying_rules().await?;
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
                Err(Error::ConditionError(format!("{e}")))
            }
        }
    }

    /// Applies the specific StateSets to the right devices. This will call update() on the
    /// resultant devices to evaluate if they need an enact() call based on their current state
    /// values.
    async fn apply_state_sets(&self, mut devices: Vec<&mut Device>) -> Result<()> {
        // We've already checked the condition, so just apply all new state sets to all devices.
        for new_state in &self.set {
            let found_device = match devices.iter_mut().find(|dev| dev.id == new_state.device_id) {
                Some(dev) => dev,
                None => {
                    return Err(Error::RuleError(format!(
                        "device `{}` was not found in the collection, and it is needed to set a new state because a rule (`{}`) was triggered",
                        new_state.device_id,
                        self.id
                    )));
                }
            };

            // Only call update on the resultant devices (the ones that get their state potentially
            // changed)
            found_device.update_without_applying_rules().await?;

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
    use std::time::Duration;

    use crate::{controllers::CN7500, state::BinaryState};

    use super::*;
    use tokio_test::assert_ok;

    // TODO: Write more tests for rules

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

    #[tokio::test]
    async fn test_rule_triggers_enaction() {
        // Generate the RTU so we can grab real devices
        let rtu = RTU::generate().unwrap();

        // get two real relays
        let mut device_a = rtu
            .devices
            .iter()
            .find(|dev| dev.id == "relay0")
            .unwrap()
            .clone();
        let mut device_b = rtu
            .devices
            .iter()
            .find(|dev| dev.id == "relay1")
            .unwrap()
            .clone();

        // Turn both relays off as our base state
        device_a.state.relay_state = Some(BinaryState::Off);
        device_b.state.relay_state = Some(BinaryState::Off);

        device_a.enact().await.unwrap();
        device_b.enact().await.unwrap();

        device_a.update().await.unwrap();
        device_b.update().await.unwrap();

        // Assert that they're off
        assert_eq!(device_a.state.relay_state, Some(BinaryState::Off));
        assert_eq!(device_b.state.relay_state, Some(BinaryState::Off));

        // Only turn on relay A
        device_a.state.relay_state = Some(BinaryState::On);
        device_a.enact().await.unwrap();

        // and assert that relay B is now on
        device_b.update().await.unwrap();
        assert_eq!(device_b.state.relay_state, Some(BinaryState::On));

        // Now we turn off relay A and assert that relay B turned off too
        device_a.state.relay_state = Some(BinaryState::Off);
        device_a.enact().await.unwrap();
        device_b.update().await.unwrap();
        assert_eq!(device_b.state.relay_state, Some(BinaryState::Off));
    }

    #[tokio::test]
    async fn test_rules_apply_without_enaction() {
        // Rules should apply when the hardware changes without us enacting them.
        // ie. the pv changes due to rising temps and triggers a condition.
        // There's not a great way to do this without checking perdiodically,
        // so instead I think we'll just apply rules when we update devices.

        let mut rtu = RTU::generate().unwrap();
        let pv: f64;

        // We have some scope skullduggery here because we can't have two
        // open connections to the omega at once time. We use the braces to
        // make sure the omega is dropped, then we do the backdoor stuff, then
        // we will get it again.
        let mut relay = rtu.device_cloned("wsrelay0").unwrap();
        // Turn this relay off. There's a rule that will turn it on when the PV meets to SV
        // on the omega1 device
        relay.state.relay_state = Some(BinaryState::Off);
        relay.enact().await.unwrap();

        {
            let mut omega = rtu.device_cloned("omega1").unwrap();
            // Set it to well outside the margin
            omega.update().await.unwrap();
            pv = omega.state.pv.unwrap();
            omega.state.sv = Some(pv + 25.0);
            omega.enact().await.unwrap();
        }

        // Relay should still be off
        relay.update().await.unwrap();
        assert_eq!(relay.state.relay_state.unwrap(), BinaryState::Off);

        {
            // Open a backdoor to the device, so we can control it directly.
            // This will allow us to change the SV without calling enact() on the device,
            // which would trigger a rule check.
            // We have to do this because we can't change the pv, so we can't set it to the correct
            // value to trigger this test. Instead, we'll set the SV to the PV and pretend that the
            // PV rose to meet the SV
            let mut omega_backdoor =
                CN7500::connect(0x16, "/dev/ttyUSB0", 19200, Duration::from_millis(50))
                    .await
                    .expect("Couldn't get device");
            // Set the sv manually to within the margin.
            // In a perfect world, we would have something detect this and trigger
            // the rule, but for now the rule will only be trigger after we call update()
            // on the device.
            omega_backdoor.set_sv(pv).await.unwrap();
        }

        let mut omega = rtu.device_cloned("omega1").unwrap();

        // Relay should still be off
        relay.update().await.unwrap();
        assert_eq!(relay.state.relay_state.unwrap(), BinaryState::Off);

        // This should trigger the rule and turn the relay on
        omega.update().await.unwrap();

        relay.update().await.unwrap();
        assert_eq!(relay.state.relay_state.unwrap(), BinaryState::On);
    }
}
