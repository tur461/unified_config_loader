use crate::errors::ConfigError;
use crate::traits::Config;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;

pub struct ConfigWatcher<C: Config> {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<Result<C, ConfigError>>,
}

impl<C: Config + Send + 'static> ConfigWatcher<C> {
    pub fn start() -> Result<(C, Self), ConfigError> {
        let file_path =
            std::env::var("CONFIG_FILE").map_err(|_| ConfigError::MissingRequiredField {
                field: "CONFIG_FILE".into(),
            })?;
        let path = Path::new(&file_path);
        let initial = C::load()?;

        let (tx, rx) = mpsc::channel();
        let mut watcher =
            notify::recommended_watcher(move |res: std::result::Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                            // Re‑load the config when the file changes
                            let _ = tx.send(C::load());
                        }
                    }
                    Err(e) => eprintln!("watch error: {}", e),
                }
            })
            .map_err(|e| ConfigError::ValidationError(format!("watcher creation failed: {}", e)))?;

        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::ValidationError(format!("watch failed: {}", e)))?;

        Ok((
            initial,
            ConfigWatcher {
                _watcher: watcher,
                rx,
            },
        ))
    }

    pub fn recv(&self) -> Result<Result<C, ConfigError>, mpsc::RecvError> {
        self.rx.recv()
    }

    pub fn try_recv(&self) -> Result<Option<Result<C, ConfigError>>, mpsc::TryRecvError> {
        match self.rx.try_recv() {
            Ok(val) => Ok(Some(val)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
