use crate::model::conditions::ConditionError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("IO Error: {0}")]
    IOError(std::io::Error),

    #[error("Parse error: {0}")]
    SerdeParseError(serde_yaml::Error),

    #[error("Condition `{0}` was not found. Is it defined?")]
    ConditionNotFoundError(String),

    #[error("device `{device_id}` not found. It is needed by the rule `{rule_id}`, either to evaluate the condition or to apply a new state.")]
    DeviceNotFoundError { device_id: String, rule_id: String },

    #[error("{0}")]
    ConditionError(ConditionError),

    #[error("Rule Error: Rule `{rule_id}` failed validation because: `{msg}`")]
    ValidationError { rule_id: String, msg: String },
}
