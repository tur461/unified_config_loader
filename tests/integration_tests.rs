// Integration tests for the unified_config_loader.
// These tests intentionally use `unwrap()` and expect panic on failure.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::print_stderr,
    dead_code,
    unused_imports
)]

use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use unified_config_loader::traits::ValidatePartial;
use unified_config_loader::{
    ConfigLoader, errors::ConfigError, models::ValueSource, traits::Config, traits::Validate,
};

const TEST_CONFIG_FILE_PATH: &str = "files/test_config.toml";

// -----------------------------------------------------------------------------
// Test helpers
// -----------------------------------------------------------------------------

fn temp_file(content: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join(TEST_CONFIG_FILE_PATH);
    let mut f = fs::File::create(&path).expect("failed to create temp file");
    f.write_all(content.as_bytes())
        .expect("failed to write temp file");
    path
}

fn temp_file_with_ext(content: &str, ext: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join(format!("files/test_config.{ext}"));
    let mut f = fs::File::create(&path).expect("failed to create temp file with given extension");
    f.write_all(content.as_bytes())
        .expect("failed to write temp file");
    path
}

/// Remove a temporary file if it exists.
fn cleanup_temp_file(path: &Path) {
    let _ = fs::remove_file(path);
}

/// Reset environment variables to a clean state.
fn cleanup_env() {
    let vars = [
        "APP_CONFIG_FILE",
        "NAME",
        "PORT",
        "LOG_LEVEL",
        "API_KEY",
        "TIMEOUT",
        "LEVEL",
        "TIMEOUT_FN",
        "REQUIRED_WITH_FN",
        "SCHEMA_PORT",
        "MYAPP_NAME",
        "MYAPP_PORT",
        "MYAPP_LOG_LEVEL",
        "MYAPP_API_KEY",
        "MYAPP_TIMEOUT",
    ];
    for var in vars {
        unsafe { env::remove_var(var) };
    }
}

// -----------------------------------------------------------------------------
// Test helper functions used as `default_fn`
// -----------------------------------------------------------------------------

fn get_dynamic_port() -> Result<u64, ConfigError> {
    Ok(9999u64)
}

fn get_timeout() -> Result<u64, ConfigError> {
    Ok(9999u64)
}

fn get_dynamic_error() -> Result<String, ConfigError> {
    Err(ConfigError::ValidationError(
        "dynamic default failed".into(),
    ))
}

// -----------------------------------------------------------------------------
// Test configuration structs
// -----------------------------------------------------------------------------

#[derive(ConfigLoader, Debug, PartialEq)]
#[config(env_prefix = "MYAPP_", file_path = "files/test_config.env")]
struct TestConfig {
    #[config(default = "default_name")]
    name: String,
    #[config(default_fn = "get_dynamic_port")]
    port: u64,
    #[config(default = "info")]
    log_level: String,
    #[config(required)]
    api_key: String,
    #[config(default_fn = "get_timeout")]
    timeout: u64,
}

impl ValidatePartial for TestConfig {
    fn validate_partial(&self) -> Vec<ConfigError> {
        let mut errors = vec![];
        if self.port == 0 {
            errors.push(ConfigError::ValidationError("port cannot be zero".into()));
        }
        if self.log_level != "debug" && self.log_level != "info" {
            errors.push(ConfigError::ValidationError("invalid log level".into()));
        }
        errors
    }
}

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct ValidatedConfig {
    #[config(default = "info")]
    level: String,
}

impl Validate for ValidatedConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        match self.level.as_str() {
            "info" | "debug" | "warn" | "error" => Ok(()),
            _ => Err(ConfigError::ValidationError("invalid level".into())),
        }
    }
}

#[derive(ConfigLoader, Debug)]
#[allow(dead_code)]
struct FailingDefaultConfig {
    #[config(default_fn = "get_dynamic_error")]
    #[config(required)]
    api_key: String,
}

