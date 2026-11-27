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
    let at = jiff::civil::date(2026, 11, 12).at(0, 19, 0, 0);
    let mut body = opening("Ideas");
    let mut before: Option<&str> = None;
    for (minute, roll) in [0, 0, 12, 24, 5, 17, 29, 41].into_iter().enumerate() {
        let at = at
            .checked_add(jiff::Span::new().minutes(minute as i64))
            .unwrap();
        let note = jotted_next(&body, &[], "idea", at, roll);
        let colour = tisty_markdown::idea_head(&note).hue;
        assert!(colour.is_some());
        assert_ne!(colour, before, "{colour:?} repeated");
        body.push_str(&format!("\n\n{note}"));
        before = colour;
    }
}

#[test]
fn no_two_ideas_on_a_sheet_share_their_minute_and_colour() {
    let at = jiff::civil::date(2026, 11, 12).at(16, 18, 0, 0);
    let each: Vec<String> = (1..=14).map(|n| format!("idea {n}")).collect();
    for roll in [0, 7, 12, 25] {
        let mut sheet = format!("{SHEET}\n> [!IDEA] 2026-11-12 16:18 · green\n> ya estaba\n");
        let added = jotted_all(&sheet, &[], &each, at, || roll);
        sheet.push_str(&format!("\n{added}\n"));
        let heads: Vec<String> = ideas_on(&sheet)
            .into_iter()
            .map(|one| one.heading)
            .collect();
        assert_eq!(heads.len(), 18);
        for (at, one) in heads.iter().enumerate() {
            assert!(
                !heads[at + 1..].contains(one),
                "{one} twice with roll {roll}"
            );
        }
        let next_minute = heads
            .iter()
            .filter(|one| one.starts_with("2026-11-12 16:19 · "));
        assert_eq!(next_minute.count(), 3);
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
    assert_eq!(
        filed("sin cabecera", "  "),
        "> [!IDEA] right\n> sin cabecera"
    );
}

const SHEET: &str = "---\nview: notes\nbrainstorm: on\n---\n\n# Ideas\n\n> [!IDEA] 2026-11-12 09:40 · yellow\n> pan con masa madre\n\n> [!IDEA] 2026-06-14 08:12 · pink\n> entradas del concierto\n> de julio\n\n> [!idea] 2026-11-11 22:17 · blue\n> ```md\n> > [!IDEA] no es idea\n> ```\n";

#[test]
fn a_fence_left_open_inside_an_idea_ends_with_it() {
    let open = "# Ideas\n\n> [!IDEA] 2026-11-12 09:40 · pink\n> ```\n\n> [!IDEA] 2026-11-12 09:41 · blue\n> dos\n";

    assert_eq!(ideas(open), 2);
    assert_eq!(ideas_on(open)[1].text, "dos");
}

#[test]
fn an_idea_written_with_its_marks_indented_reads_as_its_words() {
    let indented = "# Ideas\n\n  > [!IDEA] 2026-11-12 09:40 · pink\n  > comprar pan\n";

    assert_eq!(ideas_on(indented)[0].text, "comprar pan");
    assert_eq!(
        quoted("  > [!IDEA] 2026-11-12 · pink\n  > comprar pan").as_deref(),
        Some("comprar pan")
    );
}

#[test]
fn a_marker_right_under_a_quoted_line_stays_in_that_idea_as_markdown_reads_it() {
    let joined = "# Ideas\n\n> [!IDEA] 2026-11-12 09:40 · pink\n> uno\n> [!IDEA] 2026-11-12 09:41 · blue\n> dos\n";

    assert_eq!(ideas(joined), 1);
    assert_eq!(
        ideas_on(joined)[0].text,
        "uno\n[!IDEA] 2026-11-12 09:41 · blue\ndos"
    );
    assert_eq!(last_hue(joined), Some("pink"));
}

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
    let added = jotted_all(SHEET, &[], &each, at, || 0);
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

#[test]
fn a_heading_an_idea_left_behind_as_a_source_is_never_taken_again() {
    let at = jiff::civil::date(2026, 11, 12).at(16, 18, 0, 0);
    let used: Vec<String> = COLOURS
        .iter()
        .map(|hue| format!("2026-11-12 16:18 · {hue}"))
        .collect();

    let added = jotted_all(SHEET, &used, &["nueva".to_string()], at, || 0);

    assert!(
        added.starts_with("> [!IDEA] 2026-11-12 16:19 · "),
        "{added}"
    );
}

#[test]
fn an_idea_is_named_by_its_heading_with_or_without_its_marks() {
    let heading = "2026-11-12 09:40 · yellow";
    for said in [
        heading.to_string(),
        format!("[!IDEA] {heading}"),
        format!("> [!idea] {heading}"),
        format!("  {heading} "),
    ] {
        assert_eq!(heading_named(&said), heading, "{said}");
    }
}

#[test]
fn a_note_stuck_beside_the_words_says_its_minute_and_a_colour_the_one_before_it_lacks() {
    let at = jiff::civil::date(2026, 11, 15).at(14, 30, 0, 0);
    let was = "# Noviembre\n\n## Domingo 15\n\n> [!IDEA] 2026-11-15 14:30 · pink · right\n> una\n";
    let note = stuck_beside(was, "Llamar a Jaime", at, 0);
    let head = tisty_markdown::idea_head(&note);
    assert_eq!(head.when, Some("2026-11-15 14:30"));
    assert_eq!(head.side, Some("right"));
    assert_ne!(head.hue, Some("pink"));
    assert!(note.ends_with("\n> Llamar a Jaime"), "{note}");
}

const PLAIN: &str = "# Recetas\n\nPan.\n";

fn library(docs: &[(&str, &str, bool)]) -> (tempfile::TempDir, State) {
    let tmp = tempfile::tempdir().unwrap();
    let mut events = Vec::new();
    for (at, (file, body, marked)) in docs.iter().enumerate() {
        crate::docs::create_named(tmp.path(), file, body).unwrap();
        let said = marked.then(|| crate::event::Said::of(body));
        let op = crate::Op::DocAdd {
            id: ulid::Ulid::from_parts(at as u64 + 1, 0),
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                sown: false,
                made: None,
                by: None,
                said,
                file: (*file).into(),
                order: format!("a{at}"),
                folder: None,
                page_of: None,
                task_of: None,
                edited: None,
            },
        };
        events.push(crate::event::Event::new(
            crate::event::DeviceId("dev_mine".into()),
            jiff::Timestamp::from_millisecond(at as i64 + 1).unwrap(),
            op,
        ));
    }
    (tmp, State::replay(&events))
}

