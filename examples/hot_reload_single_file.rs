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
#[config(env_prefix = "MYAPP_", file_path = "files/my_config.toml")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(required)]
    api_key: String,
    #[config(default = 8080)]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        env::set_var(
            "APP_CONFIG_FILE",
            format!("{manifest_dir}/files/my_config.toml"),
        );
    }

    let config_handle = ReloadableConfig::<AppConfig>::load()?;

    thread::spawn(move || {
        loop {
            {
                let cfg = config_handle.get();
                println!(
                    "API key: {}, host: {}, port: {}",
                    cfg.api_key, cfg.host, cfg.port
                );
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
