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
        let shared = shared_home().is_some_and(|root| self.config.starts_with(root));
        let mut swept = match shared {
            true => vec![self.cache.clone()],
            false => vec![self.config_file(), self.cache.clone()],
        };
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
    let new = shared_home()?;
    if new.exists() {
        return Some(new);
    }
    let legacy = dirs.data_local_dir().parent()?;
    let left = legacy.exists() || legacy.parent().and_then(store_package).is_some();
    (!left).then_some(new)
}

/// Every Windows install shares it, so leaving one must not take another's settings with it.
fn shared_home() -> Option<PathBuf> {
    if !cfg!(windows) {
        return None;
    }
    Some(directories::UserDirs::new()?.home_dir().join(".tisty"))
}

/// Only the window moves the store; the command line uses whichever root exists.
pub fn settle_home() -> Option<crate::moving::Settled> {
    static ONCE: std::sync::OnceLock<Option<crate::moving::Settled>> = std::sync::OnceLock::new();
    ONCE.get_or_init(settled_home).clone()
}

fn settled_home() -> Option<crate::moving::Settled> {
    if [DATA_ENV, CONFIG_ENV, CACHE_ENV, PROFILE_ENV]
        .iter()
        .any(|key| env_path(key).is_some())
    {
        return None;
    }
    let new = shared_home()?;
    let dirs = directories::ProjectDirs::from("", "", "tisty")?;
    let real = dirs.data_local_dir().parent()?.to_path_buf();
    let private = match new.exists() {
        true => None,
        false => store_package(real.parent()?),
    };
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

#[cfg(not(windows))]
pub fn key_alone(at: &Path) -> std::io::Result<()> {
    ours_alone(at)
}

// Through icacls rather than the security API, which would need the unsafe code the workspace forbids.
#[cfg(windows)]
pub fn key_alone(at: &Path) -> std::io::Result<()> {
    let system = std::env::var_os("SystemRoot")
        .map(|root| std::path::PathBuf::from(root).join("System32"))
        .ok_or_else(|| std::io::Error::other("the system folder could not be found"))?;
    let sid = account_sid(&system)?;
    windowless(
        &system,
        "icacls.exe",
        at,
        &["/grant:r", &format!("*{sid}:(OI)(CI)F")],
    )?;
    windowless(&system, "icacls.exe", at, &["/inheritance:r"])
}

// The account the process runs as, never the name the environment claims.
#[cfg(windows)]
fn account_sid(system: &Path) -> std::io::Result<String> {
    let out = quiet(system, "whoami.exe")
        .args(["/user", "/fo", "csv", "/nh"])
        .output()?;
    String::from_utf8_lossy(&out.stdout)
        .split(',')
        .map(|one| one.trim().trim_matches('"').to_string())
        .find(|one| one.starts_with("S-1-") && one.len() > 4)
        .ok_or_else(|| std::io::Error::other("this account's SID could not be read"))
}

#[cfg(windows)]
fn windowless(system: &Path, tool: &str, at: &Path, args: &[&str]) -> std::io::Result<()> {
    let done = quiet(system, tool).arg(at).args(args).status()?;
    match done.success() {
        true => Ok(()),
        false => Err(std::io::Error::other(format!("{tool} ended with {done}"))),
    }
}

#[cfg(windows)]
fn quiet(system: &Path, tool: &str) -> std::process::Command {
    use std::os::windows::process::CommandExt;
    let mut command = std::process::Command::new(system.join(tool));
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(0x0800_0000);
    command
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
