use std::path::{Path, PathBuf};

use tisty_core::witness::{self, Fact, channel};

fn unwritten(why: &std::io::Error) {
    witness::error(
        channel::TERMINAL,
        "the tisty command could not be taken off PATH",
        &[("why", Fact::Why(why.to_string()))],
    );
}

pub const CLI: &str = if cfg!(windows) { "tisty.exe" } else { "tisty" };

pub fn beside() -> Option<PathBuf> {
    with_cli(std::env::current_exe().ok()?.as_path())
}

fn with_cli(here: &Path) -> Option<PathBuf> {
    let folder = here.parent()?;
    let cli = folder.join(CLI);
    (cli.is_file() && !one_file(&cli, here)).then(|| folder.to_path_buf())
}

/// Packaged for the Store the window is `Tisty.exe`, and Windows hands that back for `tisty.exe`:
/// the CLI is off in `cli\`, so a window that answers to its own name has none beside it.
fn one_file(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// What another program has to run to reach the MCP server: the copy beside the window, or the
/// name alone, which is what the Store's execution alias answers to and survives its updates.
pub fn calling() -> String {
    called(beside().as_deref())
}

fn called(folder: Option<&Path>) -> String {
    match folder {
        Some(at) => settled(&at.join(CLI)).display().to_string(),
        None => aliased().map_or_else(|| "tisty".to_string(), |at| at.display().to_string()),
    }
}

// The alias's own path does not move with the Store's updates and needs no PATH to be found.
fn aliased() -> Option<PathBuf> {
    let at = aliases()?.join(CLI);
    (cfg!(windows) && at.exists()).then_some(at)
}

pub fn aliases() -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    Some(Path::new(&local).join("Microsoft").join("WindowsApps"))
}

// Opened from the disk image or moved aside by macOS, the path is gone once the app is quit.
fn settled(at: &Path) -> PathBuf {
    let shown = at.display().to_string();
    let passing = shown.contains("/AppTranslocation/") || shown.starts_with("/Volumes/");
    let installed = Path::new("/Applications/Tisty.app/Contents/MacOS").join(CLI);
    match passing && installed.is_file() {
        true => installed,
        false => at.to_path_buf(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reach {
    pub binary: Option<String>,
}

pub fn reach() -> Reach {
    Reach {
        binary: beside().map(|at| at.join(CLI).display().to_string()),
    }
}

#[cfg(windows)]
const WHERE: &str = "Environment";

#[cfg(windows)]
pub(crate) fn user_path() -> Option<String> {
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
pub(crate) fn set_user_path(value: &str) -> std::io::Result<()> {
    write_to(WHERE, value)
}

#[cfg(windows)]
fn write_to(at: &str, value: &str) -> std::io::Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE, REG_EXPAND_SZ};
    let key = winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(at, KEY_WRITE)?;
    let held = winreg::RegValue {
        bytes: value
            .encode_utf16()
            .chain(std::iter::once(0))
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<u8>>()
            .into(),
        vtype: REG_EXPAND_SZ,
    };
    key.set_raw_value("Path", &held)?;
    Ok(())
}

#[cfg(not(windows))]
fn shelf() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("bin"))
}

#[cfg(not(windows))]
fn link() -> Option<PathBuf> {
    Some(shelf()?.join("tisty"))
}

#[cfg(not(windows))]
fn untie() -> std::io::Result<bool> {
    match link() {
        Some(link) => untie_at(&link),
        None => Ok(false),
    }
}

/// Only a link of ours is taken out — one pointing at this window's command, or at an
/// install that moved — and somebody else's `tisty` on the shelf is left where it is.
#[cfg(not(windows))]
fn untie_at(link: &Path) -> std::io::Result<bool> {
    if !is_our_own_link(link) {
        return Ok(false);
    }
    std::fs::remove_file(link)?;
    Ok(true)
}

#[cfg(not(windows))]
fn is_our_own_link(at: &Path) -> bool {
    at.symlink_metadata().is_ok_and(|it| it.is_symlink())
        && std::fs::read_link(at)
            .is_ok_and(|to| to.file_name().is_some_and(|n| n == "tisty") && into_a_bundle(&to))
}

