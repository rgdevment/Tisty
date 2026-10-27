use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    Windows,
    Macos,
    Linux,
}

/// Where assistants live: this computer, or one of its WSL distros, with how Tisty is reached from there.
#[derive(Debug, Clone)]
pub struct Place {
    pub distro: Option<String>,
    pub os: Os,
    pub home: PathBuf,
    pub config: PathBuf,
    pub appdata: Option<PathBuf>,
    pub local: Option<PathBuf>,
    pub bins: Vec<PathBuf>,
    pub command: String,
    pub open: bool,
    /// Where the distro mounts Windows drives, `/mnt/` by default; empty on this computer.
    pub mount: String,
}

impl Place {
    pub fn here(command: String) -> Option<Self> {
        let os = match () {
            _ if cfg!(windows) => Os::Windows,
            _ if cfg!(target_os = "macos") => Os::Macos,
            _ => Os::Linux,
        };
        let home = var(if os == Os::Windows {
            "USERPROFILE"
        } else {
            "HOME"
        })?;
        let config = var("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
        let mut bins: Vec<PathBuf> = std::env::var_os("PATH")
            .map(|all| std::env::split_paths(&all).collect())
            .unwrap_or_default();
        bins.extend(usual_bins(os, &home));
        Some(Self {
            distro: None,
            os,
            appdata: (os == Os::Windows).then(|| var("APPDATA")).flatten(),
            local: (os == Os::Windows).then(|| var("LOCALAPPDATA")).flatten(),
            home,
            config,
            bins,
            command,
            open: true,
            mount: String::new(),
        })
    }

    pub fn resolve(&self, said: &str) -> Option<PathBuf> {
        let (base, rest) = if let Some(rest) = said.strip_prefix("~/") {
            (Some(self.home.clone()), rest)
        } else if let Some(rest) = said.strip_prefix("appdata:") {
            (self.appdata.clone(), rest)
        } else if let Some(rest) = said.strip_prefix("localappdata:") {
            (self.local.clone(), rest)
        } else if let Some(rest) = said.strip_prefix("config:") {
            (Some(self.config.clone()), rest)
        } else if let Some(rest) = said.strip_prefix("support:") {
            let base =
                (self.os == Os::Macos).then(|| self.home.join("Library/Application Support"));
            (base, rest)
        } else if let Some(rest) = said.strip_prefix("store:") {
            let (prefix, rest) = rest.split_once(':')?;
            (self.local.as_deref().and_then(|at| store(at, prefix)), rest)
        } else if said.starts_with('/') {
            return (self.os == Os::Macos && self.distro.is_none()).then(|| PathBuf::from(said));
        } else {
            return None;
        };
        let mut at = base?;
        for step in rest.split('/') {
            at = match step.strip_suffix('*') {
                Some(named) => first_in(&at, named)?,
                None => at.join(step),
            };
        }
        Some(at)
    }

    // A bare name is looked up on the PATH when the assistant starts it, so there is no file to check.
    pub fn reached(&self, said: &str) -> Option<PathBuf> {
        let said = said.trim();
        let Some(distro) = &self.distro else {
            let at = PathBuf::from(said);
            return at.is_absolute().then_some(at);
        };
        let drive = said
            .strip_prefix(self.mount.trim_end_matches('/'))
            .and_then(|rest| rest.strip_prefix('/'))
            .and_then(|rest| rest.split_once('/'))
            .filter(|(drive, _)| drive.len() == 1);
        if let Some((drive, rest)) = drive {
            let drive = drive.to_ascii_uppercase();
            return Some(PathBuf::from(format!(
                "{drive}:\\{}",
                rest.replace('/', "\\")
            )));
        }
        let rest = said.strip_prefix('/')?;
        Some(
            rest.split('/')
                .fold(distro_root(distro), |at, step| at.join(step)),
        )
    }

    pub fn runs(&self, name: &str) -> bool {
        let named: &[&str] = match self.os {
            Os::Windows => &[".exe", ".cmd", ".bat", ""],
            _ => &[""],
        };
        self.bins.iter().any(|dir| {
            named
                .iter()
                .any(|end| dir.join(format!("{name}{end}")).is_file())
        })
    }
}

/// An app opened from the Finder gets a bare PATH, without where Homebrew, npm or pipx put a command.
pub fn usual_bins(os: Os, home: &Path) -> Vec<PathBuf> {
    if os == Os::Windows {
        return Vec::new();
    }
    let system: &[&str] = match os {
        Os::Macos => &["/opt/homebrew/bin", "/usr/local/bin"],
        _ => &[],
    };
    let mut found: Vec<PathBuf> = system
        .iter()
        .map(PathBuf::from)
        .chain(
            [
                ".local/bin",
                ".npm-global/bin",
                ".bun/bin",
                ".volta/bin",
                ".cargo/bin",
            ]
            .iter()
            .map(|one| home.join(one)),
        )
        .collect();
    let nvm = home.join(".nvm").join("versions").join("node");
    let versions: Vec<PathBuf> = std::fs::read_dir(nvm)
        .map(|all| all.flatten().map(|one| one.path().join("bin")).collect())
        .unwrap_or_default();
    found.extend(versions);
    found
}

fn var(named: &str) -> Option<PathBuf> {
    std::env::var_os(named)
        .filter(|it| !it.is_empty())
        .map(PathBuf::from)
}

/// Installed from the Store an app never sees `%APPDATA%`: Windows hands it a private copy under
/// its package, and the settings the person edits are the ones in there.
fn store(local: &Path, named: &str) -> Option<PathBuf> {
    Some(
        first_in(&local.join("Packages"), named)?
            .join("LocalCache")
            .join("Roaming"),
    )
}

fn first_in(folder: &Path, named: &str) -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(folder)
        .ok()?
        .flatten()
        .map(|it| it.path())
        .filter(|at| {
            at.is_dir()
                && at
                    .file_name()
                    .is_some_and(|it| it.to_string_lossy().starts_with(named))
        })
        .collect();
    found.sort();
    found.into_iter().next()
}

