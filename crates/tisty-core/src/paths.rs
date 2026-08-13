use std::path::{Path, PathBuf};

use crate::{Error, Result, event::DeviceId};

pub const DATA_ENV: &str = "TISTY_DATA";
pub const CONFIG_ENV: &str = "TISTY_CONFIG";
pub const CACHE_ENV: &str = "TISTY_CACHE";
pub const PROFILE_ENV: &str = "TISTY_PROFILE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    data: PathBuf,
    config: PathBuf,
    cache: PathBuf,
}

impl Paths {
    pub fn resolve() -> Result<Self> {
        let dirs = directories::ProjectDirs::from("", "", "tisty").ok_or(Error::NoHomeDirectory)?;
        let under = profile();

        Ok(Self {
            data: aside(
                env_path(DATA_ENV).unwrap_or_else(|| dirs.data_local_dir().to_path_buf()),
                under.as_deref(),
            ),
            config: aside(
                env_path(CONFIG_ENV).unwrap_or_else(|| local_config(&dirs)),
                under.as_deref(),
            ),
            cache: aside(
                env_path(CACHE_ENV).unwrap_or_else(|| dirs.cache_dir().to_path_buf()),
                under.as_deref(),
            ),
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

    pub fn private(&self) -> PathBuf {
        self.config.join("private")
    }

    pub fn cache(&self) -> &Path {
        &self.cache
    }

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

fn local_config(dirs: &directories::ProjectDirs) -> PathBuf {
    let config = dirs.config_local_dir();
    if config == dirs.data_local_dir() {
        config.join("config")
    } else {
        config.to_path_buf()
    }
}

#[cfg(unix)]
pub fn ours_alone(at: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = if at.is_dir() { 0o700 } else { 0o600 };
    std::fs::set_permissions(at, std::fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
pub fn ours_alone(at: &Path) -> std::io::Result<()> {
    let _ = at;
    Ok(())
}

pub fn profile() -> Option<String> {
    named(&std::env::var(PROFILE_ENV).ok()?)
}

fn named(raw: &str) -> Option<String> {
    let clean: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(48)
        .collect();
    (!clean.is_empty()).then_some(clean)
}

fn aside(root: PathBuf, under: Option<&str>) -> PathBuf {
    match under {
        Some(name) => root.join("sandboxes").join(name),
        None => root,
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{aside, named};
    use std::path::PathBuf;

    #[test]
    fn a_profile_name_cannot_escape_its_own_directory() {
        for raw in ["../..", "..", "/etc", "a/../../b", r"..\..\windows"] {
            let clean = named(raw).unwrap_or_default();
            assert!(!clean.contains(".."), "{raw} survived as {clean}");
            assert!(!clean.contains('/'), "{raw} -> {clean}");
            assert!(!clean.contains('\\'), "{raw} -> {clean}");
        }
    }

    #[test]
    fn an_ordinary_name_survives_whole() {
        assert_eq!(named("demo"), Some("demo".into()));
        assert_eq!(named("dos-maquinas_2"), Some("dos-maquinas_2".into()));
    }

    #[test]
    fn a_name_with_nothing_usable_in_it_is_no_profile() {
        assert_eq!(named(""), None);
        assert_eq!(named("   "), None);
        assert_eq!(named("../.."), None);
    }

    #[test]
    fn without_a_profile_nothing_moves() {
        let root = PathBuf::from("/data");

        assert_eq!(aside(root.clone(), None), root);
        assert_eq!(
            aside(root, Some("demo")),
            PathBuf::from("/data/sandboxes/demo")
        );
    }
    use super::*;

    fn paths() -> Paths {
        Paths::new("/data/Tisty", "/config/tisty")
    }

    #[test]
    fn nothing_private_lives_where_the_transports_look() {
        let p = Paths::resolve().unwrap();
        for kept in [p.config_file(), p.private(), p.cache().to_path_buf()] {
            assert!(!kept.starts_with(p.store()), "{kept:?}");
            assert!(!kept.starts_with(p.attachments()), "{kept:?}");
            assert!(!kept.starts_with(p.docs()), "{kept:?}");
        }
    }

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
