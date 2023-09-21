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

    /// Error when validating the configuration
    #[error("Validation Error: {item_id}.{key} = `{value}` (Rule: {rule})")]
    ValidationError {
        // The item that failed validation, usually a device id
        item_id: String,
        // The key/value that broke the rule
        key: String,
        value: String,
        // Description of the rule being broken
        rule: String,
    },
}

impl ModelError {
    /// Constructs an `ModelError::ValidationError`
    pub fn validation_error(item_id: &str, key_value: (&str, &str), rule: &str) -> ModelError {
        ModelError::ValidationError {
            item_id: item_id.to_string(),
            key: key_value.0.to_string(),
            value: key_value.1.to_string(),
            rule: rule.to_string(),
        }
    }
}
