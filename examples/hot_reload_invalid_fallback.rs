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
#[config(env_prefix = "MYAPP_", file_path = "files/config_fallback_hr.toml")]
struct AppConfig {
    #[config(required)]
    api_key: String,
    #[config(default = 8080)]
    port: u16,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    unsafe {
        env::set_var(
            "APP_CONFIG_FILE",
            format!("{manifest_dir}/files/config_fallback_hr.toml"),
        );
    }
    std::fs::write(
        format!("{manifest_dir}/files/config_fallback_hr.toml"),
        "api_key = \"valid-key\"\nport = 8080",
    )?;

    let config_handle = ReloadableConfig::<AppConfig>::load()?;

    println!("Initial valid config loaded.");
    println!(
        "Now try editing config.toml to be invalid (e.g., api_key = 123) – the old config will stay.\n"
    );

    thread::spawn(move || {
        loop {
            let cfg = config_handle.get();
            println!(
                "Current config: api_key = {}, port = {}",
                cfg.api_key, cfg.port
            );
            drop(cfg);
            thread::sleep(Duration::from_secs(3));
        }
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
