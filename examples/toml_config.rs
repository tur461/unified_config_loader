// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use std::env;
use unified_config_loader::ValueSource;
use unified_config_loader::{ConfigLoader, traits::Config};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/config.toml")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default = "3000")]
    port: u16,
    #[config(required)]
    api_key: String,
    log_level: String,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/config.toml", manifest_dir);
    unsafe {
        // Set CONFIG_FILE to a TOML file
        env::set_var("APP_CONFIG_FILE", &config_path);
        // Optionally override via environment variables
        env::set_var("PORT", "8080");
    }
    match AppConfig::load() {
        Ok(config) => println!("Loaded TOML config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
