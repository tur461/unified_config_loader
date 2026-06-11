use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "localhost"]
    host: String,
    #[default = "8080"]
    port: u16,
}

fn main() {
    let config = AppConfig::load().unwrap();
    println!("Server running at {}:{}", config.host, config.port);
}
