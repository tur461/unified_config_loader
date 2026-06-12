// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(dead_code)]

use unified_config_loader::{Config, ConfigLoader, ValueSource};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "SERVER_", file_path = "files/app.ini")]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    host: String,
    #[config(required)]
    port: u16,
    #[config(default = "info")]
    log_level: String,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/app.ini", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    match ServerConfig::load() {
        Ok(config) => println!("Loaded INI config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
