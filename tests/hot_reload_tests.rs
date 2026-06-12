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

use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::hot_reload::ReloadableConfig;

fn temp_file(content: &str, name: &str) -> PathBuf {
    let dir = env::temp_dir();
    let path = dir.join(format!("{}_{}", std::process::id(), name));
    fs::write(&path, content).unwrap();
    path
}

fn cleanup_env() {
    unsafe {
        env::remove_var("APP_CONFIG_FILE");
        env::remove_var("TEST_VALUE");
        env::remove_var("TEST_NUMBER");
    }
}

#[derive(ConfigLoader, Debug, Clone, PartialEq)]
#[config(env_prefix = "TEST_")]
struct HotReloadConfig {
    #[config(default = "default")]
    value: String,
    #[config(default = 42)]
    number: u32,
}

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
    thread::sleep(Duration::from_millis(500));

    {
        assert_eq!(handle.get().value, "initial");
        assert_eq!(handle.get().number, 42); // default
    }
    // Change the file
    fs::write(&path, r#"value = "updated""#).unwrap();
    thread::sleep(Duration::from_millis(500));

    {
        assert_eq!(handle.get().value, "updated");
        // number should still be default (not provided in file)
        assert_eq!(handle.get().number, 42);
    }
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
    thread::sleep(Duration::from_millis(500));
    {
        assert_eq!(handle.get().value, "good");
    }

    // Write invalid TOML
    fs::write(&path, r#"value = "bad" number = oops"#).unwrap();
    thread::sleep(Duration::from_millis(500));

    {
        assert_eq!(handle.get().value, "good");
    }

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

    thread::sleep(Duration::from_millis(500));
    {
        assert_eq!(handle1.get().value, "first");
        assert_eq!(handle2.get().value, "first");
    }
    fs::write(&path, r#"value = "second""#).unwrap();
    thread::sleep(Duration::from_millis(500));

    {
        assert_eq!(handle1.get().value, "second");
        assert_eq!(handle2.get().value, "second");
    }

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
    thread::sleep(Duration::from_millis(500));

    {
        let cloned = handle.get_cloned();
        assert_eq!(cloned.value, "clone_test");
        assert_eq!(cloned.number, 42);
    }
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
    thread::sleep(Duration::from_millis(500));

    {
        assert_eq!(handle.get().value, "env_value");
    }
    // Change the file – env still overrides
    fs::write(&path, r#"value = "new_file_value""#).unwrap();
    thread::sleep(Duration::from_millis(200));

    {
        assert_eq!(handle.get().value, "env_value");
    }
    // Remove env var – still it will remain as the feature is not implemented yet
    unsafe {
        env::remove_var("TEST_VALUE");
    }
    thread::sleep(Duration::from_millis(500));

    {
        assert_eq!(handle.get().value, "env_value");
    }
    cleanup_env();
    let _ = fs::remove_file(path);
}
