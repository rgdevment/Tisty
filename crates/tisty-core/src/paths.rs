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
    paired: bool,
}

impl Paths {
    pub fn resolve() -> Result<Self> {
        let dirs = directories::ProjectDirs::from("", "", "tisty").ok_or(Error::NoHomeDirectory)?;
        let under = profile();

        let told = |key| env_path(key).is_some();
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
            paired: !told(DATA_ENV) || told(CONFIG_ENV),
        })
    }

    /// The store the person chose is the one at the platform's own place, however the path was
    /// arrived at: `TISTY_DATA` pointing right back at it does not make it somebody else's,
    /// while a sandbox or a temporary directory is nobody's list.
    pub fn the_persons_own(&self) -> bool {
        let Some(dirs) = directories::ProjectDirs::from("", "", "tisty") else {
            return false;
        };
        let settled = |at: &Path| at.canonicalize().unwrap_or_else(|_| at.to_path_buf());
        settled(&self.data) == settled(dirs.data_local_dir())
    }

    pub fn swept_on_leaving(&self) -> Vec<PathBuf> {
        let mut swept = vec![self.config_file(), self.cache.clone()];
        swept.extend(crate::witness::kept_files(self));
        swept
    }

    pub fn shims() -> Vec<PathBuf> {
        let Some(dirs) = directories::UserDirs::new() else {
            return Vec::new();
        };
        let home = dirs.home_dir();
        ["tisty-mcp", "tisty"]
            .iter()
            .map(|one| home.join(".local").join("bin").join(one))
            .filter(|at| at.is_file())
            .collect()
    }

    pub fn new(data: impl Into<PathBuf>, config: impl Into<PathBuf>) -> Self {
        let config = config.into();
        Self {
            data: data.into(),
            cache: config.join("cache"),
            config,
            paired: true,
        }
    }

    pub fn of_one_install(&self) -> bool {
        self.paired
    }

    #[cfg(test)]
    pub(crate) fn unpaired_for_test(&mut self) {
        self.paired = false;
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
#[path = "paths_test.rs"]
mod tests;
