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

    #[error("Validation Error for (k,v) = (`{key}`,`{value}`): {msg}")]
    ValidationError {
        key: String,
        value: String,
        msg: String,
    }
}

impl ModelError {
    /// Constructs an `ModelError::ValidationError`
    pub fn validation_error(key_value: (&str, &str), msg: &str) -> ModelError {
        return ModelError::ValidationError {
            key: key_value.0.to_string(),
            value: key_value.1.to_string(),
            msg: msg.to_string(),
        };
    }
}
