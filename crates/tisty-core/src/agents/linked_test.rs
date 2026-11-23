use std::path::Path;

pub(crate) fn linked_folder(from: &Path, to: &Path) {
    #[cfg(windows)]
    {
        let made = std::process::Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(from)
            .arg(to)
            .output()
            .unwrap();
        assert!(made.status.success(), "{made:?}");
    }
    #[cfg(not(windows))]
    std::os::unix::fs::symlink(to, from).unwrap();
}
