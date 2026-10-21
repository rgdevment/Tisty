use super::{TOPICS, sections};

#[test]
fn every_topic_names_a_section_in_both_languages() {
    for spanish in [false, true] {
        let found = sections(tisty_markdown::reference(spanish));
        assert_eq!(
            found.len(),
            TOPICS.len(),
            "a section was added or taken out of the reference; name it in TOPICS: {:?}",
            found.iter().map(|(title, _)| title).collect::<Vec<_>>()
        );
    }
}

#[test]
fn a_topic_brings_its_own_section_and_nothing_else() {
    let found = sections(tisty_markdown::reference(false));
    let pens = TOPICS.iter().position(|one| *one == "pens").unwrap();

    assert!(found[pens].1.starts_with("## Pens"), "{}", found[pens].1);
    assert!(!found[pens].1.contains("## Links"));
}
