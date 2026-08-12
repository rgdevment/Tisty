//! Deciding the new PATH is kept apart from writing it, so the part that once
//! destroyed one can be tested without a registry.

use std::path::{Path, PathBuf};

use tisty_core::witness::{self, Fact, channel};

fn unwritten(why: &std::io::Error) {
    witness::error(
        channel::TERMINAL,
        "the command could not be put within reach",
        &[("why", Fact::Why(why.to_string()))],
    );
}

#[cfg(windows)]
fn same(a: &str, b: &str) -> bool {
    let tidy = |one: &str| {
        let one = one.trim().trim_end_matches(['\\', '/']).to_string();
        if cfg!(windows) {
            one.to_lowercase()
        } else {
            one
        }
    };
    !a.trim().is_empty() && tidy(a) == tidy(b)
}

/// `None` when it is already there, so a reinstall cannot grow the variable.
#[cfg(windows)]
pub fn with(path: &str, dir: &str) -> Option<String> {
    if path.split(SEPARATOR).any(|one| same(one, dir)) {
        return None;
    }
    Some(if path.trim().is_empty() {
        dir.to_string()
    } else {
        format!("{}{SEPARATOR}{dir}", path.trim_end_matches(SEPARATOR))
    })
}

/// `None` when it was not there. Everything else keeps its order and spelling.
#[cfg(windows)]
pub fn without(path: &str, dir: &str) -> Option<String> {
    if !path.split(SEPARATOR).any(|one| same(one, dir)) {
        return None;
    }
    let kept: Vec<&str> = path
        .split(SEPARATOR)
        .filter(|one| !same(one, dir))
        .collect();
    Some(kept.join(&SEPARATOR.to_string()))
}

// Windows only, like everything it separates: elsewhere `tisty` is reached by
// a symlink and no PATH is ever edited.
#[cfg(windows)]
const SEPARATOR: char = ';';

pub fn beside() -> Option<PathBuf> {
    let here = std::env::current_exe().ok()?;
    let folder = here.parent()?;
    let named = if cfg!(windows) { "tisty.exe" } else { "tisty" };
    folder.join(named).is_file().then(|| folder.to_path_buf())
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reach {
    /// False when the build has no `tisty` beside it — a dev run, mostly.
    pub shipped: bool,
    pub within_reach: bool,
    pub at: Option<String>,
    pub through: Option<String>,
    /// False when the link exists but nothing would find it. macOS builds its
    /// PATH from `/etc/paths`, which does not include `~/.local/bin`, so «done»
    /// and «works» are not the same answer there.
    pub on_path: bool,
}

pub fn reach() -> Reach {
    let folder = beside();
    Reach {
        // Inside a container the link would be made, reported as ready, and
        // found by no shell on the machine. Better to not offer it at all.
        shipped: folder.is_some(),
        within_reach: folder.as_deref().is_some_and(already),
        at: folder.map(|at| at.display().to_string()),
        through: through(),
        on_path: on_path(),
    }
}

/// Whether the directory the command ends up in is one the shell searches.
#[cfg(windows)]
fn on_path() -> bool {
    true
}

#[cfg(not(windows))]
fn on_path() -> bool {
    let Some(shelf) = shelf() else {
        return false;
    };
    std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).any(|one| one == shelf))
        .unwrap_or(false)
}

#[cfg(windows)]
fn through() -> Option<String> {
    beside().map(|at| at.display().to_string())
}

#[cfg(not(windows))]
fn through() -> Option<String> {
    link().map(|at| at.display().to_string())
}

#[cfg(windows)]
fn already(folder: &Path) -> bool {
    let named = folder.display().to_string();
    read()
        .unwrap_or_default()
        .split(SEPARATOR)
        .any(|one| same(one, &named))
}

#[cfg(not(windows))]
fn already(_folder: &Path) -> bool {
    ours()
}

#[cfg(windows)]
const WHERE: &str = "Environment";

#[cfg(windows)]
fn read() -> Option<String> {
    read_from(WHERE)
}

#[cfg(windows)]
fn read_from(at: &str) -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(at)
        .ok()?;
    key.get_value::<String, _>("Path").ok()
}

#[cfg(windows)]
fn write(value: &str) -> std::io::Result<()> {
    write_to(WHERE, value)
}

#[cfg(windows)]
fn write_to(at: &str, value: &str) -> std::io::Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE, REG_EXPAND_SZ};
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(at, KEY_WRITE)?;
    // Expandable, like the one Windows itself writes: plain would freeze any
    // `%USERPROFILE%` the person has in there.
    let mut held = winreg::RegValue {
        bytes: Vec::new(),
        vtype: REG_EXPAND_SZ,
    };
    held.bytes = value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect();
    key.set_raw_value("Path", &held)?;
    Ok(())
}

/// A link, not the PATH: that lives in whichever shell file the person uses,
/// and editing those by guesswork is worse than not offering it.
#[cfg(not(windows))]
fn shelf() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("bin"))
}

#[cfg(not(windows))]
fn link() -> Option<PathBuf> {
    Some(shelf()?.join("tisty"))
}

/// Ours means a symlink pointing at the binary beside this window. A real file
/// there is somebody else's `tisty` — a tarball copy, a `cargo install` — and
/// removing it would take a program we never put there.
#[cfg(not(windows))]
fn ours() -> bool {
    let (Some(at), Some(folder)) = (link(), beside()) else {
        return false;
    };
    at.symlink_metadata().is_ok_and(|it| it.is_symlink())
        && std::fs::read_link(&at).is_ok_and(|to| to == folder.join("tisty"))
}

