// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::errors::ConfigError;
use unified_config_loader::traits::Config;

fn get_default_port() -> Result<u16, ConfigError> {
    Ok(3000)
}

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default_fn = "get_default_port")]
    port: u16,
}

fn main() {
    let config = AppConfig::load().unwrap();
    println!("Dynamic default port: {}", config.port);
}
