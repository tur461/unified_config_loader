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
#[config(env_prefix = "MYAPP_", file_path = "files/basic.env")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default = "8080")]
    port: u16,
    #[config(default = "info")]
    log_level: String,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/basic.env", manifest_dir);

    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    let config = AppConfig::load().unwrap();
    println!("Configuration loaded from file:");
    println!("  host:      {}", config.host);
    println!("  port:      {}", config.port);
    println!("  log_level: {}", config.log_level);
}
