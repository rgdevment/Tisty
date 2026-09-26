use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// iCloud swaps an evicted file for `.name.ext.icloud`, and the name it had stops existing.
pub fn shed(at: &Path) -> Option<PathBuf> {
    let name = at.file_name()?.to_str()?;
    let marker = at.with_file_name(format!(".{name}.icloud"));
    marker.is_file().then_some(marker)
}

pub fn marker(name: &str) -> bool {
    name.starts_with('.') && name.ends_with(".icloud")
}

#[cfg(target_os = "macos")]
pub fn asked_back(at: &Path) -> bool {
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
pub fn asked_back(_at: &Path) -> bool {
    false
}

/// Whether this machine can even ask: off macOS there is nobody to ask, and saying «it is coming»
/// would be a promise nothing here can keep.
pub fn can_ask() -> bool {
    cfg!(target_os = "macos")
}

/// Only for as long as somebody would wait looking at a window.
pub fn waited_for(at: &Path, most: Duration) -> bool {
    if !asked_back(at) {
        return false;
    }
    let since = Instant::now();
    while since.elapsed() < most {
        if at.is_file() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(120));
    }
    at.is_file()
}

#[cfg(test)]
#[path = "icloud_test.rs"]
mod tests;
