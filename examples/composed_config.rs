// This file is a **demonstration** of the configuration loader.
// Warnings about `unwrap`, `expect`, or unused code are suppressed because:
// - Examples should be short and readable, not production‑perfect.
// - A panic in an example is acceptable – it shows what happens on error.
#![allow(
    dead_code                   // some structs or functions are for illustration only
)]

use unified_config_loader::ConfigLoader;
use unified_config_loader::ValueSource;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct ServerConfig {
    #[config(default = "0.0.0.0")]
    host: String,
    #[config(default = "8080")]
    port: u16,
}

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct DatabaseConfig {
    #[config(required)]
    url: String,
    #[config(default = "5")]
    max_connections: u32,
}

fn main() {
    unsafe {
        std::env::set_var("URL", "postgres://localhost/test");
    }
    let server = ServerConfig::load().unwrap();
    let db = DatabaseConfig::load().unwrap();
    println!("Server: {:?}", server);
    println!("Database: {:?}", db);
}
