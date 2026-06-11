use std::env;
use std::fs;
use std::io::Write;
use std::sync::Mutex;

use unified_config_loader::traits::ValidatePartial;
use unified_config_loader::{
    ConfigLoader, errors::ConfigError, models::ValueSource, traits::Config, traits::Validate,
};

static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn run_locked(test: impl FnOnce()) {
    let _guard = ENV_MUTEX.lock().unwrap();
    cleanup_env();
    test();
    cleanup_env();
}

fn temp_file(content: &str) -> String {
    let dir = env::temp_dir();
    let path = dir.join(format!("test_config_{}.env", std::process::id()));
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path.to_str().unwrap().to_string()
}

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

fn cleanup_env() {
    unsafe {
        let _ = env::remove_var("CONFIG_FILE");
        let _ = env::remove_var("NAME");
        let _ = env::remove_var("PORT");
        let _ = env::remove_var("LOG_LEVEL");
        let _ = env::remove_var("API_KEY");
        let _ = env::remove_var("TIMEOUT");
        let _ = env::remove_var("LEVEL");
        let _ = env::remove_var("TIMEOUT_FN");
        let _ = env::remove_var("REQUIRED_WITH_FN");
        let _ = env::remove_var("SCHEMA_PORT");
    }
}

#[derive(ConfigLoader, Debug, PartialEq)]
struct TestConfig {
    #[default = "default_name"]
    name: String,
    #[default_fn = "get_dynamic_port"]
    port: u64,
    #[default = "info"]
    log_level: String,
    #[required]
    api_key: String,
    #[default_fn = "get_timeout"]
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

#[test]
fn defaults_only_with_required_missing() {
    run_locked(|| {
        let err = TestConfig::load().unwrap_err();
        assert!(matches!(err, ConfigError::MissingRequiredField { .. }));
        assert!(err.to_string().contains("api_key"));
    });
}

#[test]
fn env_override_defaults() {
    run_locked(|| {
        unsafe {
            env::set_var("API_KEY", "env_key");
            env::set_var("PORT", "9090");
        }
        let config = TestConfig::load().unwrap();
        assert_eq!(config.port, 9090);
        assert_eq!(config.api_key, "env_key");
        assert_eq!(config.name, "default_name");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.timeout, 9999);
    });
}

#[test]
fn file_override_env() {
    run_locked(|| {
        let content = "port=7070\napi_key=file_key\nname=custom_name\ntimeout=42\n";
        let path = temp_file(content);
        unsafe {
            env::set_var("CONFIG_FILE", &path);
            env::set_var("PORT", "9090");
            env::set_var("API_KEY", "env_key");
        }
        let config = TestConfig::load().unwrap();
        println!("{:?}", config);
        assert_eq!(config.port, 7070);
        assert_eq!(config.api_key, "file_key");
        assert_eq!(config.name, "custom_name");
        assert_eq!(config.timeout, 42);
        let _ = fs::remove_file(&path);
    });
}

#[test]
fn file_with_comments_and_empty_lines() {
    run_locked(|| {
        let content = "# a comment\n\nport=1234\n  log_level = debug  \n";
        let path = temp_file(content);
        unsafe {
            env::set_var("CONFIG_FILE", &path);
            env::set_var("API_KEY", "key");
        }
        let config = TestConfig::load().unwrap();
        assert_eq!(config.port, 1234);
        assert_eq!(config.log_level, "debug");
        let _ = fs::remove_file(&path);
    });
}

#[test]
fn invalid_env_value_errors() {
    run_locked(|| {
        unsafe {
            env::set_var("API_KEY", "ok");
            env::set_var("PORT", "not_a_number");
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
            _ => panic!("unexpected error"),
        }
    });
}

#[test]
fn invalid_file_value_errors() {
    run_locked(|| {
        let content = "api_key=ok\nport=abc\n";
        let path = temp_file(content);
        unsafe {
            env::set_var("CONFIG_FILE", &path);
        }
        let err = TestConfig::load().unwrap_err();
        match err {
            ConfigError::InvalidValue {
                field,
                value,
                value_source,
            } => {
                assert_eq!(field, "port");
                assert_eq!(value, "abc");
                assert!(matches!(value_source, ValueSource::File));
            }
            _ => panic!("unexpected error"),
        }
        let _ = fs::remove_file(&path);
    });
}

#[test]
fn missing_file_gracefully_ignored() {
    run_locked(|| {
        unsafe {
            env::set_var("API_KEY", "key");
            env::set_var("CONFIG_FILE", "/nonexistent/path");
        }
        let err = TestConfig::load().unwrap_err();
        assert!(matches!(err, ConfigError::FileRead { .. }));
    });
}

