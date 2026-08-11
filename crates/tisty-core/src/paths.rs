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
            // Local, never roaming: a Windows domain profile copies %APPDATA%
            // at logon and logoff. For the store that is another process
            // touching the log; for the config it is the device id — and the
            // `private/` folder — leaving the machine for a company server.
            data: env_path(DATA_ENV).unwrap_or_else(|| dirs.data_local_dir().to_path_buf()),
            config: env_path(CONFIG_ENV).unwrap_or_else(|| local_config(&dirs)),
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

    /// Outside the synced directory, so no cloud client ever carries it away.
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

/// macOS gives config and data the same directory, and a config sitting where
/// the store lives reads as something that travels. It does not — only `store/`
/// and `attachments/` ever do — but the separation is worth being visible.
fn local_config(dirs: &directories::ProjectDirs) -> PathBuf {
    let config = dirs.config_local_dir();
    if config == dirs.data_local_dir() {
        config.join("config")
    } else {
        config.to_path_buf()
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

    /// Against the real resolver, not two invented paths: the old version of
    /// this test compared `/data/Tisty` with `/config/tisty` and could not fail.
    #[test]
    fn nothing_private_lives_where_the_transports_look() {
        let p = Paths::resolve().unwrap();
        for kept in [p.config_file(), p.private(), p.cache().to_path_buf()] {
            assert!(!kept.starts_with(p.store()), "{kept:?}");
            assert!(!kept.starts_with(p.attachments()), "{kept:?}");
            assert!(!kept.starts_with(p.docs()), "{kept:?}");
        }
    }

    /// A roaming profile copies `%APPDATA%` to a company server at logoff,
    /// which would take the device id and `private/` with it.
    #[test]
    fn the_config_never_lands_in_a_roaming_profile() {
        let p = Paths::resolve().unwrap();
        if let Some(roaming) = std::env::var_os("APPDATA").map(PathBuf::from) {
            let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
            if local.as_deref() != Some(roaming.as_path()) {
                assert!(!p.config().starts_with(&roaming), "{:?}", p.config());
                assert!(!p.private().starts_with(&roaming), "{:?}", p.private());
            }
        }
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
