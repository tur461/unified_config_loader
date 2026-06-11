use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "localhost"]
    host: String,
    #[required]
    database_url: String,
}

fn main() {
    match AppConfig::load() {
        Ok(config) => println!("Database URL: {}", config.database_url),
        Err(e) => eprintln!("Failed to load config: {e}"),
    }
}
