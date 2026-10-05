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
/// APFS flag for a file whose bytes live only in the cloud: iCloud Drive since Sonoma, and Dropbox
/// and OneDrive through File Provider, leave no sidecar, only this.
const SF_DATALESS: u32 = 0x4000_0000;

pub fn dataless(flags: u32) -> bool {
    flags & SF_DATALESS != 0
}

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
    let Ok(told) = std::fs::symlink_metadata(at) else {
        return false;
    };
    if told.file_type().is_symlink() {
        return std::fs::metadata(at).is_ok_and(|one| marked(one.file_attributes()));
    }
    marked(told.file_attributes())
}

#[cfg(target_os = "macos")]
fn held_away(at: &Path) -> bool {
    use std::os::macos::fs::MetadataExt;
    std::fs::metadata(at).is_ok_and(|one| dataless(one.st_flags()))
}

#[cfg(not(any(windows, target_os = "macos")))]
fn held_away(_at: &Path) -> bool {
    false
}

/// What in this directory is still in the cloud, by the path it will have once it is here.
pub fn still_away(dir: &Path) -> Vec<PathBuf> {
    let Ok(all) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    all.filter_map(|one| one.ok())
        .filter_map(|one| {
            let at = one.path();
            let name = at.file_name()?.to_str()?;
            if marker(name) {
                let real = &name[1..name.len() - ".icloud".len()];
                return Some(at.with_file_name(real));
            }
            (at.is_file() && held_away(&at)).then_some(at)
        })
        .collect()
}

/// Reading a file is what makes the cloud bring it down, and that read waits for the download:
/// done on a thread of its own, the round that found it goes on and the next one finds it here.
pub fn ask_for(all: Vec<PathBuf>) {
    if all.is_empty() {
        return;
    }
    std::thread::spawn(move || {
        for at in all {
            if sidecar(&at).is_some() {
                fetched(&at);
            } else if let Ok(mut file) = std::fs::File::open(&at) {
                let mut one = [0u8; 1];
                let _ = std::io::Read::read(&mut file, &mut one);
            }
        }
    });
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
