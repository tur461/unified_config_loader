use unified_config_loader::ConfigLoader;
use unified_config_loader::errors::ConfigError;
use unified_config_loader::traits::{Config, Validate};

#[derive(ConfigLoader, Debug)]
struct AppConfig {
    #[default = "8080"]
    port: u16,
    #[default = "info"]
    log_level: String,
}

impl Validate for AppConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.port == 0 {
            return Err(ConfigError::ValidationError("Port cannot be 0".into()));
        }
        if !["info", "debug", "warn", "error"].contains(&self.log_level.as_str()) {
            return Err(ConfigError::ValidationError(
                "log_level must be one of info, debug, warn, error".into(),
            ));
        }
        Ok(())
    }
}

fn main() {
    match AppConfig::load() {
        Ok(config) => {
            if let Err(e) = config.validate() {
                eprintln!("Invalid config: {e}");
                return;
            }
            println!("Valid config: {:?}", config);
        }
        Err(e) => eprintln!("Load error: {e}"),
    }
}
