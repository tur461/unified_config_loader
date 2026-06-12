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
#[config(env_prefix = "MYAPP_", file_path = "files/app.toml")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(required)]
    api_key: String,
    #[config(default = "30")]
    timeout: u64,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/app.toml", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    match AppConfig::load() {
        Ok(config) => println!("Loaded TOML config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
