use crate::errors::ConfigError;
use crate::traits::Config;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;

#[derive(Clone)]
pub struct ReloadableConfig<C: Config + Clone> {
    inner: Arc<RwLock<C>>,
    _watcher_handle: Arc<thread::JoinHandle<()>>,
}

impl<C: Config + Clone + Send + 'static + Sync> ReloadableConfig<C> {
    pub fn load() -> Result<Self, ConfigError> {
        let (watched_paths, is_single_file) = Self::determine_watched_paths()?;

        let initial = C::load()?;
        let inner = Arc::new(RwLock::new(initial));

        let inner_watcher = Arc::clone(&inner);

        let handle = thread::spawn(move || {
            Self::watcher_thread(watched_paths, is_single_file, inner_watcher);
        });

        Ok(ReloadableConfig {
            inner,
            _watcher_handle: Arc::new(handle),
        })
    }

    pub fn get(&self) -> RwLockReadGuard<'_, C> {
        self.inner.read().expect("config RwLock poisoned")
    }

    pub fn get_cloned(&self) -> C {
        self.inner.read().expect("config RwLock poisoned").clone()
    }

    fn determine_watched_paths() -> Result<(Vec<PathBuf>, bool), ConfigError> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // Single-file mode via APP_CONFIG_FILE (same as macro's `option_env!`)
        if let Ok(custom_file) = std::env::var("APP_CONFIG_FILE") {
            let path = manifest_dir.join(&custom_file);
            if !path.exists() {
                eprintln!(
                    "[config-watcher] Warning: APP_CONFIG_FILE points to non-existent file: {}",
                    custom_file
                );
            }
            return Ok((vec![path], true));
        }

        // Conventional mode: watch the manifest directory and all conventional files
        let mut paths = Vec::new();
        paths.push(manifest_dir.clone()); // watch the whole directory
        paths.push(manifest_dir.join(".env"));
        paths.push(manifest_dir.join(".env.local"));
        for ext in &["toml", "json", "yaml", "ini"] {
            paths.push(manifest_dir.join(format!("config.{}", ext)));
        }
        Ok((paths, false))
    }

    fn watcher_thread(watched_paths: Vec<PathBuf>, is_single_file: bool, inner: Arc<RwLock<C>>) {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher = match notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        }) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[config-watcher] Failed to create watcher: {}", e);
                return;
            }
        };

        // Set up watches on all required paths
        for path in &watched_paths {
            if path.is_dir() {
                if let Err(e) = watcher.watch(path, RecursiveMode::NonRecursive) {
                    eprintln!(
                        "[config-watcher] Failed to watch directory {:?}: {}",
                        path, e
                    );
                }
            } else if let Some(parent) = path.parent()
                && let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive)
            {
                eprintln!(
                    "[config-watcher] Failed to watch parent of {:?}: {}",
                    path, e
                );
            }
        }

        // Main event loop
        for event in rx {
            match event {
                Ok(Event { kind, paths, .. }) => {
                    let should_reload = if is_single_file {
                        // Only reload if the changed file is exactly the one we're watching
                        paths.iter().any(|p| watched_paths.contains(p))
                    } else {
                        // In conventional mode, reload on any modification/create in the manifest dir
                        matches!(kind, EventKind::Modify(_) | EventKind::Create(_))
                    };
                    if should_reload {
                        match C::load() {
                            Ok(new_config) => {
                                if let Ok(mut guard) = inner.write() {
                                    *guard = new_config;
                                }
                            }
                            Err(e) => eprintln!("[config-watcher] Reload failed: {}", e),
                        }
                    }
                }
                Err(e) => eprintln!("[config-watcher] Notify error: {}", e),
            }
        }
    }
}
