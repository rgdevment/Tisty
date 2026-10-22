#![cfg_attr(not(windows), allow(dead_code))]

//! A plugin starts the MCP server by the name `tisty`, so the copy installed by hand keeps
//! its folder on the person's PATH while assistants have a voice. The Store's execution alias
//! and Homebrew's link already answer to that name; this only places the standalone copy.

use semver::Version;

const HERE: &str = env!("CARGO_PKG_VERSION");
/// What a build carries until the release stamps it: only a developer's copy says this.
const UNSTAMPED: &str = "0.1.0";
const SEPARATOR: char = ';';

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Due {
    Untouched,
    Out,
    In { ahead_of_the_store: bool },
}

/// One rule for every copy, wherever it came from: the newest stable copy is the one `tisty`
/// answers with, and with the door shut no folder of ours stays on the PATH.
pub fn due(
    steady: bool,
    voice: bool,
    standalone: Option<&Version>,
    store: Option<&Version>,
) -> Due {
    let Some(standalone) = standalone else {
        return Due::Untouched;
    };
    if !steady {
        return Due::Untouched;
    }
    if !voice || !standalone.pre.is_empty() || store.is_some_and(|store| store > standalone) {
        return Due::Out;
    }
    Due::In {
        ahead_of_the_store: store.is_some_and(|store| standalone > store),
    }
}

/// Only a stamped, stable release writes to the PATH: a developer's build, a release candidate
/// or a sandbox profile would otherwise leave the person's assistants calling a passing copy.
pub fn steady(profile: Option<&str>) -> bool {
    !cfg!(debug_assertions)
        && profile.is_none()
        && HERE != UNSTAMPED
        && Version::parse(HERE).is_ok_and(|here| here.pre.is_empty())
}

