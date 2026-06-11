use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "localhost"]
    host: String,
    #[default = "8080"]
    port: u16,
    #[default = "info"]
    log_level: String,
}

fn main() {
    unsafe {
        std::env::set_var("PORT", "9090");
        std::env::set_var("LOG_LEVEL", "debug");
    }

    let config = AppConfig::load().unwrap();
    println!(
        "Overridden: {}:{} with log level {}",
        config.host, config.port, config.log_level
    );
}
