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

#[cfg(feature = "ini")]
use ini as rust_ini;
use std::collections::HashMap;

pub fn parse_env_file(
    content: &str,
) -> Result<HashMap<String, (String, ValueSource)>, ConfigError> {
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

        map.insert(key, (value, ValueSource::EnvFile));
    }
    Ok(map)
}

pub fn load_config_file(path: &str) -> Result<HashMap<String, (String, ValueSource)>, ConfigError> {
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

    if path.ends_with(".json") {
        #[cfg(feature = "json")]
        {
            let value: serde_json::Value =
                serde_json::from_str(&content).map_err(|e| ConfigError::FileParse {
                    line: 0,
                    message: e.to_string(),
                })?;
            return json_value_to_hashmap(value);
        }
        #[cfg(not(feature = "json"))]
        return Err(ConfigError::FileParse {
            line: 0,
            message: "JSON support not enabled; rebuild with --features json".into(),
        });
    }

    if path.ends_with(".ini") {
        #[cfg(feature = "ini")]
        {
            let content = std::fs::read_to_string(path).map_err(|e| ConfigError::FileRead {
                path: path.to_string(),
                source: e,
            })?;

            let parsed =
                rust_ini::Ini::load_from_str(&content).map_err(|e| ConfigError::FileParse {
                    line: 0,
                    message: e.to_string(),
                })?;

            return flatten_ini_map(parsed);
        }
        #[cfg(not(feature = "ini"))]
        return Err(ConfigError::FileParse {
            line: 0,
            message: "INI support not enabled; rebuild with --features ini".into(),
        });
    }

    parse_env_file(&content)
}

#[cfg(feature = "toml")]
fn toml_value_to_hashmap(
    value: toml::Value,
) -> Result<HashMap<String, (String, ValueSource)>, ConfigError> {
    let table = value.as_table().ok_or_else(|| ConfigError::FileParse {
        line: 0,
        message: "TOML content must be a table".into(),
    })?;
    let mut map = HashMap::new();
    for (k, v) in table {
        if let Some(s) = v.as_str() {
            map.insert(k.clone(), (s.to_string(), ValueSource::TomlFile));
        } else {
            map.insert(k.clone(), (v.to_string(), ValueSource::TomlFile));
        }
    }
    Ok(map)
}

#[cfg(feature = "yaml")]
fn yaml_value_to_hashmap(
    value: serde_yaml::Value,
) -> Result<HashMap<String, (String, ValueSource)>, ConfigError> {
    let mapping = value.as_mapping().ok_or_else(|| ConfigError::FileParse {
        line: 0,
        message: "YAML content must be a mapping".into(),
    })?;
    let mut map = HashMap::new();
    for (k, v) in mapping {
        let key = k.as_str().unwrap_or("").to_string();
        map.insert(key, (yaml_value_to_string(v), ValueSource::YamlFile));
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

#[cfg(feature = "json")]
fn json_value_to_hashmap(
    value: serde_json::Value,
) -> Result<HashMap<String, (String, ValueSource)>, ConfigError> {
    let obj = value.as_object().ok_or_else(|| ConfigError::FileParse {
        line: 0,
        message: "JSON content must be an object".into(),
    })?;
    let mut map = HashMap::new();
    for (k, v) in obj {
        let s = match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => v.to_string(),
        };
        map.insert(k.clone(), (s, ValueSource::JsonFile));
    }
    Ok(map)
}

#[cfg(feature = "ini")]
fn flatten_ini_map(
    ini: rust_ini::Ini,
) -> Result<HashMap<String, (String, ValueSource)>, ConfigError> {
    let mut map = HashMap::new();
    for (sec, prop) in ini.iter() {
        let section_name = sec.unwrap_or("");
        for (k, v) in prop.iter() {
            let full_key = if section_name.is_empty() {
                k.to_string()
            } else {
                format!("{}.{}", section_name, k)
            };
            map.insert(full_key, (v.to_string(), ValueSource::IniFile));
        }
    }
    Ok(map)
}

pub fn load_env_vars(prefix: &str) -> HashMap<String, (String, ValueSource)> {
    let mut map = HashMap::new();
    for (var_name, var_value) in std::env::vars() {
        if let Some(stripped) = var_name.strip_prefix(prefix) {
            // Remove the prefix and convert to lowercase for case‑insensitive matching
            let key = stripped.to_lowercase();
            map.insert(key, (var_value, ValueSource::Environment));
        }
    }
    map
}
