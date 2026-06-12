// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use std::env;
use unified_config_loader::ValueSource;
use unified_config_loader::{Config, ConfigLoader};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "DEMO_")]
struct DemoConfig {
    #[config(default = "default_value")]
    setting: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Precedence Demo ===");

    // 1. Only default (no file, no env)
    unsafe {
        env::remove_var("DEMO_SETTING");
    }
    let cfg = DemoConfig::load()?;
    println!("Only default:    setting = {}", cfg.setting);

    // 2. Create a config.toml with a different value
    std::fs::write("config.toml", "setting = \"file_value\"")?;
    let cfg = DemoConfig::load()?;
    println!("With file only:   setting = {}", cfg.setting);

    // 3. Set environment variable (highest precedence)
    unsafe {
        env::set_var("DEMO_SETTING", "env_value");
    }
    let cfg = DemoConfig::load()?;
    println!("With file + env:  setting = {}", cfg.setting);

    // Clean up
    std::fs::remove_file("config.toml")?;
    unsafe {
        env::remove_var("DEMO_SETTING");
    }
    Ok(())
}