fn tidy(one: &str, expand: &dyn Fn(&str) -> String) -> String {
    expand(one.trim())
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

fn alike(a: &str, b: &str, expand: &dyn Fn(&str) -> String) -> bool {
    !a.trim().is_empty() && tidy(a, expand) == tidy(b, expand)
}

/// The PATH with `dir` in it: at the end, so what the person put there first still wins, or just
/// before `ahead_of` when this copy has to answer before an older one. `None` when it already sits
/// where it should, so nothing is written and nobody is told.
pub fn within(
    path: &str,
    dir: &str,
    ahead_of: Option<&str>,
    expand: &dyn Fn(&str) -> String,
) -> Option<String> {
    let mut entries: Vec<&str> = path
        .split(SEPARATOR)
        .filter(|one| !one.is_empty())
        .collect();
    let mine = entries.iter().position(|one| alike(one, dir, expand));
    let theirs =
        ahead_of.and_then(|other| entries.iter().position(|one| alike(one, other, expand)));
    match (mine, theirs) {
        (Some(mine), Some(theirs)) if mine > theirs => {
            let kept = entries.remove(mine);
            entries.insert(theirs, kept);
        }
        (Some(_), _) => return None,
        (None, Some(theirs)) => entries.insert(theirs, dir),
        (None, None) => entries.push(dir),
    }
    Some(entries.join(&SEPARATOR.to_string()))
}

/// The PATH without `dir`, or `None` when it was never there.
pub fn without(path: &str, dir: &str, expand: &dyn Fn(&str) -> String) -> Option<String> {
    let entries: Vec<&str> = path.split(SEPARATOR).collect();
    if !entries.iter().any(|one| alike(one, dir, expand)) {
        return None;
    }
    let kept: Vec<&str> = entries
        .into_iter()
        .filter(|one| !alike(one, dir, expand))
        .collect();
    Some(kept.join(&SEPARATOR.to_string()))
}

/// What `tisty --version` says, read back as a version.
pub fn said(answer: &str) -> Option<Version> {
    Version::parse(answer.trim().strip_prefix("tisty ")?.trim()).ok()
}

#[cfg(windows)]
pub fn keep(voice: bool) {
    windows_side::keep(voice);
}

#[cfg(not(windows))]
pub fn keep(_voice: bool) {}

#[cfg(windows)]
mod windows_side {
    use std::path::{Path, PathBuf};

    use semver::Version;
    use tisty_core::witness::{self, Fact, channel};

    use super::{Due, HERE, due, said, steady, within, without};

    /// The identity the MSIX is published under; the Store copy is found by it.
    const PACKAGED_AS: &str = "rgdevment.Tisty";
    /// Where the installer records the folder it put Tisty in.
    const INSTALLED_AT: &str = r"Software\rgdevment\Tisty";
    /// Hidden window, so asking the other copy its version flashes nothing.
    const QUIETLY: u32 = 0x0800_0000;

    pub fn keep(voice: bool) {
        let profile = tisty_core::paths::profile();
        let steady = steady(profile.as_deref());
        let Some(folder) = standalone() else { return };
        let standalone_version = version_of(&folder);
        let store_version = store();
        let path = crate::command::user_path().unwrap_or_default();
        let dir = folder.display().to_string();
        let changed = match due(
            steady,
            voice,
            standalone_version.as_ref(),
            store_version.as_ref(),
        ) {
            Due::Untouched => None,
            Due::Out => without(&path, &dir, &expanded),
            Due::In { ahead_of_the_store } => {
                let alias = ahead_of_the_store.then(alias_folder).flatten();
                within(&path, &dir, alias.as_deref(), &expanded)
            }
        };
        let Some(next) = changed else { return };
        match crate::command::set_user_path(&next) {
            Ok(()) => {
                announce();
                witness::note(
                    channel::TERMINAL,
                    "the PATH now reaches the copy assistants should call",
                    &[("at", Fact::Path(folder))],
                );
            }
            Err(why) => witness::warn(
                channel::TERMINAL,
                "the PATH could not be brought in line",
                &[("why", Fact::Why(why.to_string()))],
            ),
        }
    }

    /// The copy installed by hand: this one, or the one the installer recorded when this is the Store's.
    fn standalone() -> Option<PathBuf> {
        if let Some(here) = crate::command::beside() {
            return Some(here);
        }
        use winreg::enums::HKEY_CURRENT_USER;
        let recorded: String = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(INSTALLED_AT)
            .ok()?
            .get_value("")
            .ok()?;
        let folder = PathBuf::from(recorded);
        folder.join(crate::command::CLI).is_file().then_some(folder)
    }

    fn version_of(folder: &Path) -> Option<Version> {
        if crate::command::beside().as_deref() == Some(folder) {
            return Version::parse(HERE).ok();
        }
        use std::os::windows::process::CommandExt;
        let answer = std::process::Command::new(folder.join(crate::command::CLI))
            .arg("--version")
            .creation_flags(QUIETLY)
            .output()
            .ok()?;
        said(&String::from_utf8_lossy(&answer.stdout))
    }

    fn store() -> Option<Version> {
        if windows::ApplicationModel::Package::Current().is_ok() {
            return Version::parse(HERE).ok();
        }
        let shelf = windows::Management::Deployment::PackageManager::new().ok()?;
        let everyone_of_mine = windows::core::HSTRING::new();
        shelf
            .FindPackagesByUserSecurityId(&everyone_of_mine)
            .ok()?
            .into_iter()
            .filter_map(|one| {
                let id = one.Id().ok()?;
                (id.Name().ok()? == PACKAGED_AS).then_some(())?;
                let v = id.Version().ok()?;
                Some(Version::new(v.Major.into(), v.Minor.into(), v.Build.into()))
            })
            .max()
    }

    fn alias_folder() -> Option<String> {
        let local = std::env::var("LOCALAPPDATA").ok()?;
        Some(format!(r"{local}\Microsoft\WindowsApps"))
    }

    /// The registry keeps `%USERPROFILE%\…` unexpanded, and the alias folder is usually written that way.
    fn expanded(one: &str) -> String {
        let mut out = String::new();
        let mut rest = one;
        while let Some(open) = rest.find('%') {
            let Some(close) = rest[open + 1..].find('%') else {
                break;
            };
            let name = &rest[open + 1..open + 1 + close];
            out.push_str(&rest[..open]);
            match std::env::var(name) {
                Ok(value) => out.push_str(&value),
                Err(_) => out.push_str(&rest[open..open + close + 2]),
            }
            rest = &rest[open + close + 2..];
        }
        out.push_str(rest);
        out
    }

    /// Without this only programs started after the next sign-in would see the change.
    #[allow(unsafe_code)]
    fn announce() {
        use windows::Win32::Foundation::{LPARAM, WPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
        };
        let what: Vec<u16> = "Environment\0".encode_utf16().collect();
        unsafe {
            let _ = SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                WPARAM(0),
                LPARAM(what.as_ptr() as isize),
                SMTO_ABORTIFHUNG,
                5000,
                None,
            );
        }
    }
}

#[cfg(test)]
#[path = "pathing_test.rs"]
mod tests;