#[cfg(not(windows))]
fn taken() -> bool {
    link().is_some_and(|at| at.symlink_metadata().is_ok())
}

#[cfg(not(windows))]
fn tie(wanted: bool) -> std::io::Result<bool> {
    let (Some(shelf), Some(link), Some(folder)) = (shelf(), link(), beside()) else {
        return Ok(false);
    };
    if wanted {
        if ours() {
            return Ok(false);
        }
        // A link of ours pointing elsewhere is stale — the app moved, or macOS
        // ran it from a translocated copy — and repointing it is the repair.
        if taken() {
            if !is_our_own_link(&link) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    link.display().to_string(),
                ));
            }
            std::fs::remove_file(&link)?;
        }
        std::fs::create_dir_all(&shelf)?;
        std::os::unix::fs::symlink(folder.join("tisty"), &link)?;
    } else {
        if !ours() {
            return Ok(false);
        }
        std::fs::remove_file(&link)?;
    }
    Ok(true)
}

/// A symlink whose target is a `tisty` of ours, wherever this build now lives.
#[cfg(not(windows))]
fn is_our_own_link(at: &Path) -> bool {
    at.symlink_metadata().is_ok_and(|it| it.is_symlink())
        && std::fs::read_link(at).is_ok_and(|to| to.file_name().is_some_and(|n| n == "tisty"))
}

/// True when the change went in.
#[cfg(windows)]
pub fn within_reach(wanted: bool) -> std::io::Result<bool> {
    let Some(folder) = beside() else {
        return Ok(false);
    };
    let named = folder.display().to_string();
    let path = read().unwrap_or_default();

    let next = if wanted {
        with(&path, &named)
    } else {
        without(&path, &named)
    };
    match next {
        Some(next) => {
            write(&next).inspect_err(unwritten)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

#[cfg(not(windows))]
pub fn within_reach(wanted: bool) -> std::io::Result<bool> {
    tie(wanted).inspect_err(unwritten)
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    fn joined(parts: &[&str]) -> String {
        parts.join(&SEPARATOR.to_string())
    }

    #[test]
    fn a_long_path_survives_the_round_trip() {
        let many: Vec<String> = (0..22)
            .map(|i| format!("C:\\Users\\Someone\\A rather long program directory number {i}"))
            .collect();
        let path = many.join(&SEPARATOR.to_string());
        assert!(path.len() > 1024, "the case only bites past 1024");

        let added = with(&path, "C:\\Programs\\Tisty").unwrap();
        assert_eq!(added.split(SEPARATOR).count(), 23);

        let back = without(&added, "C:\\Programs\\Tisty").unwrap();
        assert_eq!(back, path, "the other entries did not come back");
    }

    #[test]
    fn adding_twice_does_not_grow_it() {
        let path = joined(&["C:\\one", "C:\\two"]);
        let added = with(&path, "C:\\Tisty").unwrap();
        assert!(with(&added, "C:\\Tisty").is_none());
    }

    #[test]
    fn removing_what_was_never_there_changes_nothing() {
        let path = joined(&["C:\\one", "C:\\two"]);
        assert!(without(&path, "C:\\Tisty").is_none());
    }

    #[test]
    fn an_empty_path_is_not_a_missing_one() {
        assert_eq!(with("", "C:\\Tisty").unwrap(), "C:\\Tisty");
        assert!(without("", "C:\\Tisty").is_none());
    }

    #[test]
    fn a_trailing_separator_does_not_become_a_blank_entry() {
        let path = format!("C:\\one{SEPARATOR}");
        let added = with(&path, "C:\\Tisty").unwrap();
        assert_eq!(added, joined(&["C:\\one", "C:\\Tisty"]));
    }

    #[test]
    fn a_trailing_slash_is_the_same_directory() {
        let path = joined(&["C:\\one", "C:\\Tisty\\"]);
        assert!(with(&path, "C:\\Tisty").is_none());
        assert_eq!(without(&path, "C:\\Tisty").unwrap(), "C:\\one");
    }

    #[cfg(windows)]
    #[test]
    fn windows_does_not_mind_the_case() {
        let path = joined(&["C:\\one", "c:\\programs\\TISTY"]);
        assert!(with(&path, "C:\\Programs\\Tisty").is_none());
        assert_eq!(without(&path, "C:\\Programs\\Tisty").unwrap(), "C:\\one");
    }

    /// Against a real registry, and never the person's `Environment`.
    #[cfg(windows)]
    #[test]
    fn the_registry_keeps_a_value_longer_than_nsis_could_read() {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};

        let at = r"Software\Tisty\PathRoundTrip";
        let (_key, _) = winreg::RegKey::predef(HKEY_CURRENT_USER)
            .create_subkey_with_flags(at, KEY_WRITE)
            .unwrap();

        let long: String = (0..30)
            .map(|i| format!(r"C:\Users\Someone\Program directory number {i}"))
            .collect::<Vec<_>>()
            .join(";");
        assert!(long.len() > 1024);

        write_to(at, &long).unwrap();
        assert_eq!(read_from(at).unwrap(), long, "the registry lost part of it");

        let _ = winreg::RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(at);
        let _ = winreg::RegKey::predef(HKEY_CURRENT_USER).delete_subkey(r"Software\Tisty");
    }

    #[test]
    fn the_rest_keeps_its_order_and_its_spelling() {
        let path = joined(&[
            "%USERPROFILE%\\bin",
            "C:\\Tisty",
            "C:\\Program Files\\Git\\cmd",
        ]);
        assert_eq!(
            without(&path, "C:\\Tisty").unwrap(),
            joined(&["%USERPROFILE%\\bin", "C:\\Program Files\\Git\\cmd"])
        );
    }
}
