use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/app.yaml")]
struct AppConfig {
    #[config(default = "development")]
    environment: String,
    #[config(required)]
    secret_key: String,
    debug: bool,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/app.yaml", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    let config = AppConfig::load().unwrap();
    println!(
        "Env: {}, Secret: {}, Debug: {}",
        config.environment, config.secret_key, config.debug
    );
}
