use std::io::Write;
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
    // Create a temporary config file
    let dir = std::env::temp_dir();
    let path = dir.join("app_config.env");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "host=prod.example.com\nport=443\nlog_level=warn").unwrap();
    unsafe {
        std::env::set_var("CONFIG_FILE", &path);
    }
    let config = AppConfig::load().unwrap();
    println!(
        "File config: {}:{} (log: {})",
        config.host, config.port, config.log_level
    );

    std::fs::remove_file(&path).ok();
}
