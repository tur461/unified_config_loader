use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "localhost"]
    host: String,
    #[required]
    api_key: String,
    #[default = "30"]
    timeout: u64,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/app.toml", manifest_dir);
    unsafe {
        std::env::set_var("CONFIG_FILE", &config_path);
    }

    match AppConfig::load() {
        Ok(config) => println!("Loaded TOML config: {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
