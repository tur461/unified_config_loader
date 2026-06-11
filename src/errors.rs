use super::models::ValueSource;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read configuration file '{path}': {source}")]
    FileRead {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("parse error in configuration file at line {line}: {message}")]
    FileParse { line: usize, message: String },

    #[error("missing required configuration field: '{field}'")]
    MissingRequiredField { field: String },

    #[error("invalid value for field '{field}' from {value_source}: '{value}'")]
    InvalidValue {
        field: String,
        value: String,
        value_source: ValueSource,
    },

    #[error("validation error: {0}")]
    ValidationError(String),
}
