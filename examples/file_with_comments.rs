use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct ServerConfig {
    #[default = "127.0.0.1"]
    host: String,
    #[default = "3000"]
    port: u16,
    #[default = "info"]
    log_level: String,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/with_comments.env", manifest_dir);

    unsafe {
        std::env::set_var("CONFIG_FILE", &config_path);
    }
    let config = ServerConfig::load().unwrap();
    println!("Cleaned configuration:");
    println!("  host:      {}", config.host);
    println!("  port:      {}", config.port);
    println!("  log_level: {}", config.log_level);
}