/// The same Windows file as a WSL distro reaches it: `C:\a\b` under its mount root, `/mnt/c/a/b`.
pub fn from_linux(windows: &str, root: &str) -> Option<String> {
    let (drive, rest) = windows.split_once(":\\")?;
    let drive = drive.to_ascii_lowercase();
    if drive.len() != 1 {
        return None;
    }
    let root = root.trim_end_matches('/');
    Some(format!("{root}/{drive}/{}", rest.replace('\\', "/")))
}

/// What `/etc/wsl.conf` says about where drives are mounted and whether Windows programs may run.
pub fn told_by(conf: &str) -> (String, bool) {
    let mut root = "/mnt/".to_string();
    let mut open = true;
    for (part, key, said) in settings(conf) {
        match (part.as_str(), key.as_str()) {
            ("automount", "root") => root = said,
            ("interop", "enabled") => open = !said.eq_ignore_ascii_case("false"),
            _ => {}
        }
    }
    (root, open)
}

/// `[user] default`, which WSL logs in as before the uid the registry keeps, often root's.
pub fn user_in(conf: &str) -> Option<String> {
    settings(conf)
        .into_iter()
        .find(|(part, key, said)| part == "user" && key == "default" && !said.is_empty())
        .map(|(_, _, said)| said)
}

fn settings(conf: &str) -> Vec<(String, String, String)> {
    let mut part = String::new();
    let mut found = Vec::new();
    for line in conf.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if let Some(named) = line.strip_prefix('[').and_then(|it| it.strip_suffix(']')) {
            part = named.trim().to_ascii_lowercase();
            continue;
        }
        let Some((key, said)) = line.split_once('=') else {
            continue;
        };
        let said = said.trim().trim_matches(['"', '\'']).to_string();
        found.push((part.clone(), key.trim().to_ascii_lowercase(), said));
    }
    found
}

/// The home of the distro's default user, read from its own `/etc/passwd`.
pub fn home_in(passwd: &str, uid: u32) -> Option<String> {
    home_where(passwd, |parts| parts[2] == uid.to_string())
}

pub fn home_of(passwd: &str, user: &str) -> Option<String> {
    home_where(passwd, |parts| parts[0] == user)
}

fn home_where(passwd: &str, wanted: impl Fn(&[&str]) -> bool) -> Option<String> {
    passwd.lines().find_map(|line| {
        let parts: Vec<&str> = line.split(':').collect();
        (parts.len() > 5 && wanted(&parts)).then(|| parts[5].to_string())
    })
}

#[cfg(windows)]
pub fn distros() -> Vec<(String, u32)> {
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;
    let Ok(lxss) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Lxss")
    else {
        return Vec::new();
    };
    let mut found: Vec<(String, u32)> = lxss
        .enum_keys()
        .flatten()
        .filter_map(|guid| {
            let one = lxss.open_subkey(&guid).ok()?;
            let name: String = one.get_value("DistributionName").ok()?;
            let uid: u32 = one.get_value("DefaultUid").unwrap_or(1000);
            Some((name, uid))
        })
        // Docker's own distros run its engine, never a person's assistants.
        .filter(|(name, _)| !name.starts_with("docker-desktop"))
        .collect();
    found.sort();
    found
}

#[cfg(not(windows))]
pub fn distros() -> Vec<(String, u32)> {
    Vec::new()
}

pub fn distro_root(name: &str) -> PathBuf {
    PathBuf::from(format!(r"\\wsl.localhost\{name}"))
}

/// A distro as a place: reading its files over `\\wsl.localhost` starts it when it was stopped.
pub fn distro(name: &str, uid: u32, windows_command: &str) -> Option<Place> {
    let root = distro_root(name);
    let passwd = std::fs::read_to_string(root.join("etc").join("passwd")).ok()?;
    let conf = std::fs::read_to_string(root.join("etc").join("wsl.conf")).unwrap_or_default();
    let linux_home = user_in(&conf)
        .and_then(|user| home_of(&passwd, &user))
        .or_else(|| home_in(&passwd, uid))?;
    let (mount, open) = told_by(&conf);
    let home = linux_home
        .trim_start_matches('/')
        .split('/')
        .fold(root.clone(), |at, step| at.join(step));
    let bins = ["usr/bin", "usr/local/bin", "bin"]
        .iter()
        .map(|dir| dir.split('/').fold(root.clone(), |at, step| at.join(step)))
        .chain([home.join(".local").join("bin"), home.join("bin")])
        .collect();
    Some(Place {
        distro: Some(name.to_string()),
        os: Os::Linux,
        config: home.join(".config"),
        appdata: None,
        local: None,
        bins,
        command: from_linux(windows_command, &mount)?,
        home,
        open,
        mount,
    })
}

#[cfg(test)]
#[path = "places_test.rs"]
mod tests;
