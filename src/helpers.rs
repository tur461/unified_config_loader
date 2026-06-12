use std::{collections::HashMap, env, str::FromStr};

use crate::{errors::ConfigError, models::ValueSource};

#[doc(hidden)]
pub fn resolve_value_with_fn<T: FromStr + Default>(
    field: &str,
    required: bool,
    default_literal: Option<&str>,
    default_fn: Option<fn() -> Result<T, ConfigError>>,
    file_kvs: &HashMap<String, (String, ValueSource)>,
) -> Result<T, ConfigError> {
    let mut value: Option<T> = None;

    // Source: Environment
    if let Ok(raw) = env::var(field.to_uppercase()) {
        let parsed = raw.parse::<T>().map_err(|_| ConfigError::InvalidValue {
            field: field.to_string(),
            value: raw,
            value_source: ValueSource::Environment,
        })?;
        value = Some(parsed);
    }

    // Source: File
    if let Some(raw) = file_kvs.get(field) {
        let parsed = raw.0.parse::<T>().map_err(|_| ConfigError::InvalidValue {
            field: field.to_string(),
            value: raw.0.clone(),
            value_source: raw.1,
        })?;
        value = Some(parsed);
    }

    // Source: Fallback – check sources in order of precedence
    if let Some(v) = value {
        Ok(v)
    } else if let Some(lit) = default_literal {
        lit.parse::<T>().map_err(|_| ConfigError::InvalidValue {
            field: field.to_string(),
            value: lit.to_string(),
            value_source: ValueSource::Default,
        })
    } else if let Some(func) = default_fn {
        func()
    } else if required {
        Err(ConfigError::MissingRequiredField {
            field: field.to_string(),
        })
    } else {
        Ok(T::default())
    }
}
