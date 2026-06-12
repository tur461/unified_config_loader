// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[config(required)]
    secret_key: String,
    #[config(default = "8080")]
    port: u16,
}

fn main() {
    // Not setting secret_key will cause MissingRequiredField error.
    match AppConfig::load() {
        Ok(config) => println!("Config loaded: {:?}", config),
        Err(e) => {
            eprintln!("Configuration error: {e}");
            // You can match on error kind if you wish
            if let unified_config_loader::ConfigError::MissingRequiredField { field } = &e {
                eprintln!(
                    "Hint: set environment variable {}=...",
                    field.to_uppercase()
                );
            }
            std::process::exit(1);
        }
    }
}