#[derive(ConfigLoader, Debug)]
#[allow(dead_code)]
struct BadDefaultConfig {
    #[config(default = "not_a_number")]
    value: u32,
    #[config(required)]
    key: String,
}

// -----------------------------------------------------------------------------
// Tests - each test cleans up before and after
// -----------------------------------------------------------------------------

#[test]
fn defaults_only_with_required_missing() {
    cleanup_env();
    let err = TestConfig::load().unwrap_err();
    assert!(matches!(err, ConfigError::MissingRequiredField { .. }));
    assert!(err.to_string().contains("api_key"));
    cleanup_env();
}

#[test]
fn env_override_defaults() {
    cleanup_env();
    unsafe {
        env::set_var("MYAPP_API_KEY", "env_key");
        env::set_var("MYAPP_PORT", "9090");
    }
    let config = TestConfig::load().unwrap();
    assert_eq!(config.port, 9090);
    assert_eq!(config.api_key, "env_key");
    assert_eq!(config.name, "default_name");
    assert_eq!(config.log_level, "info");
    assert_eq!(config.timeout, 9999);
    cleanup_env();
}

#[test]
fn file_override_env() {
    cleanup_env();
    let content = r#"
port = 7070
api_key = "file_key"
name = "custom_name"
timeout = 42
"#;
    let path = temp_file(content);
    unsafe {
        env::set_var("APP_CONFIG_FILE", TEST_CONFIG_FILE_PATH);
        env::set_var("MYAPP_PORT", "9090");
        env::set_var("MYAPP_API_KEY", "env_key");
    }
    let config = TestConfig::load().unwrap();
    assert_eq!(config.port, 9090);
    assert_eq!(config.api_key, "env_key");
    assert_eq!(config.name, "custom_name");
    assert_eq!(config.timeout, 42);
    cleanup_temp_file(&path);
    cleanup_env();
}

#[test]
fn file_with_comments_and_empty_lines() {
    cleanup_env();
    let content = r#"
# a comment

port = 1234
log_level = "debug"
"#;
    let path = temp_file(content);
    unsafe {
        env::set_var("APP_CONFIG_FILE", TEST_CONFIG_FILE_PATH);
        env::set_var("MYAPP_API_KEY", "key");
    }
    let config = TestConfig::load().unwrap();
    assert_eq!(config.port, 1234);
    assert_eq!(config.log_level, "debug");
    cleanup_temp_file(&path);
    cleanup_env();
}

#[test]
fn invalid_env_value_errors() {
    cleanup_env();
    unsafe {
        env::set_var("MYAPP_API_KEY", "ok");
        env::set_var("MYAPP_PORT", "not_a_number");
    }
    let err = TestConfig::load().unwrap_err();
    match err {
        ConfigError::InvalidValue {
            field,
            value,
            value_source,
        } => {
            assert_eq!(field, "port");
            assert_eq!(value, "not_a_number");
            assert!(matches!(value_source, ValueSource::Environment));
        }
        _ => panic!("unexpected error: {:?}", err),
    }
    cleanup_env();
}

#[test]
fn invalid_file_value_errors() {
    cleanup_env();
    let content = r#"
api_key = "ok"
port = "abc"
"#;
    let path = temp_file(content);
    unsafe { env::set_var("APP_CONFIG_FILE", TEST_CONFIG_FILE_PATH) };
    let err = TestConfig::load().unwrap_err();
    match err {
        ConfigError::InvalidValue {
            field,
            value,
            value_source,
        } => {
            assert_eq!(field, "port");
            assert_eq!(value, "abc");
            assert!(matches!(value_source, ValueSource::TomlFile));
        }
        _ => panic!("unexpected error: {:?}", err),
    }
    cleanup_temp_file(&path);
    cleanup_env();
}

