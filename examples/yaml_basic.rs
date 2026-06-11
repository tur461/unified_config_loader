use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "development"]
    environment: String,
    #[required]
    secret_key: String,
    debug: bool,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/app.yaml", manifest_dir);
    unsafe {
        std::env::set_var("CONFIG_FILE", &config_path);
    }

    let config = AppConfig::load().unwrap();
    println!(
        "Env: {}, Secret: {}, Debug: {}",
        config.environment, config.secret_key, config.debug
    );
}
