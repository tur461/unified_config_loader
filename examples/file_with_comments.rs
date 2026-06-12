use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/with_comments.env")]
struct ServerConfig {
    #[config(default = "127.0.0.1")]
    host: String,
    #[config(default = "3000")]
    port: u16,
    #[config(default = "info")]
    log_level: String,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/with_comments.env", manifest_dir);

    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }
    let config = ServerConfig::load().unwrap();
    println!("Cleaned configuration:");
    println!("  host:      {}", config.host);
    println!("  port:      {}", config.port);
    println!("  log_level: {}", config.log_level);
}
