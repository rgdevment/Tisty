use super::*;

#[test]
fn a_new_brainstorm_asks_for_the_notes_view_and_names_itself_the_capture_place() {
    let body = opening("Lluvia de ideas");
    assert!(tisty_markdown::front::noted(&body));
    assert!(tisty_markdown::front::brainstorm(&body));
    assert_eq!(crate::docs::titled(&body), "Lluvia de ideas");
}

#[test]
fn an_idea_is_its_own_section_headed_by_the_minute_it_came() {
    let at = jiff::civil::date(2026, 11, 11).at(9, 7, 42, 0);
    assert_eq!(
        jotted("  llamar al vet\n", at),
        "## 2026-11-11 09:07\n\nllamar al vet"
    );
}
