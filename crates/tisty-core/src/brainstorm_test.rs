use super::*;

#[test]
fn a_new_brainstorm_asks_for_the_notes_view_and_names_itself_the_capture_place() {
    let body = opening("Lluvia de ideas");
    assert!(tisty_markdown::front::noted(&body));
    assert!(tisty_markdown::front::brainstorm(&body));
    assert_eq!(crate::docs::titled(&body), "Lluvia de ideas");
}

#[test]
fn an_idea_is_a_sticky_note_headed_by_the_minute_it_came_and_its_colour() {
    let at = jiff::civil::date(2026, 11, 11).at(9, 7, 42, 0);
    assert_eq!(
        jotted("  llamar al vet\n", at, "blue"),
        "> [!IDEA] 2026-11-11 09:07 · blue\n> llamar al vet"
    );
    assert_eq!(
        jotted("una\n\ndos", at, "pink"),
        "> [!IDEA] 2026-11-11 09:07 · pink\n> una\n>\n> dos"
    );
}

#[test]
fn a_new_idea_never_takes_the_colour_of_the_one_before_it() {
    let mut body = opening("Ideas");
    let mut before: Option<&str> = None;
    for roll in [0, 0, 12, 24, 5, 17, 29, 41] {
        let colour = coloured(&body, roll);
        assert!(COLOURS.contains(&colour));
        assert_ne!(Some(colour), before, "{colour} repeated");
        body.push_str(&format!(
            "\n\n> [!IDEA] 2026-11-12 00:19 · {colour}\n> idea"
        ));
        before = Some(colour);
    }
}

#[test]
fn every_sticky_note_outside_a_fence_counts_as_one_idea_waiting() {
    let mut body = opening("Ideas");
    assert_eq!(ideas(&body), 0);
    body.push_str("\n> [!IDEA] 2026-11-12 00:19 · blue\n> una\n\n> [!IDEA] 2026-11-12 00:20 · pink\n> ```md\n> > [!IDEA] no es idea\n> ```\n\n> [!NOTE] tampoco\n");
    assert_eq!(ideas(&body), 2);
}

#[test]
fn a_filed_idea_keeps_its_minute_and_colour_and_sits_to_the_right() {
    assert_eq!(
        filed(
            " Probar con harina integral\nY menos sal ",
            "2026-11-12 09:40 · yellow"
        ),
        "> [!IDEA] 2026-11-12 09:40 · yellow · right\n> Probar con harina integral\n> Y menos sal"
    );
    assert_eq!(
        filed("ya tiene lado", "2026-11-12 09:40 · left"),
        "> [!IDEA] 2026-11-12 09:40 · left\n> ya tiene lado"
    );
}

const SHEET: &str = "---\nview: notes\nbrainstorm: on\n---\n\n# Ideas\n\n> [!IDEA] 2026-11-12 09:40 · yellow\n> pan con masa madre\n\n> [!IDEA] 2026-06-14 08:12 · pink\n> entradas del concierto\n> de julio\n> [!idea] 2026-11-11 22:17 · blue\n> ```md\n> > [!IDEA] no es idea\n> ```\n";

#[test]
fn each_idea_is_read_by_its_head_and_its_words() {
    let on = ideas_on(SHEET);
    assert_eq!(on.len(), 3);
    assert_eq!(on[1].heading, "2026-06-14 08:12 · pink");
    assert_eq!(on[1].text, "entradas del concierto\nde julio");
    assert_eq!(on[2].heading, "2026-11-11 22:17 · blue");
    assert_eq!(on[2].text, "```md\n> [!IDEA] no es idea\n```");
}

#[test]
fn taking_an_idea_off_leaves_every_other_byte_where_it_was() {
    let gone = &ideas_on(SHEET)[1];
    assert_eq!(
        without(SHEET, gone).unwrap(),
        "---\nview: notes\nbrainstorm: on\n---\n\n# Ideas\n\n> [!IDEA] 2026-11-12 09:40 · yellow\n> pan con masa madre\n\n> [!idea] 2026-11-11 22:17 · blue\n> ```md\n> > [!IDEA] no es idea\n> ```\n"
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
    let note = filed(
        "entradas del concierto\nde julio",
        "2026-06-14 08:12 · pink",
    );
    assert_eq!(
        quoted(&note).as_deref(),
        Some("entradas del concierto\nde julio")
    );
    assert_eq!(quoted("> [!IDEA] pink\n> sin fecha"), None);
    assert_eq!(
        quoted("> [!IDEA] 2026-11-12 · pink\nfuera de la cita"),
        None
    );
    assert!(files_an_idea(
        "> [!IDEA] 2026-11-12 · pink\nfuera de la cita"
    ));
    assert!(!files_an_idea("> [!IDEA] blue\n> sin fecha"));
    assert!(!files_an_idea("> [!NOTE] Ideas · 2026-11-12\n> una nota"));
}

#[test]
fn ideas_jotted_together_never_repeat_the_colour_just_before() {
    let at = jiff::civil::date(2026, 11, 12).at(10, 5, 0, 0);
    let each: Vec<String> = ["una", "dos", "tres", "cuatro"].map(String::from).to_vec();
    let added = jotted_all(SHEET, &each, at, || 0);
    let heads: Vec<&str> = added
        .lines()
        .filter(|line| line.starts_with("> [!IDEA] "))
        .collect();
    assert_eq!(heads.len(), 4);
    assert!(
        heads
            .iter()
            .all(|one| one.starts_with("> [!IDEA] 2026-11-12 10:05 · "))
    );
    for pair in heads.windows(2) {
        assert_ne!(pair[0], pair[1]);
    }
}

#[test]
fn a_sheet_says_so_in_its_note_and_no_other_document_does() {
    let sheet = crate::event::Said::of(SHEET);
    let plain = crate::event::Said::of("# Recetas\n\nPan.\n");
    assert!(sheet.brainstorm);
    assert!(!plain.brainstorm);
    assert!(
        !serde_json::to_string(&plain)
            .unwrap()
            .contains("brainstorm")
    );
}
