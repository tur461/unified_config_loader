// This file is a **demonstration** of the configuration loader.
#![allow(dead_code)]

use unified_config_loader::{Config, ConfigLoader, ValueSource};

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "WORKER_", file_path = "files/worker.json")]
struct WorkerConfig {
    #[config(required)]
    queue_name: String,
    #[config(default = "4")]
    concurrency: usize,
    #[config(default = "1000")]
    max_retries: u32,
}

fn main() {
    // Override the concurrency value via environment variable
    unsafe {
        std::env::set_var("WORKER_CONCURRENCY", "8");
    }

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = format!("{}/files/worker.json", manifest_dir);
    unsafe {
        std::env::set_var("APP_CONFIG_FILE", &config_path);
    }

    match WorkerConfig::load() {
        Ok(config) => println!("Worker config (env overrides concurrency): {:?}", config),
        Err(e) => eprintln!("Error: {}", e),
    }
}
