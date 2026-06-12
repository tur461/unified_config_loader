// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::hot_reload::ReloadableConfig;

#[derive(ConfigLoader, Debug, Clone)]
#[config(env_prefix = "MYAPP_")]
struct AppConfig {
    #[config(default = "localhost")]
    host: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let config_handle = ReloadableConfig::<AppConfig>::load()?;

    let worker_handle = thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            let cfg = config_handle.get();
            println!("Worker: host = {}", cfg.host);
            thread::sleep(Duration::from_secs(2));
        }
        println!("Worker shutting down gracefully.");
    });

    thread::sleep(Duration::from_secs(10));
    r.store(false, Ordering::Relaxed);
    worker_handle.join().unwrap();

    println!("Main: application exited cleanly.");
    Ok(())
}
