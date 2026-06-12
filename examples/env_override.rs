use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default = "8080")]
    port: u16,
    #[config(default = "info")]
    log_level: String,
}

fn main() {
    unsafe {
        // highest precedence
        std::env::set_var("PORT", "9090");
        std::env::set_var("LOG_LEVEL", "debug");
    }

    let config = AppConfig::load().unwrap();
    println!(
        "Overridden: {}:{} with log level {}",
        config.host, config.port, config.log_level
    );
}
