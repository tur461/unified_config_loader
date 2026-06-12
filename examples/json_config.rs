// This file is a **demonstration** of the configuration loader.
#![allow(dead_code)]

use unified_config_loader::{Config, ConfigLoader, ValueSource};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "APP_", file_path = "files/app.json")]
struct AppConfig {
    #[config(default = "development")]
    environment: String,
    #[config(required)]
    secret_key: String,
    #[config(default = "8080")]
    port: u16,
    #[config(default = "false")]
    enable_debug: bool,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/app.json", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    match AppConfig::load() {
        Ok(config) => println!("App config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
