use unified_config_loader::ConfigLoader;
use unified_config_loader::traits::Config;

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[required]
    secret_key: String,
    #[default = "8080"]
    port: u16,
}

fn main() {
    // Not setting secret_key will cause MissingRequiredField error.
    match AppConfig::load() {
        Ok(config) => println!("Config loaded: {:?}", config),
        Err(e) => {
            eprintln!("Configuration error: {e}");
            // You can match on error kind if you wish
            if let unified_config_loader::ConfigError::MissingRequiredField { field } = &e {
                eprintln!(
                    "Hint: set environment variable {}=...",
                    field.to_uppercase()
                );
            }
            std::process::exit(1);
        }
    }
}
