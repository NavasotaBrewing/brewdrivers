use thiserror::Error;

use crate::controllers::InstrumentError;

#[derive(Error, Debug)]
pub enum ConditionError {
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
}
