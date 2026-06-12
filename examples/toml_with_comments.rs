use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/database.toml")]
struct DatabaseConfig {
    #[config(required)]
    url: String,
    #[config(default = "5")]
    pool_size: u32,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/database.toml", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    let config = DatabaseConfig::load().unwrap();
    println!(
        "Database URL: {}, Pool size: {}",
        config.url, config.pool_size
    );
}
