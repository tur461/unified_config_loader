use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default = 8080)]
    port: u16,
    #[config(default = true, required)]
    enabled: bool,
}

fn main() {
    let config = AppConfig::load().unwrap();
    println!(
        "Server running at {}:{} enabled?: {}",
        config.host, config.port, config.enabled
    );

    println!("Schema: {:?}", AppConfig::schema());
}
