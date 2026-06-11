use std::thread;
use std::time::Duration;
use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "info"]
    log_level: String,
}

fn main() {
    let mut config = AppConfig::load().unwrap();
    println!("Initial log level: {}", config.log_level);

    // Simulate external change (e.g., after some time)
    unsafe {
        std::env::set_var("LOG_LEVEL", "debug");
    }
    thread::sleep(Duration::from_secs(1));

    // Reload
    config = AppConfig::load().unwrap();
    println!("Updated log level: {}", config.log_level);
}
