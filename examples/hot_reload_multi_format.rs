// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use std::thread;
use std::time::Duration;
use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::hot_reload::ReloadableConfig;

#[derive(ConfigLoader, Debug, Clone)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[config(default = "default")]
    value: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write("config_temp.toml", "value = \"from_toml\"")?;
    std::fs::write("config_temp.yaml", "value: \"from_yaml\"")?;

    let config_handle = ReloadableConfig::<AppConfig>::load()?;

    println!("Both config.toml and config.json exist.");
    println!("Conventional order: toml is loaded before json, so json overrides toml.\n");
    println!("Expected value: 'from_json'\n");

    thread::spawn(move || {
        loop {
            let cfg = config_handle.get();
            println!("Current value: {}", cfg.value);
            drop(cfg);
            thread::sleep(Duration::from_secs(5));
        }
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
