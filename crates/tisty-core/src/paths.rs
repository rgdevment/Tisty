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
        let (data, config, cache) = defaults(&dirs);
        Ok(Self {
            data: aside(env_path(DATA_ENV).unwrap_or(data), under.as_deref()),
            config: aside(env_path(CONFIG_ENV).unwrap_or(config), under.as_deref()),
            cache: aside(env_path(CACHE_ENV).unwrap_or(cache), under.as_deref()),
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
        settled(&self.data) == settled(&defaults(&dirs).0)
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

const STORE_PACKAGE: &str = "rgdevment.Tisty_kdjgfdc2rb3gc";
const STORE_PACKAGE_PREFIX: &str = "rgdevment.Tisty_";

fn defaults(dirs: &directories::ProjectDirs) -> (PathBuf, PathBuf, PathBuf) {
    if let Some(root) = home_root(dirs) {
        return (root.join("data"), root.join("config"), root.join("cache"));
    }
    (
        dirs.data_local_dir().to_path_buf(),
        local_config(dirs),
        dirs.cache_dir().to_path_buf(),
    )
}

/// The Store keeps `AppData` in a copy it deletes on uninstall; the profile root it leaves alone.
fn home_root(dirs: &directories::ProjectDirs) -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    let new = directories::UserDirs::new()?.home_dir().join(".tisty");
    let legacy = dirs.data_local_dir().parent()?;
    (new.exists() || !legacy.exists()).then_some(new)
}

/// Run once per process before `resolve`, never from what only sweeps or reads a setting.
pub fn settle_home() -> Option<crate::moving::Settled> {
    if !cfg!(windows)
        || [DATA_ENV, CONFIG_ENV, CACHE_ENV, PROFILE_ENV]
            .iter()
            .any(|key| env_path(key).is_some())
    {
        return None;
    }
    let dirs = directories::ProjectDirs::from("", "", "tisty")?;
    let real = dirs.data_local_dir().parent()?.to_path_buf();
    let private = store_package(real.parent()?);
    let new = directories::UserDirs::new()?.home_dir().join(".tisty");
    Some(crate::moving::settle(&crate::moving::Roots {
        new,
        real,
        private,
    }))
}

fn store_package(local: &Path) -> Option<PathBuf> {
    let inside = |package: PathBuf| package.join("LocalCache").join("Local").join("tisty");
    let packages = local.join("Packages");
    let exact = inside(packages.join(STORE_PACKAGE));
    if exact.is_dir() {
        return Some(exact);
    }
    std::fs::read_dir(&packages)
        .ok()?
        .filter_map(|one| one.ok())
        .filter(|one| {
            one.file_name()
                .to_string_lossy()
                .starts_with(STORE_PACKAGE_PREFIX)
        })
        .map(|one| inside(one.path()))
        .find(|at| at.is_dir())
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

pub fn as_written(at: &Path) -> String {
    std::fs::canonicalize(at)
        .unwrap_or_else(|_| at.to_path_buf())
        .display()
        .to_string()
}

#[cfg(windows)]
pub fn told_apart(at: &Path) -> Option<String> {
    let held = winapi_util::Handle::from_path_any(at).ok()?;
    let one = winapi_util::file::information(&held).ok()?;
    apart(one.volume_serial_number(), one.file_index())
}

#[cfg(not(windows))]
pub fn told_apart(at: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;
    let one = std::fs::metadata(at).ok()?;
    apart(one.dev(), one.ino())
}

fn apart(volume: u64, one: u64) -> Option<String> {
    (one != 0).then(|| format!("{volume:x}:{one:x}"))
}

pub fn told_of(at: &Path) -> String {
    match told_apart(at) {
        Some(one) => format!("{one}\n{}", at.display()),
        None => at.display().to_string(),
    }
}

fn a_mark(one: &str) -> bool {
    matches!(one.split_once(':'), Some((volume, at))
        if !volume.is_empty()
            && !at.is_empty()
            && [volume, at]
                .iter()
                .all(|one| one.bytes().all(|b| b.is_ascii_hexdigit())))
}

pub fn is_the_one(kept: &str, at: &Path) -> bool {
    let (mark, path) = match kept.split_once('\n') {
        Some((one, rest)) if a_mark(one.trim()) => (Some(one.trim()), rest),
        _ => (None, kept),
    };
    let path = path.trim();
    if let (Some(mark), Some(now)) = (mark, told_apart(at)) {
        return mark == now;
    }
    !path.is_empty() && as_written(Path::new(path)) == as_written(at)
}

pub fn under_windows_apps(running: &Path) -> bool {
    running
        .to_string_lossy()
        .split(['/', '\\'])
        .any(|part| part.eq_ignore_ascii_case("WindowsApps"))
}

pub fn from_the_store() -> bool {
    std::env::current_exe().is_ok_and(|at| under_windows_apps(&at))
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