#[derive(ConfigLoader, Debug)]
struct ValidatedConfig {
    #[default = "info"]
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

#[test]
fn validate_trait_can_be_used() {
    run_locked(|| {
        let config = ValidatedConfig::load().unwrap();
        assert!(config.validate().is_ok());

        unsafe {
            env::set_var("LEVEL", "bogus");
        }
        let config = ValidatedConfig::load().unwrap();
        assert!(config.validate().is_err());
    });
}

#[test]
fn default_fn_fallback() {
    run_locked(|| {
        unsafe {
            env::set_var("API_KEY", "key");
        }
        let config = TestConfig::load().unwrap();
        assert_eq!(config.port, 9999);
        assert_eq!(config.timeout, 9999);
    });
}

#[test]
fn default_fn_ignored_when_env_set() {
    run_locked(|| {
        unsafe {
            env::set_var("API_KEY", "key");
            env::set_var("PORT", "1234");
        }
        let config = TestConfig::load().unwrap();
        assert_eq!(config.port, 1234);
    });
}

#[derive(ConfigLoader, Debug)]
struct FailingDefaultConfig {
    #[default_fn = "get_dynamic_error"]
    #[required]
    api_key: String,
}

#[test]
fn default_fn_error_propagates() {
    run_locked(|| {
        let err = FailingDefaultConfig::load().unwrap_err();
        assert!(matches!(err, ConfigError::ValidationError(_)));
        assert!(err.to_string().contains("dynamic default failed"));
    });
}

#[test]
fn required_field_with_default_fn_succeeds() {
    run_locked(|| {
        #[derive(ConfigLoader, Debug)]
        struct RequiredWithDefaultFn {
            #[required]
            #[default_fn = "get_dynamic_port"]
            value: u64,
        }
        let config = RequiredWithDefaultFn::load().unwrap();
        assert_eq!(config.value, 9999);
    });
}
#[test]
fn validate_partial_collects_errors() {
    run_locked(|| {
        unsafe {
            env::set_var("API_KEY", "key");
            env::set_var("PORT", "0"); // invalid
            env::set_var("LOG_LEVEL", "trace"); // invalid
        }
        let config = TestConfig::load().unwrap();
        let errors = config.validate_partial();
        assert_eq!(errors.len(), 2);
        assert!(errors.iter().any(|e| e.to_string().contains("port")));
        assert!(errors.iter().any(|e| e.to_string().contains("log level")));
    });
}

#[test]
fn schema_contains_field_info() {
    let schema = TestConfig::schema();
    assert!(schema.contains("\"name\""));
    assert!(schema.contains("\"port\""));
    assert!(schema.contains("\"log_level\""));
    assert!(schema.contains("\"api_key\""));
    assert!(schema.contains("\"timeout\""));

    assert!(schema.contains("\"type\": \"string\""));
    assert!(schema.contains("\"type\": \"integer\""));

    assert!(schema.contains("\"required\": true")); // for api_key
    assert!(schema.contains("\"required\": false")); // for others
}

#[test]
fn file_with_inline_comments() {
    run_locked(|| {
        let content = "name = app # my app\nport=42 # the answer\napi_key=secret # key\n";
        let path = temp_file(content);
        unsafe {
            env::set_var("CONFIG_FILE", &path);
        }
        let config = TestConfig::load().unwrap();
        assert_eq!(config.name, "app");
        assert_eq!(config.port, 42);
        assert_eq!(config.api_key, "secret");
        let _ = fs::remove_file(&path);
    });
}

#[test]
fn duplicate_keys_in_file() {
    run_locked(|| {
        let content = "port=111\nport=222\napi_key=key\n";
        let path = temp_file(content);
        unsafe {
            env::set_var("CONFIG_FILE", &path);
        }
        let config = TestConfig::load().unwrap();
        assert_eq!(config.port, 222);
        let _ = fs::remove_file(&path);
    });
}

#[derive(ConfigLoader, Debug)]
struct BadDefaultConfig {
    #[default = "not_a_number"]
    value: u32,
    #[required]
    key: String,
}

#[test]
fn bad_default_literal_causes_error() {
    run_locked(|| {
        unsafe {
            env::set_var("KEY", "x");
        }
        let err = BadDefaultConfig::load().unwrap_err();
        assert!(matches!(err, ConfigError::InvalidValue { .. }));
        assert!(err.to_string().contains("value"));
    });
}

#[test]
fn multiple_configs_independent() {
    run_locked(|| {
        #[derive(ConfigLoader, Debug)]
        struct ServerCfg {
            #[default = "0.0.0.0"]
            bind: String,
            #[default = "80"]
            port: u16,
        }
        let server = ServerCfg::load().unwrap();
        assert_eq!(server.bind, "0.0.0.0");
        assert_eq!(server.port, 80);

        unsafe {
            env::set_var("API_KEY", "xyz");
        }
        let test = TestConfig::load().unwrap();
        assert_eq!(test.api_key, "xyz");
    });
}
