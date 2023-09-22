pub mod condition_validators;

use std::fs;

use crate::defaults::*;
use log::*;
use serde::{Deserialize, Serialize};

use crate::defaults::conditions_file;
// use crate::logging_utils::device_error;
use crate::model::Device;
use crate::state::DeviceState;
use crate::{error::Error, Result};

#[derive(Deserialize)]
pub struct ConditionCollection(pub Vec<Condition>);

impl ConditionCollection {
    fn get_from_file() -> Result<Self> {
        let file_path = conditions_file();
        info!("Generating Conditions. Using config file: {:?}", file_path);

        // Get the contents of the config file
        let file_contents = fs::read_to_string(file_path).map_err(Error::IOError)?;

        // Deserialize the file. Return an Err if it doesn't succeed
        let conditions = serde_yaml::from_str::<ConditionCollection>(&file_contents)
            .map_err(Error::YamlError)?;

        Ok(conditions)
    }

    /// Gets all conditions from the file, and validates them
    pub fn get_all() -> Result<Self> {
        let conditions = ConditionCollection::get_from_file()?;
        conditions.validate()?;
        Ok(conditions)
    }

    /// Runs all validators on the conditions found
    pub fn validate(&self) -> Result<()> {
        use condition_validators::*;

        conditions_have_unique_ids(&self.0)?;
        conditions_have_existing_device(&self.0)?;
        conditions_have_correct_device_type(&self.0)?;
        conditions_have_no_whitespace(&self.0)?;

        Ok(())
    }

    /// Gets conditions from the conditions file, but filters then by the device id
    ///
    /// Validates only the filtered ones.
    pub fn get_for_device(device_id: &str) -> Result<Self> {
        let collection = ConditionCollection::get_from_file()?;
        let filtered_collection = ConditionCollection(
            collection
                .0
                .into_iter()
                .filter(|cond| cond.device_id == device_id)
                .collect(),
        );

        filtered_collection.validate()?;

        Ok(filtered_collection)
    }

