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
        jotted("  llamar al vet\n", at, "blue"),
        "## 2026-11-11 09:07 · blue\n\nllamar al vet"
    );
}

#[test]
fn a_new_idea_never_takes_the_colour_of_the_one_before_it() {
    let mut body = opening("Ideas");
    for roll in [0, 0, 12, 24, 5, 17, 29, 41] {
        let before = body
            .lines()
            .rev()
            .find(|line| line.starts_with("## "))
            .map(str::to_string);
        let colour = coloured(&body, roll);
        assert!(COLOURS.contains(&colour));
        if let Some(line) = before {
            assert!(!line.ends_with(colour), "{colour} repeated after {line}");
        }
        body.push_str(&format!("\n\n## 2026-11-12 00:19 · {colour}\n\nidea"));
    }
}
