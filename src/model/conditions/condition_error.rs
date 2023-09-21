use thiserror::Error;

use crate::controllers::InstrumentError;

/// An error when evaluating a condition.
///
/// While technically the MissingTargetValueError is checked at start time and *shouldn't* ever be
/// returned at run time, ConditionErrors should only be returned at run time.
/// [ModelError](crate::model::ModelError) is returned at start time if the condition fails it's
/// validation.
#[derive(Error, Debug)]
pub enum ConditionError {
    #[error("IO Error: {0}")]
    IOError(std::io::Error),

    #[error("Parse error: {0}")]
    SerdeParseError(serde_yaml::Error),

    /// An instrument error, in case the instrument fails when we're updating it's value
    #[error("{0}")]
    InstrumentError(InstrumentError),

    /// They didn't define the target state in the config file when defining the rule
    #[error("Missing target value `{state_name}` when evaluating condition `{condition_id}`. This state must be defined in the condition definition.")]
    MissingTargetValueError {
        condition_id: String,
        state_name: String,
    },

    /// Actual value on the device was none. This should basically never happen.
    #[error("Missing actual value `{state_name}` on device `{device_id}` when evaluating condition `{condition_id}`")]
    MissingActualValueError {
        condition_id: String,
        device_id: String,
        state_name: String,
    },

    #[error(
        "Device `{device_id}` not found in device list when evaluating condition `{condition_id}`"
    )]
    MissingDeviceError {
        condition_id: String,
        device_id: String,
    },

    /// Error when validating the configuration
    #[error("Validation Error: Condition `{condition_id}` failed validation because: `{msg}`")]
    ValidationError { condition_id: String, msg: String },
}

impl ConditionError {
    pub fn validation_error(condition_id: &str, msg: String) -> Self {
        Self::ValidationError {
            condition_id: condition_id.to_string(),
            msg,
        }
    }
}
