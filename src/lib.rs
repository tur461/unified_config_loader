pub mod errors;
pub mod helpers;
#[cfg(feature = "hot-reload")]
pub mod hot_reload;
pub mod models;
pub mod traits;

pub use config_derive_pm::*;
pub use errors::*;
pub use helpers::*;
#[cfg(feature = "hot-reload")]
pub use hot_reload::*;
pub use models::*;
pub use traits::*;

use std::collections::HashMap;

pub fn parse_config_file(content: &str) -> Result<HashMap<String, String>, ConfigError> {
    let mut map = HashMap::new();
    for (line_no, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| ConfigError::FileParse {
            line: line_no + 1,
            message: "missing '=' separator".to_string(),
        })?;
        let key = key.trim().to_string();
        if key.is_empty() {
            return Err(ConfigError::FileParse {
                line: line_no + 1,
                message: "empty key".to_string(),
            });
        }

        let raw_value = value.trim();
        let value = match raw_value.find('#') {
            Some(pos) => raw_value[..pos].trim().to_string(),
            None => raw_value.to_string(),
        };

        map.insert(key, value);
    }
    Ok(map)
}

pub fn load_config_file(path: &str) -> Result<HashMap<String, String>, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::FileRead {
        path: path.to_string(),
        source: e,
    })?;

    if path.ends_with(".toml") {
        #[cfg(feature = "toml")]
        {
            let value: toml::Value =
                toml::from_str(&content).map_err(|e| ConfigError::FileParse {
                    line: 0,
                    message: e.to_string(),
                })?;
            return toml_value_to_hashmap(value);
        }
        #[cfg(not(feature = "toml"))]
        return Err(ConfigError::FileParse {
            line: 0,
            message: "TOML support not enabled; rebuild with --features toml".into(),
        });
    }

    if path.ends_with(".yaml") || path.ends_with(".yml") {
        #[cfg(feature = "yaml")]
        {
            let value: serde_yaml::Value =
                serde_yaml::from_str(&content).map_err(|e| ConfigError::FileParse {
                    line: 0,
                    message: e.to_string(),
                })?;
            return yaml_value_to_hashmap(value);
        }
        #[cfg(not(feature = "yaml"))]
        return Err(ConfigError::FileParse {
            line: 0,
            message: "YAML support not enabled; rebuild with --features yaml".into(),
        });
    }

    parse_config_file(&content)
}

#[cfg(feature = "toml")]
fn toml_value_to_hashmap(value: toml::Value) -> Result<HashMap<String, String>, ConfigError> {
    let table = value.as_table().ok_or_else(|| ConfigError::FileParse {
        line: 0,
        message: "TOML content must be a table".into(),
    })?;
    let mut map = HashMap::new();
    for (k, v) in table {
        if let Some(s) = v.as_str() {
            map.insert(k.clone(), s.to_string());
        } else {
            map.insert(k.clone(), v.to_string());
        }
    }
    Ok(map)
}

#[cfg(feature = "yaml")]
fn yaml_value_to_hashmap(value: serde_yaml::Value) -> Result<HashMap<String, String>, ConfigError> {
    let mapping = value.as_mapping().ok_or_else(|| ConfigError::FileParse {
        line: 0,
        message: "YAML content must be a mapping".into(),
    })?;
    let mut map = HashMap::new();
    for (k, v) in mapping {
        let key = k.as_str().unwrap_or("").to_string();
        map.insert(key, yaml_value_to_string(v));
    }
    Ok(map)
}

#[cfg(feature = "yaml")]
fn yaml_value_to_string(v: &serde_yaml::Value) -> String {
    use serde_yaml::Value;
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => format!("{v:?}"),
    }
}
