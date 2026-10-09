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