    pub fn get_by_id(condition_id: &str) -> Option<Condition> {
        let collection = ConditionCollection::get_from_file().ok()?;
        collection
            .0
            .into_iter()
            .find(|cond| cond.id == condition_id)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ConditionKind {
    RelayStateIs,
    PVIsAtLeast,
    PVIsAround,
    PVMeetsSV,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Condition {
    /// Name of the condition
    pub name: String,
    /// Condition ID. Normal rules apply (unique, no whitespace)
    pub id: String,
    /// Kind of condition
    #[serde(rename = "condition")]
    pub kind: ConditionKind,
    /// ID of the device
    pub device_id: String,
    /// Target state
    #[serde(default)]
    pub state: DeviceState,
    /// margin above the value for conditions to be met (for PVIsAround)
    #[serde(default = "default_condition_margin_above")]
    pub margin_above: f64,
    /// margin below the value for conditions to be met (for PVIsAround)
    #[serde(default = "default_condition_margin_below")]
    pub margin_below: f64,
}

impl Condition {
    /// Evaluates if the condition is true or false.
    ///
    /// Note that this does *not* poll the device for updated values. It's your responsibility to
    /// update the device before calling this method on it.
    pub async fn evaluate_on(&mut self, device: &mut Device) -> Result<bool> {
        trace!(
            "Evaluating condition {} (`{}`) on device {} (`{}`)",
            self.name,
            self.id,
            device.name,
            device.id
        );

        // Update the device state so we have accurate values
        // if let Err(e) = device.update().await {
        //     device_error!(
        //         device,
        //         &format!(
        //             "error updating device when evaluating condition `{}`: {e}",
        //             self.id
        //         )
        //     );
        //     return Err(ConditionError::InstrumentError(e));
        // }

        match self.kind {
            ConditionKind::RelayStateIs => self.evaluate_relay_state_is(device),
            ConditionKind::PVIsAtLeast => self.evaluate_pv_is_at_least(device),
            ConditionKind::PVIsAround => {
                // This evaluates if the PV is around the target value from the condition
                // definition
                self.ensure_actual_value(device.state.pv, "pv", device)?;
                self.evaluate_pv_is_around(self.state.pv.unwrap(), device)
            }
            ConditionKind::PVMeetsSV => {
                // This evaluates if the PV is around the SV, with margins applied
                self.ensure_actual_value(device.state.pv, "pv", device)?;
                self.ensure_actual_value(device.state.sv, "sv", device)?;
                self.evaluate_pv_is_around(device.state.sv.unwrap(), device)
            }
        }
    }

    fn ensure_target_value<T>(&self, value: Option<T>, name: &str) -> Result<()> {
        if value.is_none() {
            return Err(Error::ConditionError(format!(
                "condition `{}` is missing target value `{}`",
                self.id, name
            )));
        }
        Ok(())
    }

    // We need to pass in a device for error reporting
    fn ensure_actual_value<T>(&self, value: Option<T>, name: &str, device: &Device) -> Result<()> {
        if value.is_none() {
            return Err(Error::ConditionError(format!(
                "when evaluating condition `{}`, missing target value `{}` which should be provided by device `{}`",
                self.id,
                name,
                device.id
            )));
        }
        Ok(())
    }

    fn evaluate_relay_state_is(&mut self, device: &Device) -> Result<bool> {
        // Ensure we have a target state (from the condition definition)
        self.ensure_target_value(self.state.relay_state, "relay_state")?;
        // And an actual state
        self.ensure_actual_value(device.state.relay_state, "relay_state", device)?;

        let result = device.state.relay_state.unwrap() == self.state.relay_state.unwrap();

        trace!(
            "`{}`: Evaluating that `{}`.relay_state is `{}` and found that to be {}",
            self.id,
            device.id,
            self.state.relay_state.unwrap(),
            result
        );

        Ok(result)
    }

    fn evaluate_pv_is_at_least(&self, device: &Device) -> Result<bool> {
        self.ensure_target_value(self.state.pv, "pv")?;
        self.ensure_actual_value(device.state.pv, "pv", device)?;

        let result = device.state.pv >= self.state.pv;

        trace!(
            "`{}`: Evaluating that `{}`.pv is at least `{}` and found that to be {}",
            self.id,
            device.id,
            self.state.pv.unwrap(),
            result
        );

        Ok(result)
    }

    /// Compares the pv of the device to the given value, with the margins applied.
    fn evaluate_pv_is_around(&self, target: f64, device: &Device) -> Result<bool> {
        self.ensure_actual_value(device.state.pv, "pv", device)?;

        let actual = device.state.pv.unwrap();

        let lower_bound = target - self.margin_below;
        let upper_bound = target + self.margin_above;

        let result = actual >= lower_bound && actual <= upper_bound;

        trace!(
            "`{}`: Evaluating that `{}`.pv is in the range [{}, {}] and found that to be {}",
            self.id,
            device.id,
            lower_bound,
            upper_bound,
            result
        );
        trace!(
            "`{}`: range was found by taking target value {} and applying the lower/upper margins: [-{},+{}]",
            self.id,
            target,
            self.margin_below,
            self.margin_above
        );
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        controllers::{BinaryState, Controller},
        tests::test_device_from_type,
    };

    fn test_relay() -> Device {
        test_device_from_type(Controller::STR1)
    }

    #[tokio::test]
    async fn test_relay_state_is_condition() {
        let mut device = test_relay();

        // Make sure it's off
        device.state.relay_state = Some(BinaryState::Off);
        device.enact().await.unwrap();

        let target_state = DeviceState {
            relay_state: Some(BinaryState::Off),
            pv: None,
            sv: None,
        };

        let mut condition = Condition {
            name: "My Condition".to_string(),
            id: "my-condition".to_string(),
            kind: ConditionKind::RelayStateIs,
            device_id: device.id.clone(),
            state: target_state,
            margin_above: 0.0,
            margin_below: 0.0,
        };

        assert!(condition.evaluate_on(&mut device).await.unwrap());

        device.state.relay_state = Some(BinaryState::On);
        device.enact().await.unwrap();

        assert!(!condition.evaluate_on(&mut device).await.unwrap());

        condition.state.relay_state = Some(BinaryState::On);

        assert!(condition.evaluate_on(&mut device).await.unwrap());
    }

    #[tokio::test]
    async fn test_pv_is_at_least() {
        let mut omega = test_device_from_type(Controller::CN7500);

        // We can't set the PV and we never really know what it is
        omega.update().await.unwrap();
        let pv = omega.state.pv.unwrap();

        let target_state = DeviceState {
            relay_state: None,
            pv: Some(pv - 20.0),
            sv: None,
        };

        let mut condition = Condition {
            name: "My Condition".to_string(),
            id: "my-condition".to_string(),
            kind: ConditionKind::PVIsAtLeast,
            device_id: omega.id.clone(),
            state: target_state,
            margin_above: 0.0,
            margin_below: 0.0,
        };

        assert!(condition.evaluate_on(&mut omega).await.unwrap());

        condition.state.pv = Some(pv + 20.0);

        assert!(!condition.evaluate_on(&mut omega).await.unwrap());
    }

    #[tokio::test]
    async fn test_pv_meets_sv() {
        let mut omega = test_device_from_type(Controller::CN7500);

        // Target state actually doesn't matter here because we want to compare the pv and sv
        // values on the controller itself
        let target_state = DeviceState {
            relay_state: None,
            pv: None,
            sv: None,
        };

        let mut condition = Condition {
            name: "My Condition".to_string(),
            id: "my-condition".to_string(),
            kind: ConditionKind::PVMeetsSV,
            device_id: omega.id.clone(),
            state: target_state,
            margin_above: 10.0,
            margin_below: 10.0,
        };

        omega.update().await.unwrap();

        omega.state.sv = omega.state.pv;
        omega.enact().await.unwrap();

        assert!(condition.evaluate_on(&mut omega).await.unwrap());

        omega.state.sv = Some(omega.state.pv.unwrap() + 4.0);
        omega.enact().await.unwrap();

        assert!(condition.evaluate_on(&mut omega).await.unwrap());

        omega.state.sv = Some(omega.state.pv.unwrap() + 25.0);
        omega.enact().await.unwrap();

        assert!(!condition.evaluate_on(&mut omega).await.unwrap());
    }

    #[tokio::test]
    async fn test_pv_is_around() {
        let mut omega = test_device_from_type(Controller::CN7500);

        // We can't set the PV
        omega.update().await.unwrap();
        let pv = omega.state.pv.unwrap();

        let target_state = DeviceState {
            relay_state: None,
            // Margins should apply to this condition
            pv: Some(pv + 2.0),
            sv: None,
        };

        let mut condition = Condition {
            name: "My Condition".to_string(),
            id: "my-condition".to_string(),
            kind: ConditionKind::PVIsAround,
            device_id: omega.id.clone(),
            state: target_state,
            margin_above: 5.0,
            margin_below: 5.0,
        };

        assert!(condition.evaluate_on(&mut omega).await.unwrap());

        condition.state.pv = Some(pv + 20.0);

        assert!(!condition.evaluate_on(&mut omega).await.unwrap());
    }

    #[test]
    fn test_deserialize_condition() {
        let source = r#"
            name: My Condition
            id: my-condition
            condition: RelayStateIs
            device_id: relay1
            state:
                relay_state: On
            "#;

        let result = serde_yaml::from_str::<Condition>(source);
        assert!(result.is_ok());

        let source2 = r#"
            name: My Condition
            id: my-condition
            condition: PVIsAround
            device_id: omega1
            state:
                pv: 172.0
            margin_above: 5.0
            margin_below: 0.0
            "#;

        let result2 = serde_yaml::from_str::<Condition>(source2);
        assert!(result2.is_ok());
    }
}
