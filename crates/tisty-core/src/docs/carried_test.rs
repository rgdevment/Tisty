use super::*;

#[test]
fn a_reader_that_never_ends_is_refused_at_the_ceiling() {
    assert!(read_within(std::io::repeat(b'x'), 10).is_err());
}

#[test]
fn a_reader_right_up_to_the_ceiling_is_read_whole() {
    assert_eq!(
        read_within(&b"0123456789"[..], 10).unwrap(),
        b"0123456789".to_vec()
    );
}

#[test]
fn a_reader_one_byte_past_the_ceiling_is_refused() {
    assert!(read_within(&b"0123456789x"[..], 10).is_err());
}

#[test]
fn a_body_past_the_ceiling_is_refused_without_being_opened_to_be_read() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("huge.md");
    std::fs::write(&at, vec![b'x'; (BODY_AT_MOST + 1) as usize]).unwrap();
    crate::counting::from_now();

    assert!(whole_of(&at).is_err());

    assert_eq!(
        crate::counting::from_now(),
        0,
        "a body past the ceiling was read to find that out"
    );
}

#[test]
fn the_same_folder_come_back_with_another_mark_keeps_what_was_carried() {
    let shared = tempfile::tempdir().unwrap();
    let mut said = Carried {
        prints: [("dev_a-0001".to_string(), "huella".to_string())].into(),
        up_to: Some(format!("1:2\n{}", shared.path().display())),
    };

    assert!(!said.facing(shared.path(), false, true));

    assert_eq!(said.of("dev_a-0001"), Some("huella"));
    assert_eq!(
        said.up_to.as_deref(),
        Some(crate::paths::told_of(shared.path()).as_str())
    );
}

#[test]
fn the_same_path_holding_another_store_forgets_what_was_carried() {
    let shared = tempfile::tempdir().unwrap();
    let mut said = Carried {
        prints: [("dev_a-0001".to_string(), "huella".to_string())].into(),
        up_to: Some(format!("1:2\n{}", shared.path().display())),
    };

    assert!(said.facing(shared.path(), false, false));

    assert_eq!(said.of("dev_a-0001"), None);
}

#[test]
fn another_folder_forgets_what_was_carried() {
    let before = tempfile::tempdir().unwrap();
    let now = tempfile::tempdir().unwrap();
    let mut said = Carried {
        prints: [("dev_a-0001".to_string(), "huella".to_string())].into(),
        up_to: Some(crate::paths::told_of(before.path())),
    };

    assert!(said.facing(now.path(), false, true));

    assert_eq!(said.of("dev_a-0001"), None);
}