// Only what points into an app bundle was ours: a formula's or a build of somebody's own stays.
#[cfg(not(windows))]
fn into_a_bundle(to: &Path) -> bool {
    to.components()
        .any(|one| one.as_os_str().to_string_lossy().ends_with(".app"))
}

// Only what calls this install's own copy goes: a wrapper of the person's for anything else stays.
pub fn ours_on_the_shelf(at: &Path, home: &Path) -> bool {
    let named = |text: &str| {
        text.to_lowercase()
            .contains(&home.display().to_string().to_lowercase())
    };
    match at.symlink_metadata() {
        Ok(it) if it.is_symlink() => {
            std::fs::read_link(at).is_ok_and(|to| named(&to.display().to_string()))
        }
        Ok(it) => {
            it.len() < 4096
                && std::fs::read(at).is_ok_and(|bytes| named(&String::from_utf8_lossy(&bytes)))
        }
        Err(_) => false,
    }
}

#[cfg(windows)]
pub fn out_of_reach() -> std::io::Result<bool> {
    crate::pathing::leaving();
    let Some(folder) = beside() else {
        return Ok(false);
    };
    let path = user_path().unwrap_or_default();
    match crate::pathing::without(
        &path,
        &folder.display().to_string(),
        &crate::pathing::expanded,
    ) {
        Some(next) => {
            set_user_path(&next).inspect_err(unwritten)?;
            crate::pathing::announce();
            Ok(true)
        }
        None => Ok(false),
    }
}

#[cfg(not(windows))]
pub fn out_of_reach() -> std::io::Result<bool> {
    untie().inspect_err(unwritten)
}

#[cfg(test)]
#[path = "command_finding.rs"]
mod finding;

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;

    struct Laid {
        _tmp: tempfile::TempDir,
        shelf: PathBuf,
        link: PathBuf,
        folder: PathBuf,
    }

    fn laid() -> Laid {
        let tmp = tempfile::tempdir().unwrap();
        let shelf = tmp.path().join("bin");
        let folder = tmp.path().join("Tisty.app/Contents/MacOS");
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("tisty"), b"#!/bin/sh\n").unwrap();
        Laid {
            link: shelf.join("tisty"),
            shelf,
            folder,
            _tmp: tmp,
        }
    }

    fn linked(it: &Laid) {
        std::fs::create_dir_all(&it.shelf).unwrap();
        std::os::unix::fs::symlink(it.folder.join("tisty"), &it.link).unwrap();
    }

    #[test]
    fn the_link_is_taken_back_out() {
        let it = laid();
        linked(&it);

        assert!(untie_at(&it.link).unwrap());
        assert!(it.link.symlink_metadata().is_err());
    }

    #[test]
    fn taking_out_what_was_never_there_changes_nothing() {
        let it = laid();

        assert!(!untie_at(&it.link).unwrap());
    }

    #[test]
    fn somebody_elses_command_is_left_alone_when_the_link_is_taken_out() {
        let it = laid();
        std::fs::create_dir_all(&it.shelf).unwrap();
        std::fs::write(&it.link, b"somebody else's command").unwrap();

        assert!(!untie_at(&it.link).unwrap());
        assert!(it.link.is_file());
    }

    #[test]
    fn a_link_to_a_command_installed_some_other_way_is_left_alone() {
        let it = laid();
        std::fs::create_dir_all(&it.shelf).unwrap();
        std::os::unix::fs::symlink("/opt/homebrew/bin/tisty", &it.link).unwrap();

        assert!(!untie_at(&it.link).unwrap());
        assert!(it.link.symlink_metadata().is_ok());
    }

    #[test]
    fn a_link_of_ours_left_pointing_at_an_old_install_is_still_ours_to_take_out() {
        let it = laid();
        let gone = it.folder.parent().unwrap().join("Old/tisty");
        std::fs::create_dir_all(&it.shelf).unwrap();
        std::os::unix::fs::symlink(&gone, &it.link).unwrap();

        assert!(untie_at(&it.link).unwrap());
        assert!(it.link.symlink_metadata().is_err());
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

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
}
