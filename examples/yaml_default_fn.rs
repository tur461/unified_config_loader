use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::errors::ConfigError;
use unified_config_loader::traits::Config;

fn get_dynamic_timeout() -> Result<u64, ConfigError> {
    Ok(42) // dynamic default
}

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/service.yaml")]
struct ServiceConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default_fn = "get_dynamic_timeout")]
    timeout: u64,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/service.yaml", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    let config = ServiceConfig::load().unwrap();
    println!("Host: {}, Timeout: {}", config.host, config.timeout);
}
