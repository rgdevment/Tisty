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
