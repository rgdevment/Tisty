use super::*;

#[test]
fn only_the_block_outside_the_language_is_named_with_its_mend() {
    let body = "# Plan\n\nWords as they are.\n\nThe thread said <b>loud</b>.\n";
    let found = mends(body);

    assert_eq!(found.len(), 1, "{found:?}");
    let one = &found[0];
    assert_eq!(&body[one.from..one.to], "The thread said <b>loud</b>.");
    assert_eq!(one.fixed, "The thread said **loud**.");
}

#[test]
fn code_properties_references_and_windows_lines_are_never_mends() {
    let body = "---\r\ntags: a\r\n---\r\n\r\nSee [[Rina]] and [[#2]].\r\n\r\n```widget\r\n<div class=\"grid\"></div>\r\n```\r\n";
    assert_eq!(mends(body), Vec::new());
}

#[test]
fn a_mend_keeps_the_lines_the_document_is_written_in() {
    let body = "One <u>line</u>\r\nand another\r\n";
    let found = mends(body);

    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].fixed, "One ++line++\r\nand another");
}

#[test]
fn each_piece_outside_the_language_is_named_with_how_it_is_written() {
    let body = "Said <b>two words</b> and then <b>one</b>, then nothing.\n";
    let found = mends(body);

    assert_eq!(found.len(), 1, "{found:?}");
    let told: Vec<(&str, &str)> = found[0]
        .pieces
        .iter()
        .map(|one| (one.said.as_str(), one.fixed.as_str()))
        .collect();
    assert_eq!(
        told,
        [
            ("<b>two words</b>", "**two words**"),
            ("<b>one</b>,", "**one**,")
        ]
    );
}

#[test]
fn a_document_already_in_the_language_is_told_nothing_without_reading_its_blocks() {
    assert_eq!(
        mends("# Plan\n\nWords as they are, ==a pen==.\n"),
        Vec::new()
    );
}
