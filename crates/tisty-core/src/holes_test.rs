use super::*;

const EXTENDED_ATTRIBUTES: u32 = 0x0004_0000;
const ARCHIVE: u32 = 0x0000_0020;

#[test]
fn the_sidecar_icloud_leaves_is_found_by_the_name_that_went() {
    let room = tempfile::tempdir().unwrap();
    let gone = room.path().join("charla-90706bde.mp4");
    std::fs::write(room.path().join(".charla-90706bde.mp4.icloud"), b"a few").unwrap();

    assert_eq!(
        left_in_place(&gone),
        Left::Sidecar(room.path().join(".charla-90706bde.mp4.icloud"))
    );
    assert!(a_hole(&gone));
}

#[test]
fn a_file_that_is_all_there_is_not_a_hole() {
    let room = tempfile::tempdir().unwrap();
    let whole = room.path().join("charla-90706bde.mp4");
    std::fs::write(&whole, b"the whole of it").unwrap();

    assert_eq!(left_in_place(&whole), Left::Nothing);
    assert!(!a_hole(&whole));
}

#[test]
fn a_name_that_is_missing_altogether_is_not_a_hole() {
    let room = tempfile::tempdir().unwrap();

    assert_eq!(
        left_in_place(&room.path().join("never-00000000.mp4")),
        Left::Nothing
    );
}

#[test]
fn a_sidecar_is_told_from_a_name_that_merely_says_icloud() {
    assert!(marker(".charla-90706bde.mp4.icloud"));
    assert!(!marker("charla-90706bde.mp4"));
    assert!(!marker("notes.icloud.txt"));
}

#[test]
fn each_bit_a_keeper_sets_reads_as_a_hole() {
    assert!(marked(OFFLINE));
    assert!(marked(RECALL_ON_DATA_ACCESS));
    assert!(marked(REPARSE_POINT | RECALL_ON_OPEN));
    assert!(marked(ARCHIVE | RECALL_ON_DATA_ACCESS));
}

#[test]
fn a_file_carrying_extended_attributes_is_not_accused_of_being_a_hole() {
    assert!(!marked(ARCHIVE | EXTENDED_ATTRIBUTES));
    assert!(!marked(ARCHIVE));
    assert!(!marked(REPARSE_POINT));
}

#[test]
fn nothing_is_asked_of_a_marked_entry_because_reading_it_is_the_asking() {
    assert!(comes_by_reading(&Left::Marked));
    assert!(!can_ask(&Left::Marked));

    assert!(!comes_by_reading(&Left::Nothing));
    assert!(!can_ask(&Left::Nothing));
}

#[test]
fn a_sidecar_is_only_worth_asking_about_where_somebody_answers() {
    let asked = can_ask(&Left::Sidecar("anywhere".into()));

    assert_eq!(asked, cfg!(target_os = "macos"));
    assert!(!comes_by_reading(&Left::Sidecar("anywhere".into())));
}

#[cfg(windows)]
#[test]
fn a_file_windows_itself_marks_as_held_away_is_read_as_a_hole() {
    use std::os::windows::fs::OpenOptionsExt;

    let room = tempfile::tempdir().unwrap();
    let away = room.path().join("charla-90706bde.mp4");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .attributes(OFFLINE)
        .open(&away)
        .unwrap();

    assert!(
        away.is_file(),
        "windows keeps the entry, which is the whole trap"
    );
    assert_eq!(left_in_place(&away), Left::Marked);
    assert!(a_hole(&away));
}

#[cfg(windows)]
#[test]
fn a_link_standing_where_the_attachment_goes_does_not_hide_the_hole_behind_it() {
    use std::os::windows::fs::OpenOptionsExt;

    let room = tempfile::tempdir().unwrap();
    let away = room.path().join("charla-90706bde.mp4");
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .attributes(OFFLINE)
        .open(&away)
        .unwrap();
    let named = room.path().join("linked-90706bde.mp4");
    if std::os::windows::fs::symlink_file(&away, &named).is_err() {
        return;
    }

    assert!(named.is_file(), "a link resolves like any other file");
    assert_eq!(left_in_place(&named), Left::Marked);
    assert!(a_hole(&named));
}

#[test]
fn a_file_whose_bytes_live_only_in_the_cloud_is_told_by_its_flag_alone() {
    assert!(dataless(0x4000_0000));
    assert!(dataless(0x4000_0000 | 0x20));
    assert!(!dataless(0x20));
}

#[test]
fn what_is_still_away_is_named_by_the_path_it_will_have() {
    let room = tempfile::tempdir().unwrap();
    std::fs::write(room.path().join("000001.tisty"), b"here").unwrap();
    std::fs::write(room.path().join(".000002.tisty.icloud"), b"stub").unwrap();

    assert_eq!(
        still_away(room.path()),
        vec![room.path().join("000002.tisty")]
    );
}

#[test]
fn a_file_named_only_icloud_names_nothing_and_breaks_nothing() {
    let room = tempfile::tempdir().unwrap();
    std::fs::write(room.path().join(".icloud"), b"planted").unwrap();

    assert_eq!(named_away(".icloud"), None);
    assert!(still_away(room.path()).is_empty());
}
