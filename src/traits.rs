use super::errors::ConfigError;

pub trait Config: Sized {
    fn load() -> Result<Self, ConfigError>;
}

pub trait Validate {
    fn validate(&self) -> Result<(), ConfigError>;
}

pub trait ValidatePartial {
    fn validate_partial(&self) -> Vec<ConfigError>;
}

impl<T: Validate> ValidatePartial for T {
    fn validate_partial(&self) -> Vec<ConfigError> {
        match self.validate() {
            Ok(()) => vec![],
            Err(e) => vec![e],
        }
    }
}
