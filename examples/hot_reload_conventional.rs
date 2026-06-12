// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use std::env;
use std::thread;
use std::time::Duration;
use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::hot_reload::ReloadableConfig;

#[derive(ConfigLoader, Debug, Clone)]
#[config(env_prefix = "MYAPP_", file_path = "files/hr_conv.toml")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default = 8080)]
    port: u16,
    #[config(default = "info")]
    log_level: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        env::set_var("APP_CONFIG_FILE", "files/hr_conv.toml");
    }
    let config = ReloadableConfig::<AppConfig>::load()?;

    for i in 0..3 {
        let cfg_handle = config.clone();
        thread::spawn(move || {
            loop {
                let cfg = cfg_handle.get();
                println!("[Worker {}] host={}, port={}", i, cfg.host, cfg.port);
                drop(cfg);
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

    loop {
        let cfg = config.get();
        println!("[Main] log_level={}", cfg.log_level);
        thread::sleep(Duration::from_secs(3));
    }
}
