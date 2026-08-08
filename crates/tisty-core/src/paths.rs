use std::path::{Path, PathBuf};

use crate::{Error, Result, event::DeviceId};

pub const DATA_ENV: &str = "TISTY_DATA";
pub const CONFIG_ENV: &str = "TISTY_CONFIG";
pub const CACHE_ENV: &str = "TISTY_CACHE";

/// Apart on purpose: a synced device id would break one-writer-per-directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    data: PathBuf,
    config: PathBuf,
    cache: PathBuf,
}

impl Paths {
    pub fn resolve() -> Result<Self> {
        let dirs = directories::ProjectDirs::from("", "", "tisty").ok_or(Error::NoHomeDirectory)?;

        Ok(Self {
            data: env_path(DATA_ENV).unwrap_or_else(|| dirs.data_dir().to_path_buf()),
            config: env_path(CONFIG_ENV).unwrap_or_else(|| dirs.config_dir().to_path_buf()),
            cache: env_path(CACHE_ENV).unwrap_or_else(|| dirs.cache_dir().to_path_buf()),
        })
    }

    pub fn new(data: impl Into<PathBuf>, config: impl Into<PathBuf>) -> Self {
        let config = config.into();
        Self {
            data: data.into(),
            cache: config.join("cache"),
            config,
        }
    }

    pub fn data(&self) -> &Path {
        &self.data
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn config_file(&self) -> PathBuf {
        self.config.join("config.toml")
    }

    /// Outside the synced directory: a `.gitignore` only stops Git, not Dropbox.
    pub fn private(&self) -> PathBuf {
        self.config.join("private")
    }

    pub fn cache(&self) -> &Path {
        &self.cache
    }

    /// Disposable: only outlives the seconds between reading a list and typing.
    pub fn selection_file(&self) -> PathBuf {
        self.cache.join("selection.json")
    }

    pub fn store(&self) -> PathBuf {
        self.data.join("store")
    }

    pub fn device_dir(&self, device: &DeviceId) -> PathBuf {
        self.store().join(&device.0)
    }

    pub fn attachments(&self) -> PathBuf {
        self.data.join("attachments")
    }

    pub fn docs(&self) -> PathBuf {
        self.data.join("docs")
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths() -> Paths {
        Paths::new("/data/Tisty", "/config/tisty")
    }

    #[test]
    fn config_never_lives_inside_the_synced_directory() {
        let p = paths();
        assert!(!p.config_file().starts_with(p.data()));
        assert!(!p.private().starts_with(p.data()));
    }

    #[test]
    fn every_device_gets_its_own_directory() {
        let p = paths();
        let a = p.device_dir(&DeviceId("dev_a".into()));
        let b = p.device_dir(&DeviceId("dev_b".into()));

        assert_ne!(a, b);
        assert!(a.starts_with(p.store()));
        assert!(b.starts_with(p.store()));
    }

    #[test]
    fn the_store_never_lands_in_the_documents_folder() {
        let p = Paths::resolve().unwrap();
        assert!(p.data().is_absolute(), "{:?}", p.data());

        let documents =
            directories::UserDirs::new().and_then(|d| d.document_dir().map(Path::to_path_buf));
        if let Some(documents) = documents {
            assert!(!p.data().starts_with(&documents), "{:?}", p.data());
        }
    }

    #[test]
    fn the_cache_is_disposable_and_never_synced() {
        let p = paths();
        assert!(!p.selection_file().starts_with(p.data()));
    }

    #[test]
    fn store_docs_and_attachments_are_all_synced() {
        let p = paths();
        for path in [p.store(), p.docs(), p.attachments()] {
            assert!(path.starts_with(p.data()), "{path:?} should be synced");
        }
    }
}
