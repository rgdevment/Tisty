use super::relabeled;

#[test]
fn a_renamed_page_takes_its_new_name_where_its_document_lists_it() {
    let whole =
        "# Plan\n\nintro\n\n![Old name](tisty:doc/aaaa-0001)\n\n![Notes](tisty:doc/aaaa-0002)\n";

    let now = relabeled(whole, "aaaa-0001", "New name").unwrap();

    assert_eq!(
        now,
        "# Plan\n\nintro\n\n![New name](tisty:doc/aaaa-0001)\n\n![Notes](tisty:doc/aaaa-0002)\n"
    );
}

#[test]
fn a_line_that_already_says_the_name_is_left_as_it_is() {
    let whole = "# Plan\n\n![Plan *v2*](tisty:doc/aaaa-0001)\n";

    assert_eq!(relabeled(whole, "aaaa-0001", "Plan v2"), None);
    assert_eq!(
        relabeled(whole, "aaaa-0009", "Other"),
        None,
        "no card of that page"
    );
}

#[test]
fn a_mention_in_a_sentence_is_not_a_card_and_stays() {
    let whole = "# Plan\n\nver [Old name](tisty:doc/aaaa-0001) arriba\n\n![Old name](tisty:doc/aaaa-0001)\n";

    let now = relabeled(whole, "aaaa-0001", "New name").unwrap();

    assert!(
        now.contains("ver [Old name](tisty:doc/aaaa-0001) arriba"),
        "{now}"
    );
    assert!(now.contains("![New name](tisty:doc/aaaa-0001)"), "{now}");
}

#[test]
fn a_line_holding_more_than_the_card_keeps_all_of_it() {
    let whole = "# Plan\n\n![la foto](foto.png) ![Old name](tisty:doc/aaaa-0001)\n";

    assert_eq!(relabeled(whole, "aaaa-0001", "New name"), None);
}

#[test]
fn a_card_shown_as_an_example_in_code_is_left_as_written() {
    let whole = "# Plan\n\n![Old name](tisty:doc/aaaa-0001)\n\n```md\n![Old name](tisty:doc/aaaa-0001)\n```\n";

    let now = relabeled(whole, "aaaa-0001", "New name").unwrap();

    assert!(
        now.contains("```md\n![Old name](tisty:doc/aaaa-0001)\n```"),
        "{now}"
    );
    assert!(
        now.starts_with("# Plan\n\n![New name](tisty:doc/aaaa-0001)"),
        "{now}"
    );
}

#[test]
fn every_card_of_the_page_is_looked_at_not_only_the_first() {
    let whole = "# Plan\n\n![New name](tisty:doc/aaaa-0001)\n\n![Old name](tisty:doc/aaaa-0001)\n";

    let now = relabeled(whole, "aaaa-0001", "New name").unwrap();

    assert!(!now.contains("Old name"), "{now}");
}

#[test]
fn a_book_whose_title_is_the_card_itself_is_not_renamed_by_its_page() {
    let whole = "![Old name](tisty:doc/aaaa-0001)\n\nmore\n";

    assert_eq!(relabeled(whole, "aaaa-0001", "New name"), None);
}
