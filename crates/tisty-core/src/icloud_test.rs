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
