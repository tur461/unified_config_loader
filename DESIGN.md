# Design of `unified_config_loader`

This document explains why things work the way they do. No fancy terminology—just straightforward explanations.

## Features We Built

1. **One function to load everything** — `Config::load()`. You don't need to tell it where to look; it follows conventions.
2. **Strong types** — your config struct uses real Rust types like `u16`, not `String` everywhere.
3. **Three config sources** — defaults (in code), config files, and environment variables.
4. **Clear precedence** — environment variables override files, and files override defaults.
5. **Hot reload** — use a `ReloadableConfig` handle and it updates itself in the background when files change.
6. **Good errors** — if a required field is missing, the error tells you exactly which field is missing.

---

## Assumptions We Made

- Your project is a Cargo crate, so we can use `env!("CARGO_MANIFEST_DIR")` to find the crate root.
- Configuration files live in that root directory using standard names:
  - `.env`
  - `.env.local`
  - `.env.*`
  - `config.toml`
  - `config.json`
  - `config.yaml`
  - `config.ini`

- If you want to use a single custom config file, you set the `APP_CONFIG_FILE` environment variable or mention the file path in `#[config(file_path = "...")]` attribute.
- Environment variables use a prefix such as `APP_` (customizable with `#[config(env_prefix = "...")]`).
- Hot reload only watches configuration files. It does **not** watch environment variables. To apply environment variable changes, restart the application.

---

## Precedence Rules (Which Source Wins)

| Priority | Source                |
| -------- | --------------------- |
| Highest  | Environment Variables |
| Medium   | Config Files          |
| Lowest   | Defaults in Code      |

If multiple conventional config files exist, they are loaded in this order:

```text
.env
.env.local
.env.* (alphabetically sorted)
config.toml
config.json
config.yaml
config.ini
```

Later files override values from earlier files.

After all files are processed, environment variables still have the highest priority and override everything else.

---

## Why We Did Things This Way

### Convention Over Configuration

Most users don't want to write boilerplate just to tell the loader where config files are located. Looking in the crate root for standard filenames is simple and covers most use cases.

### Proc Macro

We could have used a manual builder pattern, but a proc macro lets users define a struct and a few attributes, and everything else is generated automatically.

### Hot Reload with `RwLock`

We chose `RwLock` instead of `tokio::watch` because we didn't want to require Tokio.

`RwLock` works well for configuration data:

- Many readers
- Occasional writes
- No async runtime required

A background thread handles reloading.

### No `unwrap()`

We use `?` for error propagation wherever possible.

The only exception is `expect()` when accessing a poisoned `RwLock`. A poisoned lock indicates a bug or panic elsewhere, so terminating is considered acceptable.

### Single-File Mode

Some users want to use a config file that doesn't follow the conventional naming scheme.

The `APP_CONFIG_FILE` environment variable supports that use case without requiring API changes.

---

## Possible Improvements

If we had more time, we could add:

- Watching environment variables for changes.
- Support for custom validation:

```rust
#[config(validate = "function")]
```

- Better automatic JSON Schema generation.
- Support for watching multiple files in single-file mode.
- A more advanced channel implementation than `std::sync::mpsc` in the watcher thread.

---

## Trade-offs We Made

### Conventional Files Only

Users cannot directly provide an arbitrary list of config files through the API.

This keeps the API simple and predictable.

### Hot Reload Thread Never Stops

The watcher thread runs until the process exits.

For long-running services, a graceful shutdown mechanism could be added later.

### Error Recovery

If a config file becomes invalid during runtime:

- The old configuration remains active.
- An error is logged.

This prevents applications from crashing due to temporary config mistakes, although it may delay discovery of configuration issues.

---

## Why a Derive Macro?

We wanted the `load()` method to be generated automatically instead of requiring users to write it manually.

Using:

```rust
#[derive(ConfigLoader)]
```

is the most natural and idiomatic Rust experience for this use case.

---

That is the design.

It is not perfect, but it is straightforward, practical, and gets the job done.
