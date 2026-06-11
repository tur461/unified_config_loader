use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct ServerConfig {
    #[default = "0.0.0.0"]
    host: String,
    #[default = "8080"]
    port: u16,
}

#[derive(ConfigLoader, Debug)]
struct DatabaseConfig {
    #[required]
    url: String,
    #[default = "5"]
    max_connections: u32,
}

fn main() {
    // Note: both structs share the same environment and file source.
    // You may need to set CONFIG_FILE before each load if they use different files.
    // For simplicity, we load them sequentially.
    unsafe {
        std::env::set_var("URL", "postgres://localhost/test");
    }
    let server = ServerConfig::load().unwrap();
    let db = DatabaseConfig::load().unwrap();
    println!("Server: {:?}", server);
    println!("Database: {:?}", db);
}
