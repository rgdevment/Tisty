use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Left {
    Nothing,
    Sidecar(PathBuf),
    Marked,
}

const OFFLINE: u32 = 0x0000_1000;
const REPARSE_POINT: u32 = 0x0000_0400;
const RECALL_ON_OPEN: u32 = 0x0004_0000;
const RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;

pub fn marked(attributes: u32) -> bool {
    attributes & (OFFLINE | RECALL_ON_DATA_ACCESS) != 0
        || attributes & (REPARSE_POINT | RECALL_ON_OPEN) == REPARSE_POINT | RECALL_ON_OPEN
}

pub fn left_in_place(at: &Path) -> Left {
    if let Some(beside) = sidecar(at) {
        return Left::Sidecar(beside);
    }
    match held_away(at) {
        true => Left::Marked,
        false => Left::Nothing,
    }
}

pub fn a_hole(at: &Path) -> bool {
    left_in_place(at) != Left::Nothing
}

fn sidecar(at: &Path) -> Option<PathBuf> {
    let name = at.file_name()?.to_str()?;
    let beside = at.with_file_name(format!(".{name}.icloud"));
    beside.is_file().then_some(beside)
}

pub fn marker(name: &str) -> bool {
    name.starts_with('.') && name.ends_with(".icloud")
}

#[cfg(windows)]
fn held_away(at: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    std::fs::symlink_metadata(at).is_ok_and(|told| marked(told.file_attributes()))
}

#[cfg(not(windows))]
fn held_away(_at: &Path) -> bool {
    false
}

pub fn can_ask(left: &Left) -> bool {
    match left {
        Left::Sidecar(_) => cfg!(target_os = "macos"),
        Left::Marked | Left::Nothing => false,
    }
}

pub fn comes_by_reading(left: &Left) -> bool {
    matches!(left, Left::Marked)
}

#[cfg(target_os = "macos")]
fn fetched(at: &Path) -> bool {
    std::process::Command::new("/usr/bin/brctl")
        .arg("download")
        .arg(at)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|done| done.success())
}

#[cfg(not(target_os = "macos"))]
fn fetched(_at: &Path) -> bool {
    false
}

pub fn waited_for(at: &Path, most: Duration) -> bool {
    if !fetched(at) {
        return false;
    }
    let since = std::time::Instant::now();
    while since.elapsed() < most {
        if at.is_file() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(120));
    }
    at.is_file()
}

#[cfg(test)]
#[path = "holes_test.rs"]
mod tests;
