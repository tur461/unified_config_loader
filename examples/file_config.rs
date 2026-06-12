use std::io::Write;
use std::path::Path;
use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_", file_path = "files/app_config.env")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default = "8080")]
    port: u16,
    #[config(default = "info")]
    log_level: String,
}

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = dir.join("files/app_config.env");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "host=prod.example.com\nport=443\nlog_level=warn").unwrap();
    let config = AppConfig::load().unwrap();
    println!(
        "File config: {}:{} (log: {})",
        config.host, config.port, config.log_level
    );

    std::fs::remove_file(&path).ok();
}
