// Hot reload integration tests.
// These tests require the "hot-reload" feature.
#![cfg(feature = "hot-reload")]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::print_stderr,
    dead_code
)]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::hot_reload::ReloadableConfig;

// -----------------------------------------------------------------------------
// Test helpers
// -----------------------------------------------------------------------------

/// Poll until the config value matches expected, or timeout after 5 seconds.
fn wait_for_config_value<F>(
    handle: &ReloadableConfig<HotReloadConfig>,
    mut get_val: F,
    expected: &str,
) where
    F: FnMut(&HotReloadConfig) -> String,
{
    let start = Instant::now();
    let timeout = Duration::from_secs(5);
    loop {
        let cfg = handle.get();
        if get_val(&cfg) == expected {
            return;
        }
        if start.elapsed() > timeout {
            panic!(
                "Config did not update to '{}' within {:?}",
                expected, timeout
            );
        }
        thread::sleep(Duration::from_millis(100));
    }
}

/// Create a temporary file and return its path.
fn temp_file(content: &str, name: &str) -> PathBuf {
    let dir = env::temp_dir();
    let path = dir.join(format!("{}_{}", std::process::id(), name));
    fs::write(&path, content).unwrap();
    path
}

/// Reset environment.
fn cleanup_env() {
    unsafe {
        env::remove_var("APP_CONFIG_FILE");
        env::remove_var("TEST_VALUE");
        env::remove_var("TEST_NUMBER");
    }
}

// -----------------------------------------------------------------------------
// Test config struct
// -----------------------------------------------------------------------------

#[derive(ConfigLoader, Debug, Clone, PartialEq)]
#[config(env_prefix = "TEST_")]
struct HotReloadConfig {
    #[config(default = "default")]
    value: String,
    #[config(default = 42)]
    number: u32,
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[test]
fn initial_config_is_loaded() {
    cleanup_env();
    let handle = ReloadableConfig::<HotReloadConfig>::load().unwrap();
    let cfg = handle.get();
    assert_eq!(cfg.value, "default");
    assert_eq!(cfg.number, 42);
    cleanup_env();
}

#[cfg(feature = "toml")]
#[test]
fn single_file_mode_reloads_on_change() {
    cleanup_env();
    let path = temp_file(r#"value = "initial""#, "test.toml");

    unsafe {
        env::set_var("APP_CONFIG_FILE", path.to_str().unwrap());
    }
    let handle = ReloadableConfig::<HotReloadConfig>::load().unwrap();
    wait_for_config_value(&handle, |c| c.value.clone(), "initial");
    let initial = handle.get();
    assert_eq!(initial.value, "initial");
    assert_eq!(initial.number, 42); // default

    // Change the file
    fs::write(&path, r#"value = "updated""#).unwrap();
    wait_for_config_value(&handle, |c| c.value.clone(), "updated");
    let final_cfg = handle.get();
    assert_eq!(final_cfg.value, "updated");
    // number should still be default (not provided in file)
    assert_eq!(final_cfg.number, 42);

    cleanup_env();
    let _ = fs::remove_file(path);
}

#[cfg(feature = "toml")]
#[test]
fn invalid_file_does_not_replace_config() {
    cleanup_env();
    let path = temp_file(r#"value = "good""#, "test.toml");
    unsafe {
        env::set_var("APP_CONFIG_FILE", path.to_str().unwrap());
    }

    let handle = ReloadableConfig::<HotReloadConfig>::load().unwrap();
    wait_for_config_value(&handle, |c| c.value.clone(), "good");
    let initial = handle.get();
    assert_eq!(initial.value, "good");

    // Write invalid TOML
    fs::write(&path, r#"value = "bad" number = oops"#).unwrap();
    thread::sleep(Duration::from_millis(500));

    let still_good = handle.get();
    assert_eq!(still_good.value, "good");

    cleanup_env();
    let _ = fs::remove_file(path);
}

#[cfg(feature = "toml")]
#[test]
fn cloned_handles_share_same_config() {
    cleanup_env();
    let path = temp_file(r#"value = "first""#, "test.toml");
    unsafe {
        env::set_var("APP_CONFIG_FILE", path.to_str().unwrap());
    }

    let handle1 = ReloadableConfig::<HotReloadConfig>::load().unwrap();
    let handle2 = handle1.clone();
    wait_for_config_value(&handle1, |c| c.value.clone(), "first");
    assert_eq!(handle1.get().value, "first");
    assert_eq!(handle2.get().value, "first");

    fs::write(&path, r#"value = "second""#).unwrap();
    wait_for_config_value(&handle1, |c| c.value.clone(), "second");

    assert_eq!(handle1.get().value, "second");
    assert_eq!(handle2.get().value, "second");

    cleanup_env();
    let _ = fs::remove_file(path);
}

#[cfg(feature = "toml")]
#[test]
fn get_cloned_returns_a_copy() {
    cleanup_env();
    let path = temp_file(r#"value = "clone_test""#, "test.toml");

    unsafe {
        env::set_var("APP_CONFIG_FILE", path.to_str().unwrap());
    }
    let handle = ReloadableConfig::<HotReloadConfig>::load().unwrap();
    wait_for_config_value(&handle, |c| c.value.clone(), "clone_test");
    let cloned = handle.get_cloned();
    assert_eq!(cloned.value, "clone_test");
    assert_eq!(cloned.number, 42);

    cleanup_env();
    let _ = fs::remove_file(path);
}

#[cfg(feature = "toml")]
#[test]
fn environment_variable_has_highest_precedence_even_during_reload() {
    cleanup_env();
    let path = temp_file(r#"value = "file_value""#, "test.toml");
    unsafe {
        env::set_var("APP_CONFIG_FILE", path.to_str().unwrap());
        env::set_var("TEST_VALUE", "env_value");
    }

    let handle = ReloadableConfig::<HotReloadConfig>::load().unwrap();
    wait_for_config_value(&handle, |c| c.value.clone(), "env_value");
    assert_eq!(handle.get().value, "env_value");

    // Change the file – env still overrides
    fs::write(&path, r#"value = "new_file_value""#).unwrap();
    thread::sleep(Duration::from_millis(200));
    assert_eq!(handle.get().value, "env_value");

    // Remove env var – now file value should appear
    unsafe {
        env::remove_var("TEST_VALUE");
    }
    wait_for_config_value(&handle, |c| c.value.clone(), "new_file_value");
    assert_eq!(handle.get().value, "new_file_value");

    cleanup_env();
    let _ = fs::remove_file(path);
}
