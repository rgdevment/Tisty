use super::*;

fn paths_in(tmp: &tempfile::TempDir) -> tisty_core::Paths {
    tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"))
}

fn put_package(paths: &tisty_core::Paths) {
    std::fs::create_dir_all(kept(paths)).unwrap();
    std::fs::write(kept(paths).join(PACKAGE), b"bytes").unwrap();
}

fn put_said(paths: &tisty_core::Paths, version: &str) {
    std::fs::create_dir_all(kept(paths)).unwrap();
    let said = Said {
        version: version.to_string(),
        raw: serde_json::json!({ "version": version }),
    };
    std::fs::write(kept(paths).join(SAID), serde_json::to_vec(&said).unwrap()).unwrap();
}

#[test]
fn nothing_waits_when_no_file_was_kept() {
    let tmp = tempfile::tempdir().unwrap();

    assert!(!waits_for(&paths_in(&tmp), "0.3.0"));
}

#[test]
fn nothing_waits_with_only_the_note_of_what_was_read() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths_in(&tmp);
    put_said(&paths, "0.3.0");

    assert!(!waits_for(&paths, "0.3.0"));
}

#[test]
fn nothing_waits_with_only_the_package() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths_in(&tmp);
    put_package(&paths);

    assert!(!waits_for(&paths, "0.3.0"));
}

#[test]
fn nothing_waits_when_the_note_names_another_version() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths_in(&tmp);
    put_package(&paths);
    put_said(&paths, "0.2.5");

    assert!(!waits_for(&paths, "0.3.0"));
}

#[test]
fn nothing_waits_when_the_note_cannot_be_read() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths_in(&tmp);
    put_package(&paths);
    std::fs::write(kept(&paths).join(SAID), b"not json").unwrap();

    assert!(!waits_for(&paths, "0.3.0"));
}

#[test]
fn the_package_and_the_note_of_the_same_version_wait() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths_in(&tmp);
    put_package(&paths);
    put_said(&paths, "0.3.0");

    assert!(waits_for(&paths, "0.3.0"));
}

#[test]
fn forgetting_removes_what_was_kept() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = paths_in(&tmp);
    put_package(&paths);
    put_said(&paths, "0.3.0");
    std::fs::write(kept(&paths).join(PART), b"half").unwrap();

    forget(&paths);

    assert!(!kept(&paths).exists());
    assert!(!waits_for(&paths, "0.3.0"));
}

#[test]
fn forgetting_with_nothing_kept_does_nothing() {
    let tmp = tempfile::tempdir().unwrap();

    forget(&paths_in(&tmp));
}
