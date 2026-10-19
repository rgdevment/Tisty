use super::*;

const DAY: u64 = 24 * 60 * 60;

#[test]
fn a_long_upload_is_found_again_until_it_closes_and_leaves_no_file_behind() {
    let home = tempfile::tempdir().unwrap();
    let uploads = Uploads::under(home.path(), DAY);

    uploads.opened("one", "https://upload/1");
    assert_eq!(
        uploads.session_for("one").as_deref(),
        Some("https://upload/1")
    );
    assert_eq!(
        Uploads::under(home.path(), DAY)
            .session_for("one")
            .as_deref(),
        Some("https://upload/1"),
        "a restart forgot where the upload had got to"
    );

    uploads.closed("one");
    assert_eq!(uploads.session_for("one"), None);
    assert!(!home.path().join(FILE).exists());
}

#[test]
fn a_session_older_than_the_provider_keeps_it_is_begun_again() {
    let home = tempfile::tempdir().unwrap();
    Uploads::under(home.path(), DAY).opened("one", "https://upload/1");

    assert_eq!(Uploads::under(home.path(), 0).session_for("one"), None);
}

#[test]
fn a_file_changed_since_is_another_upload() {
    let home = tempfile::tempdir().unwrap();
    let file = home.path().join("big.bin");
    std::fs::write(&file, b"one").unwrap();

    let before = key_of("https://to", "a/big.bin", &file, 3).unwrap();
    assert_eq!(key_of("https://to", "a/big.bin", &file, 3).unwrap(), before);
    assert_ne!(key_of("https://to", "a/big.bin", &file, 4).unwrap(), before);
    assert_ne!(key_of("https://to", "b/big.bin", &file, 3).unwrap(), before);
}

#[cfg(unix)]
#[test]
fn where_a_long_upload_got_to_is_readable_only_by_its_owner() {
    use std::os::unix::fs::PermissionsExt;
    let home = tempfile::tempdir().unwrap();

    Uploads::under(home.path(), DAY).opened("one", "https://upload/1");

    let mode = std::fs::metadata(home.path().join(FILE))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o777, 0o600);
}
