// This file is a **demonstration** of the configuration loader.
#![allow(dead_code)]

use unified_config_loader::{Config, ConfigLoader, ValueSource};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "API_", file_path = "files/api.json")]
struct ApiConfig {
    #[config(default = "https://api.example.com")]
    base_url: String,
    #[config(required)]
    api_key: String,
    #[config(default = "30")]
    timeout_secs: u64,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/api.json", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    match ApiConfig::load() {
        Ok(config) => println!("Loaded JSON config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
