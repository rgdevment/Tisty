use super::{GLANCE_AT_MOST, glanced};

const TRIPS: &str = "---\ntags: [x]\n---\n# Trips\n\nWhere we went.\n\n## Hotels\n\nThe one by the sea.\n\n### Rooms\n\nTwo beds.\n\n## Food\n\nFish.\n";

#[test]
fn a_heading_glances_at_its_own_section_and_the_ones_under_it() {
    let said = glanced(TRIPS, Some("hotels"));

    assert_eq!(said.title, "Trips");
    assert_eq!(said.heading.as_deref(), Some("Hotels"));
    assert!(said.html.contains("The one by the sea."), "{}", said.html);
    assert!(said.html.contains("Two beds."), "{}", said.html);
    assert!(!said.html.contains("Fish."), "{}", said.html);
    assert!(!said.html.contains(">Hotels<"), "the heading is said apart");
    assert!(!said.more);
}

#[test]
fn no_heading_or_one_gone_glances_at_the_first_words_past_the_title() {
    for at in [None, Some("gone")] {
        let said = glanced(TRIPS, at);

        assert_eq!(said.heading, None);
        assert!(said.html.contains("Where we went."), "{}", said.html);
        assert!(!said.html.contains(">Trips<"), "{}", said.html);
        assert!(!said.html.contains("tags"), "{}", said.html);
    }
}

#[test]
fn a_long_section_is_cut_where_a_paragraph_ends() {
    let paragraph = "word ".repeat(60);
    let body = format!("# Long\n\n{}", [paragraph.trim(); 10].join("\n\n"));

    let said = glanced(&body, None);

    assert!(said.more);
    let kept = said.html.matches("<p").count();
    assert!(
        kept > 0 && kept * paragraph.len() <= GLANCE_AT_MOST + paragraph.len(),
        "{kept} paragraphs"
    );
}

#[test]
fn a_cut_never_splits_a_letter() {
    let body = format!("# Ñ\n\n{}", "ñ".repeat(GLANCE_AT_MOST));

    let said = glanced(&body, None);

    assert!(said.more);
    assert!(said.html.contains('ñ'));
}