#[test]
fn missing_file_gracefully_ignored() {
    cleanup_env();
    unsafe {
        env::set_var("MYAPP_API_KEY", "key");
        env::set_var("APP_CONFIG_FILE", "/nonexistent/path/to/file");
    }
    let config = TestConfig::load().unwrap();
    assert_eq!(config.api_key, "key");
    assert_eq!(config.name, "default_name");
    assert_eq!(config.log_level, "info");
    assert_eq!(config.timeout, 9999);
    assert_eq!(config.port, 9999);
    cleanup_env();
}

#[test]
fn validate_trait_can_be_used() {
    cleanup_env();
    let config = ValidatedConfig::load().unwrap();
    assert!(config.validate().is_ok());

    unsafe { env::set_var("MYAPP_LEVEL", "bogus") };
    let config = ValidatedConfig::load().unwrap();
    assert!(config.validate().is_err());
    cleanup_env();
}

#[test]
fn default_fn_fallback() {
    cleanup_env();
    unsafe { env::set_var("MYAPP_API_KEY", "key") };
    let config = TestConfig::load().unwrap();
    assert_eq!(config.port, 9999);
    assert_eq!(config.timeout, 9999);
    cleanup_env();
}

#[test]
fn default_fn_ignored_when_env_set() {
    cleanup_env();
    unsafe {
        env::set_var("MYAPP_API_KEY", "key");
        env::set_var("MYAPP_PORT", "1234");
    }
    let config = TestConfig::load().unwrap();
    assert_eq!(config.port, 1234);
    cleanup_env();
}

#[test]
fn default_fn_error_propagates() {
    cleanup_env();
    let err = FailingDefaultConfig::load().unwrap_err();
    assert!(matches!(err, ConfigError::ValidationError(_)));
    assert!(err.to_string().contains("dynamic default failed"));
    cleanup_env();
}

#[test]
fn required_field_with_default_fn_succeeds() {
    cleanup_env();
    #[derive(ConfigLoader, Debug)]
    struct RequiredWithDefaultFn {
        #[config(required)]
        #[config(default_fn = "get_dynamic_port")]
        value: u64,
    }
    let config = RequiredWithDefaultFn::load().unwrap();
    assert_eq!(config.value, 9999);
    cleanup_env();
}

#[test]
fn validate_partial_collects_errors() {
    cleanup_env();
    unsafe {
        env::set_var("MYAPP_API_KEY", "key");
        env::set_var("MYAPP_PORT", "0");
        env::set_var("MYAPP_LOG_LEVEL", "trace");
    }
    let config = TestConfig::load().unwrap();
    let errors = config.validate_partial();
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().any(|e| e.to_string().contains("port")));
    assert!(errors.iter().any(|e| e.to_string().contains("log level")));
    cleanup_env();
}

#[test]
fn schema_contains_field_info() {
    cleanup_env();
    let schema = TestConfig::schema();
    assert!(schema.contains("\"name\""));
    assert!(schema.contains("\"port\""));
    assert!(schema.contains("\"log_level\""));
    assert!(schema.contains("\"api_key\""));
    assert!(schema.contains("\"timeout\""));
    assert!(schema.contains("\"type\": \"string\""));
    assert!(schema.contains("\"type\": \"integer\""));
    assert!(schema.contains("\"required\": true"));
    assert!(schema.contains("\"required\": false"));
    cleanup_env();
}

#[test]
fn file_with_inline_comments() {
    cleanup_env();
    let content = r#"
name = "app" # my app
port = 42 # the answer
api_key = "secret" # key
"#;
    let path = temp_file(content);
    unsafe { env::set_var("APP_CONFIG_FILE", TEST_CONFIG_FILE_PATH) };
    let config = TestConfig::load().unwrap();
    assert_eq!(config.name, "app");
    assert_eq!(config.port, 42);
    assert_eq!(config.api_key, "secret");
    cleanup_temp_file(&path);
    cleanup_env();
}

