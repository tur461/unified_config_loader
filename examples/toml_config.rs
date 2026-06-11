use std::env;
use unified_config_loader::{ConfigLoader, traits::Config};

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "localhost"]
    host: String,
    #[default = "3000"]
    port: u16,
    #[required]
    api_key: String,
    log_level: String,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/config.toml", manifest_dir);
    unsafe {
        // Set CONFIG_FILE to a TOML file
        env::set_var("CONFIG_FILE", &config_path);
        // Optionally override via environment variables
        env::set_var("PORT", "8080");
    }
    match AppConfig::load() {
        Ok(config) => println!("Loaded TOML config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
