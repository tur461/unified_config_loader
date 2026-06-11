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
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/basic.env", manifest_dir);

    unsafe {
        std::env::set_var("CONFIG_FILE", &config_path);
    }

    let config = AppConfig::load().unwrap();
    println!("Configuration loaded from file:");
    println!("  host:      {}", config.host);
    println!("  port:      {}", config.port);
    println!("  log_level: {}", config.log_level);
}
