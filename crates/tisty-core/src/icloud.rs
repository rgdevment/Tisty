use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// iCloud takes a file away and leaves `.name.ext.icloud` in its place, a few bytes long: the name
/// it had stops existing, so nothing that opens it finds anything until it is asked back.
pub fn shed(at: &Path) -> Option<PathBuf> {
    let name = at.file_name()?.to_str()?;
    let marker = at.with_file_name(format!(".{name}.icloud"));
    marker.is_file().then_some(marker)
}

/// Whether a name in a folder is one of those markers rather than something somebody put there.
pub fn marker(name: &str) -> bool {
    name.starts_with('.') && name.ends_with(".icloud")
}

#[cfg(target_os = "macos")]
pub fn asked_back(at: &Path) -> bool {
    std::process::Command::new("brctl")
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

/// Asks for it and waits, but only for as long as somebody would wait looking at a window.
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
mod tests {
    use super::*;

    #[test]
    fn a_file_that_is_there_was_never_shed() {
        let room = tempfile::tempdir().unwrap();
        let at = room.path().join("charla-90706bde.mp4");
        std::fs::write(&at, b"the whole of it").unwrap();

        assert!(shed(&at).is_none());
    }

    #[test]
    fn the_marker_icloud_leaves_is_found_by_the_name_that_went() {
        let room = tempfile::tempdir().unwrap();
        let at = room.path().join("charla-90706bde.mp4");
        std::fs::write(room.path().join(".charla-90706bde.mp4.icloud"), b"a few").unwrap();

        assert_eq!(
            shed(&at),
            Some(room.path().join(".charla-90706bde.mp4.icloud"))
        );
    }

    #[test]
    fn nothing_is_read_into_a_folder_that_holds_neither() {
        let room = tempfile::tempdir().unwrap();

        assert!(shed(&room.path().join("charla-90706bde.mp4")).is_none());
    }

    #[test]
    fn a_marker_is_told_from_an_attachment_by_its_name() {
        assert!(marker(".charla-90706bde.mp4.icloud"));
        assert!(!marker("charla-90706bde.mp4"));
        assert!(!marker(".hidden"));
        assert!(!marker("notes.icloud.txt"));
    }
}
