// This file is a **demonstration** of the configuration loader.
#![allow(dead_code)]

use unified_config_loader::{Config, ConfigLoader, ValueSource};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "DB_", file_path = "files/database.ini")]
struct DatabaseConfig {
    #[config(required)]
    url: String,
    #[config(default = "postgres")]
    driver: String,
    #[config(default = "10")]
    max_connections: u32,
}

fn main() {
    // Set environment variable to override the `url` field
    unsafe {
        std::env::set_var("DB_URL", "postgres://override:5432/mydb");
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/database.ini", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    match DatabaseConfig::load() {
        Ok(config) => println!("Database config (with env override): {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
