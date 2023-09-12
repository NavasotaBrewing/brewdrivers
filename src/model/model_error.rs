use thiserror::Error;

/// Error types that could occur when working with models
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Configuration file not found")]
    FileNotFound,

    #[error("IO error: {0}")]
    IOError(std::io::Error),

    #[error("Permission error, cannot access configuration file")]
    PermissionError,

    #[error("Serde parse error: {0}")]
    SerdeParseError(serde_yaml::Error),

    #[error("Validation Error: {item_id}.{key} = `{value}`. {msg}")]
    ValidationError {
        // The item that failed validation, usually a device id
        item_id: String,
        key: String,
        value: String,
        msg: String,
    },
}

impl ModelError {
    /// Constructs an `ModelError::ValidationError`
    pub fn validation_error(item_id: &str, key_value: (&str, &str), msg: &str) -> ModelError {
        return ModelError::ValidationError {
            item_id: item_id.to_string(),
            key: key_value.0.to_string(),
            value: key_value.1.to_string(),
            msg: msg.to_string(),
        };
    }
}
