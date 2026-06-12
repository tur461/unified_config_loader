use std::env;
use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/server.toml")]
struct ServerConfig {
    #[config(default = "0.0.0.0")]
    bind: String,
    #[config(default = "8080")]
    port: u16,
}

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/server.toml", manifest_dir);
    unsafe {
        env::set_var("APP_CONFIG_FILE", &config_path);
        env::set_var("PORT", "9090"); // overrides file & default
    }

    let config = ServerConfig::load().unwrap();
    println!("Bind: {}, Port: {}", config.bind, config.port);
}
