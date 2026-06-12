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
use unified_config_loader::ConfigError;
use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::hot_reload::ReloadableConfig;

#[derive(ConfigLoader, Debug, Clone)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
    #[config(default_fn = "default_port")]
    port: u16,
}

fn default_port() -> Result<u16, ConfigError> {
    println!("default_port() called – returning 9999");
    Ok(9999)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        env::set_var(
            "APP_CONFIG_FILE",
            format!("{manifest_dir}/files/config_default_fn_hr.toml"),
        );
    }
    let config_handle = ReloadableConfig::<AppConfig>::load()?;

    println!("No config file yet – default_fn will provide the port.\n");

    thread::spawn(move || {
        loop {
            {
                let cfg = config_handle.get();
                println!("Config: host={}, port={}", cfg.host, cfg.port);
            }
            thread::sleep(Duration::from_secs(3));
        }
    });

    thread::spawn(|| {
        thread::sleep(Duration::from_secs(10));
        std::fs::write("files/config_default_fn_hr.toml", "port = 1234").unwrap();
        println!("\n[writer] config.toml created with port=1234\n");
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
