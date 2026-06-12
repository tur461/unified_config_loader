# Unified Configuration Loader

A simple but powerful configuration loader for Rust applications.

It loads settings from defaults, configuration files, and environment variables, then merges them together so your application always gets the final value it should use.

## Features

- Define your configuration using a normal Rust struct.
- Add a few `#[config(...)]` attributes.
- Automatically get a `load()` method for your config type.
- Load configuration from:
  - Defaults defined in code
  - Configuration files (`.env`, `config.toml`, `config.json`, `config.yaml`, `config.ini`, etc.)
  - Environment variables with a configurable prefix

- Clear error messages when required values are missing.
- Optional hot reload support that automatically updates configuration when files change.

---

## Example

```rust
use unified_config_loader::ConfigLoader;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct MyConfig {
    #[config(default = "hello")]
    message: String,

    #[config(required)]
    api_key: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = MyConfig::load()?;

    println!("Loaded: {:?}", cfg);

    Ok(())
}
```

---

## Documentation

- [Instructions for Installation, Usage, Testing, and Examples](INSTRUCTIONS.md)
- [Design Decisions and Trade-offs](DESIGN.md)
- [License](LICENSE)

---

## Configuration Precedence

Configuration values are merged using the following priority order:

| Priority | Source                |
| -------- | --------------------- |
| Highest  | Environment Variables |
| Medium   | Configuration Files   |
| Lowest   | Defaults in Code      |

This means environment variables always override file values, and file values always override defaults.

---

## Hot Reload

If you want configuration updates without restarting your application, use:

```rust
use unified_config_loader::hot_reload::ReloadableConfig;

let handle = ReloadableConfig::<MyConfig>::load()?;
let config = handle.get();
```

The loader watches configuration files in the background and automatically reloads them when they change.

---

## License

Licensed under the MIT License. See the `LICENSE` file for details.
