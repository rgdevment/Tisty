#![cfg_attr(not(windows), allow(dead_code))]

//! Plugins start the MCP server as `tisty`; this keeps the installer's copy on the PATH while assistants have a voice.

use semver::Version;

const HERE: &str = env!("CARGO_PKG_VERSION");
// Only a developer's build carries the version the release has not stamped yet.
const UNSTAMPED: &str = "0.1.0";
const SEPARATOR: char = ';';

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Due {
    Untouched,
    Out,
    In { ahead_of_the_store: bool },
}

/// Leaving the PATH is always safe; only a steady copy, newer than the Store's, with a voice joins it.
pub fn due(
    steady: bool,
    voice: bool,
    standalone: Option<&Version>,
    store: Option<&Version>,
) -> Due {
    let Some(standalone) = standalone.filter(|it| it.pre.is_empty()) else {
        return Due::Out;
    };
    if !voice || store.is_some_and(|store| store > standalone) {
        return Due::Out;
    }
    if !steady {
        return Due::Untouched;
    }
    Due::In {
        ahead_of_the_store: store.is_some_and(|store| standalone > store),
    }
}

/// A developer's build or a release candidate never makes `tisty` answer with itself.
pub fn steady() -> bool {
    !cfg!(debug_assertions)
        && HERE != UNSTAMPED
        && Version::parse(HERE).is_ok_and(|here| here.pre.is_empty())
}

/// `%NAME%` as Windows would expand it, so an entry written either way is recognised.
pub fn expanded(one: &str) -> String {
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

fn tidy(one: &str, expand: &dyn Fn(&str) -> String) -> String {
    expand(one.trim())
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

fn alike(a: &str, b: &str, expand: &dyn Fn(&str) -> String) -> bool {
    !a.trim().is_empty() && tidy(a, expand) == tidy(b, expand)
}

/// The PATH with `dir` at the end, or just before `ahead_of`; `None` when it already sits there.
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

#[cfg(windows)]
pub fn keep(paths: tisty_core::Paths) {
    windows_side::keep(&paths);
}

#[cfg(not(windows))]
pub fn keep(_paths: tisty_core::Paths) {}

#[cfg(windows)]
mod windows_side {
    use std::sync::Mutex;

    use semver::Version;
    use tisty_core::witness::{self, Fact, channel};

    use super::{Due, HERE, due, expanded, steady, within, without};

    const PACKAGED_AS: &str = "rgdevment.Tisty";
    // A start and a quick turn of the voice would otherwise race, and the older choice could write last.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

    pub fn keep(paths: &tisty_core::Paths) {
        let _turn = ONE_AT_A_TIME
            .lock()
            .unwrap_or_else(|held| held.into_inner());
        // A sandbox profile, or the Store copy whose registry writes Windows keeps in its own container.
        if tisty_core::paths::profile().is_some()
            || windows::ApplicationModel::Package::Current().is_ok()
        {
            return;
        }
        let Some(folder) = crate::command::beside() else {
            return;
        };
        // Read now, not when the thread was asked for, so the last choice is the one written.
        let voice = tisty_core::Config::load(&paths.config_file())
            .ok()
            .flatten()
            .is_some_and(|config| config.agent_id.is_some());
        let steady = steady();
        let store = (voice && steady).then(store).flatten();
        // An unreadable PATH is left alone rather than taken for an empty one.
        let Some(path) = crate::command::user_path() else {
            return;
        };
        let dir = folder.display().to_string();
        let changed = match due(
            steady,
            voice,
            Version::parse(HERE).ok().as_ref(),
            store.as_ref(),
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

    fn store() -> Option<Version> {
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
