use std::env;
use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/auth.yaml")]
struct AuthConfig {
    #[config(required)]
    jwt_secret: String,
    #[config(default = "3600")]
    ttl: u64,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/auth.yaml", manifest_dir);
    unsafe {
        env::set_var("APP_CONFIG_FILE", &config_path);
        env::set_var("TTL", "7200"); // overrides YAML & default
    }

    let config = AuthConfig::load().unwrap();
    println!(
        "JWT Secret length: {}, TTL: {}",
        config.jwt_secret.len(),
        config.ttl
    );
}
