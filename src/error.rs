

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Instrument Error: {0}")]
    InstrumentError(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("YAML Parse Error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Model Error: {0}")]
    ModelError(String),
    #[error("Condition Error: {0}")]
    ConditionError(String),
    #[error("Rule Error: {0}")]
    RuleError(String),
    #[error("Validation Error: {0}")]
    ValidationError(String),
    #[error("File not found error: {0}")]
    FileNotFoundError(String),
    #[error("Permission Error: {0}")]
    PermissionError(String)
}
