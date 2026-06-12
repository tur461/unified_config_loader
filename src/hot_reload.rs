use crate::errors::ConfigError;
use crate::traits::Config;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use std::thread;

#[derive(Clone)]
pub struct ReloadableConfig<C: Config + Clone> {
    inner: Arc<RwLock<C>>,
    _watcher_handle: Arc<thread::JoinHandle<()>>,
}

impl<C: Config + Clone + Send + 'static + Sync + std::fmt::Debug> ReloadableConfig<C> {
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
            let path = PathBuf::from(&custom_file);
            if !path.exists() {
                eprintln!(
                    "[config-watcher] Warning: APP_CONFIG_FILE points to non-existent file: {}",
                    custom_file
                );
            }
            let canonical_path = Self::canonicalize_path(&path);
            println!("watching custom_file: {}", canonical_path.display());

            return Ok((vec![canonical_path], true));
        }

        let mut paths = Vec::new();
        let canonical_manifest = Self::canonicalize_path(&manifest_dir);
        paths.push(canonical_manifest.clone()); // watch the whole directory
        let mut add_file = |file_name: &str| {
            let file_path = manifest_dir.join(file_name);
            paths.push(Self::canonicalize_path(&file_path));
        };
        add_file(".env");
        add_file(".env.local");
        for ext in &["toml", "json", "yaml", "ini"] {
            add_file(&format!("config.{}", ext));
        }
        Ok((paths, false))
    }

    fn canonicalize_path(path: &Path) -> PathBuf {
        if path.exists() {
            if let Ok(canonical) = path.canonicalize() {
                return canonical;
            }
        } else if let Some(parent) = path.parent()
            && parent.exists()
            && let Ok(canonical_parent) = parent.canonicalize()
            && let Some(file_name) = path.file_name()
        {
            return canonical_parent.join(file_name);
        }
        path.to_path_buf()
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
                        let watched = &watched_paths[0]; // there is exactly one path
                        paths.iter().any(|event_path| {
                            if let Ok(canonical) = event_path.canonicalize() {
                                canonical == *watched
                            } else {
                                event_path == watched
                            }
                        })
                    } else {
                        matches!(kind, EventKind::Modify(_) | EventKind::Create(_))
                    };

                    if should_reload {
                        println!("we should reload");
                        match C::load() {
                            Ok(new_config) => {
                                println!("we got new config: {new_config:?}");
                                if let Ok(mut guard) = inner.write() {
                                    *guard = new_config;
                                }
                                println!("config reloaded.");
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
