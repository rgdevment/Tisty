use super::*;

#[test]
fn a_file_kept_here_says_when_it_changed_and_one_that_is_not_says_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    std::fs::create_dir_all(data.join("attachments")).unwrap();
    std::fs::write(data.join("attachments/map-1a2b3c4d.png"), b"a map").unwrap();

    let changed = changed_at("attachments/map-1a2b3c4d.png", &data, None).unwrap();

    assert!(changed.parse::<jiff::Timestamp>().is_ok(), "{changed}");
    assert_eq!(
        changed_at("attachments/gone-1a2b3c4d.png", &data, None),
        None
    );
}
