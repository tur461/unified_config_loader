// This file is a **demonstration** of the configuration loader.
#![allow(dead_code)]

use unified_config_loader::{Config, ConfigLoader, ValueSource};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "SERVER_", file_path = "files/server.ini")]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    host: String,
    #[config(required)]
    port: u16,
    #[config(default = "./logs")]
    log_dir: String,
    #[config(default = "info")]
    log_level: String,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/server.ini", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    match ServerConfig::load() {
        Ok(config) => println!("Server config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
