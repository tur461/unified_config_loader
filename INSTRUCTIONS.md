# Instructions for Using `unified_config_loader`

This guide explains how to add the crate, write configuration files, run tests, and try the examples.

## Adding to Your Project

Add this to your `Cargo.toml`:

```toml
[dependencies]
unified_config_loader = { git = "https://github.com/tur461/unified_config_loader" }
```

Or, if it's published on crates.io:

```toml
[dependencies]
unified_config_loader = "0.1.0"
```

---

## Writing Your Config Struct

```rust
use unified_config_loader::ConfigLoader;

#[derive(ConfigLoader, Debug)]
#[config(env_prefix = "MYAPP_")]
struct Settings {
    #[config(default = "127.0.0.1")]
    bind_addr: String,

    #[config(default = 3000)]
    port: u16,

    #[config(required)]
    secret: String,
}
```

---

## Available Attributes

### Default Value

Uses the provided value if nothing else supplies one.

```rust
#[config(default = "value")]
```

### Default Function

Calls a function that returns `Result<T, ConfigError>`.

```rust
#[config(default_fn = "path::to::function")]
```

### Required Field

Loading fails if the value is still missing after merging all sources.

```rust
#[config(required)]
```

### Environment Variable Prefix

Changes the environment variable prefix for the entire struct.

```rust
#[config(env_prefix = "PREFIX_")]
```

---

## Where Config Files Are Read From

By default, the loader looks in the crate root (the same directory that contains `Cargo.toml`).

It checks the following files:

```text
.env
.env.local
.env.development
.env.production
... (any .env.* files, loaded alphabetically)

config.toml
config.json
config.yaml
config.ini
```

### Using a Specific Config File

If you set the `APP_CONFIG_FILE` environment variable, the loader reads **only that file** and ignores the conventional files listed above.

Example:

```bash
export APP_CONFIG_FILE=/path/to/config.toml
```

---

## Environment Variables

The loader reads all environment variables that start with the configured prefix (default: `APP_`).

It:

1. Removes the prefix.
2. Converts the remaining name to lowercase.
3. Uses that as the config key.

Example:

```bash
APP_DATABASE_URL=postgres://localhost
```

becomes:

```text
database_url = "postgres://localhost"
```

Environment variables always override values from config files and defaults.

---

## Using Hot Reload

Instead of calling `Config::load()`, use:

```rust
use unified_config_loader::hot_reload::ReloadableConfig;

let handle = ReloadableConfig::<Settings>::load()?;
let current = handle.get();

println!("port = {}", current.port);
```

The handle can be cloned and shared across threads.

A background thread watches the same configuration files that the loader normally reads. Whenever a file changes, it calls `Settings::load()` again and updates the internal `RwLock`.

---

## Running Tests

From the crate directory:

```bash
cargo test --all-features -- --test-threads=1
```

This runs all unit tests and integration tests.

The tests do not touch your real environment. They use temporary files and a fake `CARGO_MANIFEST_DIR`.

---

## Running Examples

Examples are located in the `examples/` directory.

Run them with:

```bash
cargo run --example basic
cargo run --example ini_basic --features=ini
cargo run --example toml_basic --features=toml
cargo run --example hot_reload_conventional --features=hot-reload,toml
cargo run --example hot_reload_single_file --features=hot-reload,toml
```

### Example: Creating a Config File

Before running `hot_reload_conventional`, create a config file (well we have some files already created in files/):

```bash
echo 'host = "example.com"' > config.toml
```

---

## If Something Doesn't Work

- Make sure your config file is in the crate root (the same directory as `Cargo.toml`).
- Check that your environment variable prefix is correct.
- Check examples from examples/ folder for reference.
- If a required field is missing, the error message will tell you which one.
- If you see strange macro-related errors, try:

```bash
cargo clean
cargo build
```

---

## Contributing or Reporting Bugs

The project is available on GitHub:

<https://github.com/tur461/unified_config_loader>

Feel free to open an issue or submit a pull request.

Please keep the code safe and avoid using `unwrap()` where errors can be handled properly.
