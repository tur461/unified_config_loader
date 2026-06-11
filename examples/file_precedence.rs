//! Demonstrates precedence: file > environment > defaults.

use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "localhost"]
    host: String,
    #[default = "8080"]
    port: u16,
    #[default = "info"]
    log_level: String,
}

fn main() {
    // Set an environment variable that will be overridden by the file.
    unsafe {
        std::env::set_var("PORT", "9999");
        std::env::set_var("HOST", "env.example.com");
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/production.env", manifest_dir);

    unsafe {
        std::env::set_var("CONFIG_FILE", &config_path);
    }
    let config = AppConfig::load().unwrap();
    println!("Configuration (file > env > default):");
    println!("  host:      {}  (file wins over env)", config.host);
    println!("  port:      {}    (file wins over env)", config.port);
    println!(
        "  log_level: {}  (file overrides default)",
        config.log_level
    );
}
