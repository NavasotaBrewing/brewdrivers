use serde::Deserialize;

use crate::defaults::*;
use crate::state::DeviceState;

use crate::model::conditions::ConditionKind;

#[derive(Deserialize)]
pub struct ConditionDefinition {
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

#[cfg(test)]
mod tests {
    use super::*;

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

        let result = serde_yaml::from_str::<ConditionDefinition>(&source);
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

        let result2 = serde_yaml::from_str::<ConditionDefinition>(&source2);
        assert!(result2.is_ok());
    }
}