#[test]
fn duplicate_keys_in_file() {
    cleanup_env();
    let content = r#"
port = 111
port = 222
api_key = "key"
"#;
    let path = temp_file_with_ext(content, "env");
    unsafe { env::set_var("APP_CONFIG_FILE", "files/test_config.env") };
    let config = TestConfig::load().unwrap();
    assert_eq!(config.port, 222);
    cleanup_temp_file(&path);
    cleanup_env();
}

#[test]
fn bad_default_literal_causes_error() {
    cleanup_env();
    unsafe { env::set_var("MYAPP_KEY", "x") };
    let err = BadDefaultConfig::load().unwrap_err();
    assert!(matches!(err, ConfigError::InvalidValue { .. }));
    assert!(err.to_string().contains("value"));
    cleanup_env();
}

#[test]
fn multiple_configs_independent() {
    cleanup_env();
    #[derive(ConfigLoader, Debug)]
    #[config(env_prefix = "SERVER_")]
    struct ServerCfg {
        #[config(default = "0.0.0.0")]
        bind: String,
        #[config(default = "80")]
        port: u16,
    }
    let server = ServerCfg::load().unwrap();
    assert_eq!(server.bind, "0.0.0.0");
    assert_eq!(server.port, 80);

    unsafe { env::set_var("MYAPP_API_KEY", "xyz") };
    let test = TestConfig::load().unwrap();
    assert_eq!(test.api_key, "xyz");
    cleanup_env();
}

// -----------------------------------------------------------------------------
// Additional format tests (feature‑gated)
// -----------------------------------------------------------------------------

#[cfg(feature = "toml")]
#[test]
fn load_toml_file() {
    use unified_config_loader::load_config_file;
    cleanup_env();
    let content = r#"
string = "hello"
number = 42
"#;
    let path = temp_file(content);
    let map = load_config_file(path.to_str().unwrap()).unwrap();
    assert_eq!(
        map.get("string"),
        Some(&("hello".to_string(), ValueSource::TomlFile))
    );
    assert_eq!(
        map.get("number"),
        Some(&("42".to_string(), ValueSource::TomlFile))
    );
    cleanup_temp_file(&path);
    cleanup_env();
}

#[cfg(feature = "json")]
#[test]
fn load_json_file() {
    use unified_config_loader::load_config_file;
    cleanup_env();
    let content = r#"{"string": "hello", "number": 42}"#;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test.json");
    fs::write(&path, content).unwrap();
    let map = load_config_file(path.to_str().unwrap()).unwrap();
    assert_eq!(
        map.get("string"),
        Some(&("hello".to_string(), ValueSource::JsonFile))
    );
    assert_eq!(
        map.get("number"),
        Some(&("42".to_string(), ValueSource::JsonFile))
    );
    cleanup_temp_file(&path);
    cleanup_env();
}

#[cfg(feature = "yaml")]
#[test]
fn load_yaml_file() {
    use unified_config_loader::load_config_file;
    cleanup_env();
    let content = r#"
string: hello
number: 42
"#;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test.yaml");
    fs::write(&path, content).unwrap();
    let map = load_config_file(path.to_str().unwrap()).unwrap();
    assert_eq!(
        map.get("string"),
        Some(&("hello".to_string(), ValueSource::YamlFile))
    );
    assert_eq!(
        map.get("number"),
        Some(&("42".to_string(), ValueSource::YamlFile))
    );
    cleanup_temp_file(&path);
    cleanup_env();
}

#[cfg(feature = "ini")]
#[test]
fn load_ini_file() {
    use unified_config_loader::load_config_file;
    cleanup_env();
    let content = r#"
[DEFAULT]
string = hello
number = 42
"#;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test.ini");
    fs::write(&path, content).unwrap();
    let map = load_config_file(path.to_str().unwrap()).unwrap();
    assert_eq!(
        map.get("DEFAULT.string"),
        Some(&("hello".to_string(), ValueSource::IniFile))
    );
    assert_eq!(
        map.get("DEFAULT.number"),
        Some(&("42".to_string(), ValueSource::IniFile))
    );
    cleanup_temp_file(&path);
    cleanup_env();
}
