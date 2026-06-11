use unified_config_loader::ConfigLoader;
use unified_config_loader::errors::ConfigError;
use unified_config_loader::traits::Config;

fn get_default_port() -> Result<u16, ConfigError> {
    Ok(3000)
}

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "localhost"]
    host: String,
    #[default_fn = "get_default_port"]
    port: u16,
}

fn main() {
    let config = AppConfig::load().unwrap();
    println!("Dynamic default port: {}", config.port);
}
