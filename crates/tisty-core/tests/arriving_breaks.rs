use tisty_core::arriving::{
    arrived, arrived_among, from_html, from_pasted_html, from_text, tidied,
};

const NOTE: &str = "line breaks written as <br>, in Tisty's own words now";

fn landed(html: &str) -> String {
    arrived(&from_pasted_html(html)).body
}

#[test]
fn a_line_of_only_br_becomes_one_blank_paragraph_for_each_break() {
    assert_eq!(arrived("a\n\n<br>\n\nb").body, "a\n\n&nbsp;\n\nb");
    assert_eq!(arrived("a\n\n<br />\n\nb").body, "a\n\n&nbsp;\n\nb");
    assert_eq!(arrived("a\n\n<BR>\n\nb").body, "a\n\n&nbsp;\n\nb");
    assert_eq!(
        arrived("a\n\n<br>\n<br>\n\nb").body,
        "a\n\n&nbsp;\n\n&nbsp;\n\nb"
    );
    let made = arrived("a\n\n<br>\n\nb");
    assert!(made.changed.contains(&NOTE), "{:?}", made.changed);
}

#[test]
fn a_br_among_words_becomes_tistys_line_break() {
    assert_eq!(arrived("uno<br>dos").body, "uno\\\ndos");
    assert_eq!(arrived("uno<br>\ndos").body, "uno\\\ndos");
    assert_eq!(arrived("- a<br>b").body, "- a\\\nb");
}

#[test]
fn a_br_ending_its_paragraph_is_dropped() {
    assert_eq!(arrived("uno<br>").body, "uno");
    assert_eq!(arrived("a<br>\n\nb").body, "a\n\nb");
}

#[test]
fn code_stays_as_written_and_a_body_with_no_br_gets_no_note() {
    let code = "```\nx<br>y\n```";
    assert_eq!(arrived(code).body, code);
    assert!(!arrived("uno\n\ndos").changed.contains(&NOTE));
}

#[test]
fn a_document_already_here_is_mended_without_turning_its_br() {
    let made = tidied("a\n\n<br>\n\nb");
    assert!(!made.body.contains("&nbsp;"), "{:?}", made.body);
    assert!(!made.changed.contains(&NOTE), "{:?}", made.changed);
}

#[test]
fn a_pasted_line_of_only_no_break_spaces_arrives_as_a_blank_paragraph() {
    assert_eq!(
        landed("<p>uno</p><p>\u{a0}</p><p>dos</p>"),
        "uno\n\n&nbsp;\n\ndos"
    );
    assert_eq!(landed("<p>&nbsp;</p>"), "&nbsp;");
}

#[test]
fn the_import_of_a_markdown_file_turns_its_br_the_same_way() {
    let made = arrived_among("uno<br>dos\n\n<br>\n\nfin", &Default::default());
    assert_eq!(made.body, "uno\\\ndos\n\n&nbsp;\n\nfin");
}

#[test]
fn several_br_on_one_line_are_as_many_blank_paragraphs() {
    assert_eq!(
        arrived("a\n\n<br><br>\n\nb").body,
        "a\n\n&nbsp;\n\n&nbsp;\n\nb"
    );
    assert_eq!(
        arrived("a\n\n<br><br><br>\n\nb").body,
        "a\n\n&nbsp;\n\n&nbsp;\n\n&nbsp;\n\nb"
    );
}

#[test]
fn plain_text_pasted_from_another_editor_turns_its_br_and_its_nbsp() {
    assert_eq!(
        from_text("a\n\n<br>\n\nb").as_deref(),
        Some("a\n\n&nbsp;\n\nb")
    );
    assert_eq!(
        from_text("uno &nbsp;&nbsp;dos").as_deref(),
        Some("uno   dos")
    );
    assert_eq!(from_text("uno<br>dos").as_deref(), Some("uno\\\ndos"));
}

#[test]
fn plain_text_with_nothing_to_turn_is_left_to_the_clipboard() {
    assert_eq!(from_text("uno dos"), None);
}

#[test]
fn a_br_inside_a_table_cell_becomes_a_space_in_paste_and_import() {
    let html = "<table><tr><th>h</th></tr><tr><td>a<br>b</td></tr></table>";
    assert!(arrived(&from_pasted_html(html)).body.contains("| a b |"));
    let said = arrived_among(&from_html(html), &Default::default()).body;
    assert!(said.contains("| a b |"), "{said:?}");
}

#[test]
fn an_imported_html_page_evens_its_no_break_spaces_like_a_paste() {
    let said = |html: &str| arrived_among(&from_html(html), &Default::default()).body;
    assert_eq!(said("<p>&nbsp;</p>"), "&nbsp;");
    assert_eq!(said("<p>uno&nbsp;&nbsp;dos</p>"), "uno  dos");
}
