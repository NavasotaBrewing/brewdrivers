pub mod condition_definition;
pub mod condition_error;

use condition_error::ConditionError;
use log::*;
use serde::Deserialize;

use crate::logging_utils::device_error;
use crate::model::Device;
use crate::state::DeviceState;
use condition_definition::ConditionDefinition;

#[derive(Deserialize)]
pub enum ConditionKind {
    RelayStateIs,
    PVIsAtLeast,
    PVIsAround,
    PVMeetsSV,
}

pub struct Condition<'a> {
    pub name: String,
    pub id: String,
    pub kind: ConditionKind,
    pub device: &'a mut Device,
    pub state: DeviceState,
    pub margin_above: f64,
    pub margin_below: f64,
}

impl<'a> Condition<'a> {
    pub fn from_definition(def: ConditionDefinition, device: &'a mut Device) -> Self {
        return Self {
            name: def.name,
            id: def.id,
            kind: def.kind,
            device,
            state: def.state,
            margin_above: def.margin_above,
            margin_below: def.margin_below,
        };
    }

    pub async fn evaluate(&mut self) -> Result<bool, ConditionError> {
        trace!(
            "Evaluating condition {} (`{}`) on device {} (`{}`)",
            self.name,
            self.id,
            self.device.name,
            self.device.id
        );

        if let Err(e) = self.device.update().await {
            device_error!(
                self.device,
                &format!(
                    "error updating device when evaluating condition `{}`: {e}",
                    self.id
                )
            );
            return Err(ConditionError::InstrumentError(e));
        }

        match self.kind {
            ConditionKind::RelayStateIs => return self.evaluate_relay_state_is(),
            ConditionKind::PVIsAtLeast => return self.evaluate_pv_is_at_least(),
            ConditionKind::PVIsAround => {
                // This evaluates if the PV is around the target value from the condition
                // definition
                self.ensure_actual_value(self.device.state.pv, "pv")?;
                return self.evaluate_pv_is_around(self.state.pv.unwrap());
            }
            ConditionKind::PVMeetsSV => {
                // This evaluates if the PV is around the SV, with margins applied
                self.ensure_actual_value(self.device.state.pv, "pv")?;
                self.ensure_actual_value(self.device.state.sv, "sv")?;
                return self.evaluate_pv_is_around(self.device.state.sv.unwrap());
            }
        };
    }

    fn ensure_target_value<T>(&self, value: Option<T>, name: &str) -> Result<(), ConditionError> {
        if value.is_none() {
            return Err(ConditionError::MissingTargetValueError {
                condition_id: self.id.clone(),
                state_name: name.to_string(),
            });
        }
        Ok(())
    }

    fn ensure_actual_value<T>(&self, value: Option<T>, name: &str) -> Result<(), ConditionError> {
        if value.is_none() {
            return Err(ConditionError::MissingActualValueError {
                condition_id: self.id.clone(),
                device_id: self.device.id.clone(),
                state_name: name.to_string(),
            });
        }
        Ok(())
    }

    fn evaluate_relay_state_is(&mut self) -> Result<bool, ConditionError> {
        // Ensure we have a target state (from the condition definition)
        self.ensure_target_value(self.state.relay_state, "relay_state")?;
        // And an actual state
        self.ensure_actual_value(self.device.state.relay_state, "relay_state")?;

        let result = self.device.state.relay_state.unwrap() == self.state.relay_state.unwrap();

        trace!(
            "`{}`: Evaluating that `{}`.relay_state is `{}` and found that to be {}",
            self.id,
            self.device.id,
            self.state.relay_state.unwrap(),
            result
        );

        Ok(result)
    }

    fn evaluate_pv_is_at_least(&self) -> Result<bool, ConditionError> {
        self.ensure_target_value(self.state.pv, "pv")?;
        self.ensure_actual_value(self.device.state.pv, "pv")?;

        let result = self.device.state.pv >= self.state.pv;

        trace!(
            "`{}`: Evaluating that `{}`.pv is at least `{}` and found that to be {}",
            self.id,
            self.device.id,
            self.state.pv.unwrap(),
            result
        );

        Ok(result)
    }

    /// Compares the pv of the device to the given value, with the margins applied.
    fn evaluate_pv_is_around(&self, target: f64) -> Result<bool, ConditionError> {
        self.ensure_actual_value(self.device.state.pv, "pv")?;

        let actual = self.device.state.pv.unwrap();

        let lower_bound = target - self.margin_below;
        let upper_bound = target + self.margin_above;

        let result = actual >= lower_bound && actual <= upper_bound;

        trace!(
            "`{}`: Evaluating that `{}`.pv is in the range [{}, {}] and found that to be {}",
            self.id,
            self.device.id,
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

    #[test]
    fn test_condition_from_definition() {
        let source = r#"
            name: My Condition
            id: my-condition
            condition: RelayStateIs
            device_id: relay1
            state:
                relay_state: On
            "#;

        let result = serde_yaml::from_str::<ConditionDefinition>(&source);
        assert!(result.is_ok());

        let mut device = test_relay();
        Condition::from_definition(result.unwrap(), &mut device);
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
            name: format!("My Condition"),
            id: format!("my-condition"),
            kind: ConditionKind::RelayStateIs,
            device: &mut device.clone(),
            state: target_state,
            margin_above: 0.0,
            margin_below: 0.0,
        };

        assert!(condition.evaluate().await.unwrap());

        device.state.relay_state = Some(BinaryState::On);
        device.enact().await.unwrap();

        assert!(!condition.evaluate().await.unwrap());

        condition.state.relay_state = Some(BinaryState::On);

        assert!(condition.evaluate().await.unwrap());
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
            name: format!("My Condition"),
            id: format!("my-condition"),
            kind: ConditionKind::PVIsAtLeast,
            device: &mut omega.clone(),
            state: target_state,
            margin_above: 0.0,
            margin_below: 0.0,
        };

        assert!(condition.evaluate().await.unwrap());

        condition.state.pv = Some(pv + 20.0);

        assert!(!condition.evaluate().await.unwrap());
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
            name: format!("My Condition"),
            id: format!("my-condition"),
            kind: ConditionKind::PVMeetsSV,
            device: &mut omega.clone(),
            state: target_state,
            margin_above: 10.0,
            margin_below: 10.0,
        };

        omega.update().await.unwrap();

        omega.state.sv = omega.state.pv;
        omega.enact().await.unwrap();

        assert!(condition.evaluate().await.unwrap());

        omega.state.sv = Some(omega.state.pv.unwrap() + 4.0);
        omega.enact().await.unwrap();

        assert!(condition.evaluate().await.unwrap());

        omega.state.sv = Some(omega.state.pv.unwrap() + 25.0);
        omega.enact().await.unwrap();

        assert!(!condition.evaluate().await.unwrap());
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
            name: format!("My Condition"),
            id: format!("my-condition"),
            kind: ConditionKind::PVIsAround,
            device: &mut omega.clone(),
            state: target_state,
            margin_above: 5.0,
            margin_below: 5.0,
        };

        assert!(condition.evaluate().await.unwrap());

        condition.state.pv = Some(pv + 20.0);

        assert!(!condition.evaluate().await.unwrap());
    }
}

