use super::{PIECE_ABOUT, body_of, drawn_from};

fn file_of_size(room: &tempfile::TempDir, bytes: usize) -> std::path::PathBuf {
    let at = room.path().join("long.md");
    std::fs::write(&at, "x".repeat(bytes)).unwrap();
    at
}

#[test]
fn reading_takes_a_file_a_document_could_not_hold() {
    let room = tempfile::tempdir().unwrap();
    let at = file_of_size(&room, tisty_core::docs::BODY_AT_MOST as usize + 1);
    assert!(tisty_core::docs::read_outside(&at).is_err());
    assert!(body_of(&at).is_ok());
}

#[test]
fn reading_refuses_past_its_own_ceiling_and_names_it() {
    let room = tempfile::tempdir().unwrap();
    let at = file_of_size(&room, tisty_core::docs::READ_AT_MOST as usize + 1);
    let refused = body_of(&at).unwrap_err();
    assert_eq!(refused.code, "documentTooBig");
}

#[test]
fn reading_does_not_change_a_byte_of_the_file() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("kept.md");
    let written = "# Title\r\n\r\ncut at\r\neighty columns #ff0000 [[x]]\r\n";
    std::fs::write(&at, written).unwrap();
    let before = std::fs::metadata(&at).unwrap().modified().unwrap();
    let body = body_of(&at).unwrap();
    drawn_from(&body);
    assert_eq!(std::fs::read_to_string(&at).unwrap(), written);
    assert_eq!(std::fs::metadata(&at).unwrap().modified().unwrap(), before);
}

#[test]
fn the_pieces_add_up_to_what_one_drawing_says() {
    let body: String = (0..6000)
        .map(|at| format!("## Heading {at}\n\nA paragraph that takes its room, number {at}.\n\n"))
        .collect();
    let (pieces, heads) = drawn_from(&body);
    assert!(pieces.len() > 1);
    assert_eq!(heads.len(), 6000);
    assert_eq!(
        pieces.concat(),
        tisty_markdown::html(&tisty_markdown::read(&body))
    );
    let last = pieces.len() - 1;
    assert!(pieces[..last].iter().all(|one| one.len() >= PIECE_ABOUT));
}

#[test]
fn the_index_names_what_the_page_anchors() {
    let (pieces, heads) = drawn_from("# One\n\n## Two\n");
    assert_eq!(heads.len(), 2);
    assert_eq!((heads[1].level, heads[1].said.as_str()), (2, "Two"));
    assert!(pieces[0].contains(&format!("id=\"{}\"", heads[1].id)));
}

#[test]
fn an_empty_file_is_no_pieces_at_all() {
    let (pieces, heads) = drawn_from("");
    assert!(pieces.is_empty());
    assert!(heads.is_empty());
}
