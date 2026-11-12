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

#[test]
fn every_section_outside_a_fence_counts_as_one_idea_waiting() {
    let mut body = opening("Ideas");
    assert_eq!(ideas(&body), 0);
    body.push_str("\n## 2026-11-12 00:19 · blue\n\nuna\n\n## 2026-11-12 00:20 · pink\n\n```md\n## no es idea\n```\n");
    assert_eq!(ideas(&body), 2);
}

#[test]
fn a_filed_idea_is_a_note_that_says_where_and_when_it_came_from() {
    let on = jiff::civil::date(2026, 11, 12);
    assert_eq!(
        filed(
            " Probar con harina integral\nY menos sal ",
            "Lluvia de ideas",
            on
        ),
        "> [!NOTE] Lluvia de ideas · 2026-11-12\n> Probar con harina integral\n> Y menos sal"
    );
}

const SHEET: &str = "---\nview: notes\nbrainstorm: on\n---\n\n# Ideas\n\n## 2026-11-12 09:40 · yellow\n\npan con masa madre\n\n## 2026-06-14 08:12 · pink\nentradas del concierto\nde julio\n## 2026-11-11 22:17 · blue\n\n```md\n## no es idea\n```\n";

#[test]
fn each_idea_is_read_by_its_heading_and_its_words() {
    let on = ideas_on(SHEET);
    assert_eq!(on.len(), 3);
    assert_eq!(on[1].heading, "2026-06-14 08:12 · pink");
    assert_eq!(on[1].text, "entradas del concierto\nde julio");
    assert_eq!(on[2].text, "```md\n## no es idea\n```");
}

#[test]
fn taking_an_idea_off_leaves_every_other_byte_where_it_was() {
    let gone = &ideas_on(SHEET)[1];
    assert_eq!(
        without(SHEET, gone).unwrap(),
        "---\nview: notes\nbrainstorm: on\n---\n\n# Ideas\n\n## 2026-11-12 09:40 · yellow\n\npan con masa madre\n\n## 2026-11-11 22:17 · blue\n\n```md\n## no es idea\n```\n"
    );
    let windows = SHEET.replace('\n', "\r\n");
    assert_eq!(
        without(&windows, gone).unwrap(),
        without(SHEET, gone).unwrap().replace('\n', "\r\n")
    );
    let reworded = Idea {
        heading: gone.heading.clone(),
        text: "otra cosa".into(),
    };
    assert_eq!(
        without(SHEET, &reworded),
        None,
        "an idea changed since was taken off"
    );
}

#[test]
fn a_filed_note_gives_back_the_idea_it_quotes() {
    let on = jiff::civil::date(2026, 11, 12);
    let note = filed("entradas del concierto\nde julio", "Lluvia de ideas", on);
    assert_eq!(
        quoted(&note).as_deref(),
        Some("entradas del concierto\nde julio")
    );
    assert_eq!(quoted("> [!NOTE] Lluvia de ideas\n> sin fecha"), None);
    assert_eq!(
        quoted("> [!NOTE] Ideas · 2026-11-12\nfuera de la cita"),
        None
    );
}

#[test]
fn ideas_jotted_together_never_repeat_the_colour_just_before() {
    let at = jiff::civil::date(2026, 11, 12).at(10, 5, 0, 0);
    let each: Vec<String> = ["una", "dos", "tres", "cuatro"].map(String::from).to_vec();
    let added = jotted_all(SHEET, &each, at, || 0);
    let headings: Vec<&str> = added
        .lines()
        .filter(|line| line.starts_with("## "))
        .collect();
    assert_eq!(headings.len(), 4);
    assert!(
        headings
            .iter()
            .all(|one| one.starts_with("## 2026-11-12 10:05 · "))
    );
    for pair in headings.windows(2) {
        assert_ne!(pair[0], pair[1]);
    }
}