#[test]
fn a_sheet_nobody_marked_is_only_found_when_the_whole_library_may_be_read() {
    let (tmp, state) = library(&[("mine-1", PLAIN, false), ("mine-2", SHEET, false)]);

    assert!(find(tmp.path(), &state, None, false).is_none());
    let (one, body) = find(tmp.path(), &state, None, true).unwrap();
    assert_eq!(one.file, "mine-2");
    assert!(tisty_markdown::front::brainstorm(&body));
}

#[test]
fn a_document_marked_as_a_brainstorm_is_found_without_reading_the_whole_library() {
    let (tmp, state) = library(&[("mine-1", PLAIN, false), ("mine-2", SHEET, true)]);

    let (one, body) = find(tmp.path(), &state, None, false).unwrap();

    assert_eq!(one.file, "mine-2");
    assert_eq!(body, SHEET);
}

#[test]
fn the_remembered_sheet_is_preferred_to_an_older_marked_one() {
    let (tmp, state) = library(&[("mine-1", SHEET, true), ("mine-2", SHEET, true)]);

    let (kept, _) = find(tmp.path(), &state, Some("mine-2"), false).unwrap();
    let (oldest, _) = find(tmp.path(), &state, None, false).unwrap();

    assert_eq!(kept.file, "mine-2");
    assert_eq!(oldest.file, "mine-1");
}

#[test]
fn a_remembered_sheet_that_stopped_saying_so_is_let_go() {
    let (tmp, state) = library(&[("mine-1", SHEET, true), ("mine-2", PLAIN, true)]);

    let (one, _) = find(tmp.path(), &state, Some("mine-2"), false).unwrap();

    assert_eq!(one.file, "mine-1");
}
