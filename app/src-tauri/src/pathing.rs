#![cfg_attr(not(windows), allow(dead_code))]

//! Plugins start the MCP server as `tisty`; this keeps the installer's copy on the PATH while assistants have a voice.

use semver::Version;

const HERE: &str = env!("CARGO_PKG_VERSION");
// Only a developer's build carries the version the release has not stamped yet.
const UNSTAMPED: &str = "0.1.0";
const SEPARATOR: char = ';';

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Due {
    Out,
    In,
}

/// Leaving the PATH is always safe; only a stable copy with a voice, newer than any other, joins it.
pub fn due(voice: bool, standalone: Option<&Version>, newest_other: Option<&Version>) -> Due {
    let Some(standalone) = standalone.filter(|it| it.pre.is_empty()) else {
        return Due::Out;
    };
    if !voice || newest_other.is_some_and(|other| other > standalone) {
        return Due::Out;
    }
    Due::In
}

/// The first entry holding an older copy, which this one has to come before; an equal one keeps its place.
pub fn first_older<'a>(others: &'a [(String, Version)], here: &Version) -> Option<&'a str> {
    others
        .iter()
        .find(|(_, other)| other < here)
        .map(|(entry, _)| entry.as_str())
}

/// A developer's build never touches the PATH, not even to take a folder of the developer's own off it.
pub fn developer() -> bool {
    cfg!(debug_assertions) || HERE == UNSTAMPED
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
    // Empty entries stay, so taking the folder out again gives back the PATH as it was spelled.
    let mut entries: Vec<&str> = match path.is_empty() {
        true => Vec::new(),
        false => path.split(SEPARATOR).collect(),
    };
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

// Leaving does not wait for a `keep` already under way, so that one checks this after it writes.
static LEAVING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn leaving() {
    LEAVING.store(true, std::sync::atomic::Ordering::SeqCst);
}

fn left() -> bool {
    LEAVING.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(windows)]
pub fn keep(paths: tisty_core::Paths) {
    windows_side::keep(&paths);
}

#[cfg(not(windows))]
pub fn keep(_paths: tisty_core::Paths) {}

#[cfg(windows)]
pub use windows_side::announce;

#[cfg(windows)]
mod windows_side {
    use std::sync::Mutex;

    use semver::Version;
    use tisty_core::witness::{self, Fact, channel};

    use super::{
        Due, HERE, SEPARATOR, alike, developer, due, expanded, first_older, left, within, without,
    };

    // A start and a quick turn of the voice would otherwise race, and the older choice could write last.
    static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

    pub fn keep(paths: &tisty_core::Paths) {
        let _turn = ONE_AT_A_TIME
            .lock()
            .unwrap_or_else(|held| held.into_inner());
        // A sandbox profile, or the Store copy whose registry writes Windows keeps in its own container.
        if left()
            || developer()
            || tisty_core::paths::profile().is_some()
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
        // An unreadable PATH is left alone rather than taken for an empty one.
        let Some(path) = crate::command::user_path() else {
            return;
        };
        let dir = folder.display().to_string();
        let here = Version::parse(HERE).ok();
        let store = voice.then(store).flatten();
        let others = match voice {
            true => copies_on(&path, &dir, store.as_ref()),
            false => Vec::new(),
        };
        let newest_other = others.iter().map(|(_, it)| it).chain(&store).max();
        let changed = match due(voice, here.as_ref(), newest_other) {
            Due::Out => without(&path, &dir, &expanded)
                .map(|next| (next, "the PATH no longer reaches this copy")),
            Due::In => {
                let ahead = here.as_ref().and_then(|here| first_older(&others, here));
                within(&path, &dir, ahead, &expanded)
                    .map(|next| (next, "the PATH now reaches the copy assistants should call"))
            }
        };
        let Some((next, said)) = changed else { return };
        match crate::command::set_user_path(&next) {
            Ok(()) if left() => {
                let _ = crate::command::out_of_reach();
            }
            Ok(()) => {
                announce();
                witness::note(channel::TERMINAL, said, &[("at", Fact::Path(folder))]);
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
        let this_user = windows::core::HSTRING::new();
        let family = windows::core::HSTRING::from(tisty_core::paths::STORE_PACKAGE);
        shelf
            .FindPackagesByUserSecurityIdPackageFamilyName(&this_user, &family)
            .ok()?
            .into_iter()
            .filter_map(|one| {
                let v = one.Id().ok()?.Version().ok()?;
                Some(Version::new(v.Major.into(), v.Minor.into(), v.Build.into()))
            })
            .max()
    }

    /// The other copies on the PATH, in its order: the Store's alias folder and every installed copy.
    fn copies_on(path: &str, dir: &str, store: Option<&Version>) -> Vec<(String, Version)> {
        let alias = crate::command::aliases().map(|at| at.display().to_string());
        path.split(SEPARATOR)
            .filter(|one| !one.trim().is_empty() && !alike(one, dir, &expanded))
            .filter_map(|one| {
                let found = match alias.as_deref() {
                    Some(alias) if alike(one, alias, &expanded) => store.cloned(),
                    _ => installed_copy(std::path::Path::new(&expanded(one.trim()))),
                };
                found.map(|version| (one.to_string(), version))
            })
            .collect()
    }

    // The window beside the command is what marks a folder as a Tisty install and carries its version.
    fn installed_copy(folder: &std::path::Path) -> Option<Version> {
        if !folder.join(crate::command::CLI).is_file() {
            return None;
        }
        version_of(&folder.join("tisty-gui.exe"))
    }

    #[allow(unsafe_code)]
    fn version_of(exe: &std::path::Path) -> Option<Version> {
        use windows::Win32::Storage::FileSystem::{
            GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
        };
        let name = windows::core::HSTRING::from(exe.as_os_str());
        let mut fixed: *mut core::ffi::c_void = std::ptr::null_mut();
        let mut length = 0u32;
        unsafe {
            let size = GetFileVersionInfoSizeW(&name, None);
            if size == 0 {
                return None;
            }
            let mut block = vec![0u8; size as usize];
            GetFileVersionInfoW(&name, None, size, block.as_mut_ptr().cast()).ok()?;
            let found = VerQueryValueW(
                block.as_ptr().cast(),
                windows::core::w!("\\"),
                &mut fixed,
                &mut length,
            );
            if !found.as_bool()
                || fixed.is_null()
                || (length as usize) < size_of::<VS_FIXEDFILEINFO>()
            {
                return None;
            }
            let fixed = std::ptr::read_unaligned(fixed as *const VS_FIXEDFILEINFO);
            Some(Version::new(
                (fixed.dwFileVersionMS >> 16).into(),
                (fixed.dwFileVersionMS & 0xffff).into(),
                (fixed.dwFileVersionLS >> 16).into(),
            ))
        }
    }

    /// Without this only programs started after the next sign-in would see the change.
    #[allow(unsafe_code)]
    pub fn announce() {
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
