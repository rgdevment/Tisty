use super::*;

fn doc(paper: Paper) -> DocCard {
    DocCard {
        file: "f2".into(),
        words: "Huerta".into(),
        paper,
        blurb: "Tomates · riego".into(),
        more: false,
    }
}

fn open_task(steps: Vec<Step>) -> Task {
    Task {
        title: "Llevar a Rina al veterinario".into(),
        standing: Standing::Open,
        list: Some("Casa".into()),
        date: None,
        deadline: None,
        closed: None,
        steps,
        parts: None,
    }
}

fn cited(task: Option<Task>, acting: bool) -> String {
    let card = TaskCard {
        id: "01JC".into(),
        words: "Rina & co".into(),
        task,
        acting,
    };
    task_card(&card, &SPANISH)
}

#[test]
fn a_document_card_opens_its_document_and_says_what_it_holds() {
    let drawn = doc_card(
        &doc(Paper::Doc {
            title: "Huerta".into(),
        }),
        &ENGLISH,
    );
    assert!(drawn.starts_with(
        r#"<span class="doc-card is-doc" contenteditable="false" tabindex="0" role="link" data-doc="f2"><span class="card-tile"><svg class="glyph""#
    ));
    assert!(drawn.contains(
        r#"<span class="card-name">Huerta</span><span class="card-blurb">Tomates · riego</span>"#
    ));
    assert!(drawn.ends_with(
        r#"<span class="card-meta"><span class="kind">Document</span><span class="card-go">›</span></span></span>"#
    ));
}

#[test]
fn a_page_card_says_where_it_stands_among_the_pages() {
    let drawn = doc_card(
        &doc(Paper::Page {
            title: "Riego".into(),
            at: 2,
            of: 5,
        }),
        &SPANISH,
    );
    assert!(drawn.contains(r#"class="doc-card is-page""#));
    assert!(drawn.contains(BOOK));
    assert!(drawn.contains(r#"<span class="chip">Página 2 de 5</span>"#));
}

#[test]
fn an_archived_card_still_opens_and_says_so() {
    let drawn = doc_card(
        &doc(Paper::Away {
            title: "Huerta".into(),
        }),
        &SPANISH,
    );
    assert!(drawn.contains(r#"data-doc="f2""#));
    assert!(drawn.contains(ARCHIVE));
    assert!(drawn.contains(r#"<span class="chip away">Archivado</span>"#));
}

#[test]
fn a_card_to_its_own_document_and_a_gone_one_open_nothing() {
    let itself = doc_card(&doc(Paper::Itself { title: " ".into() }), &ENGLISH);
    assert!(itself.starts_with(r#"<span class="doc-card is-self" contenteditable="false"><span"#));
    assert!(itself.contains(r#"<span class="card-name">Untitled</span><span class="card-blurb">This very document</span>"#));
    assert!(!itself.contains("card-go"));
    let gone = doc_card(&doc(Paper::Gone), &SPANISH);
    assert!(gone.starts_with(r#"<span class="doc-card is-gone" contenteditable="false"><span"#));
    assert!(gone.contains(r#"<span class="card-name">Huerta</span><span class="card-blurb">Ese documento no está en esta máquina</span>"#));
    assert!(gone.contains(r#"<span class="chip gone">Ya no está</span>"#));
}

#[test]
fn a_page_being_written_gives_each_card_its_more_button() {
    let card = DocCard {
        more: true,
        ..doc(Paper::Doc {
            title: "<Huerta>".into(),
        })
    };
    let drawn = doc_card(&card, &SPANISH);
    assert!(drawn.contains(r#"<span class="card-name">&lt;Huerta&gt;</span>"#));
    assert!(drawn.contains(
        r#"<button type="button" class="card-more" data-asked="more" data-target="tisty:doc/f2" aria-label="Más sobre esta tarjeta" title="Más sobre esta tarjeta">⋯</button><span class="card-go">›</span>"#
    ));
}

#[test]
fn an_open_task_card_names_its_list_and_leaves_its_days_to_the_window() {
    let task = Task {
        date: Some(When {
            at: "2026-11-20T00:00:00".into(),
            timed: false,
        }),
        deadline: Some(When {
            at: "2026-11-21T18:30:00".into(),
            timed: true,
        }),
        ..open_task(Vec::new())
    };
    let drawn = cited(Some(task), false);
    assert!(drawn.starts_with(
        r#"<span class="doc-card is-task is-open" contenteditable="false" tabindex="0" role="link" data-task="01JC">"#
    ));
    assert!(drawn.contains(
        r#"<span class="card-blurb">Casa · <time datetime="2026-11-20T00:00:00" data-say="when"></time> · vence <time datetime="2026-11-21T18:30:00" data-say="when" data-timed></time></span>"#
    ));
    assert!(drawn.contains(r#"<span class="card-meta"><span class="kind">Tarea</span><span class="card-go">›</span></span>"#));
    assert!(!drawn.contains("button"));
}

#[test]
fn a_window_that_can_mark_the_task_gets_its_circle_and_step_boxes() {
    let steps = vec![
        Step {
            id: "s1".into(),
            text: "Pedir **hora**".into(),
            done: true,
            said: false,
        },
        Step {
            id: "s2".into(),
            text: "Llevar la cartilla".into(),
            done: false,
            said: false,
        },
    ];
    let drawn = cited(Some(open_task(steps)), true);
    assert!(drawn.contains(
        r#"<button type="button" data-asked="taskDone" data-task="01JC" aria-label="Marcarla hecha" class="card-check" title="Marcarla hecha"><svg"#
    ));
    assert!(drawn.contains(
        r#"<button type="button" data-asked="taskSteps" data-task="01JC" aria-label="1 de 2 pasos" class="chip card-unfold" aria-expanded="false">1 de 2 pasos</button>"#
    ));
    assert!(drawn.contains(
        r#"<span class="card-step is-done"><button type="button" data-asked="taskStep" data-task="01JC" aria-label="Pedir hora" class="card-step-box" data-step="s1" role="checkbox" aria-checked="true"></button><span class="card-step-words">Pedir <strong>hora</strong></span></span>"#
    ));
    assert!(drawn.contains(r#"data-step="s2" role="checkbox" aria-checked="false">"#));
}

#[test]
fn a_step_an_agent_said_done_is_counted_apart_on_the_card() {
    let step = |id: &str, done: bool, said: bool| Step {
        id: id.into(),
        text: format!("paso {id}"),
        done,
        said,
    };
    let drawn = cited(
        Some(open_task(vec![
            step("s1", true, false),
            step("s2", false, true),
            step("s3", false, false),
        ])),
        true,
    );
    assert!(
        drawn.contains(r#"aria-expanded="false">1 de 3 pasos · 1 por confirmar</button>"#),
        "{drawn}"
    );
    let plain = cited(Some(open_task(vec![step("s1", true, false)])), true);
    assert!(!plain.contains("por confirmar"), "{plain}");
}

#[test]
fn a_window_that_only_reads_shows_the_steps_without_boxes_to_press() {
    let steps = vec![Step {
        id: "s1".into(),
        text: "Pedir hora".into(),
        done: false,
        said: false,
    }];
    let drawn = cited(Some(open_task(steps)), false);
    assert!(!drawn.contains("taskDone"));
    assert!(drawn.contains(r#"<span class="card-step"><span class="card-step-box"></span><span class="card-step-words">Pedir hora</span></span>"#));
}

#[test]
fn a_closed_task_says_how_it_ended_and_when() {
    let task = Task {
        standing: Standing::Dropped,
        closed: Some("2026-11-17T23:14:00Z".into()),
        parts: Some((1, 3)),
        ..open_task(Vec::new())
    };
    let drawn = cited(Some(task), true);
    assert!(drawn.starts_with(
        r#"<span class="doc-card is-task is-closed" contenteditable="false" tabindex="0""#
    ));
    assert!(drawn.contains(DONE));
    assert!(!drawn.contains("taskDone"));
    assert!(drawn.contains(
        r#"Casa · cerrada el <time datetime="2026-11-17T23:14:00Z" data-say="day"></time>"#
    ));
    assert!(drawn.contains(
        r#"<span class="chip">1 de 3 partes</span><span class="chip here">No se hará</span>"#
    ));
}

#[test]
fn a_task_that_is_not_here_keeps_its_words_and_opens_nothing() {
    let drawn = cited(None, true);
    assert_eq!(
        drawn,
        format!(
            r#"<span class="doc-card is-task is-gone" contenteditable="false" data-task="01JC"><span class="card-tile">{}</span><span class="card-name">Rina &amp; co</span><span class="card-blurb">Esa tarea no está en esta máquina</span><span class="card-meta"></span></span>"#,
            glyph(TODO)
        )
    );
}

#[test]
fn the_words_follow_the_language_spoken() {
    assert_eq!(words("es-CL").done, "Hecha");
    assert_eq!(words("en-US").done, "Done");
    assert_eq!(words("").done, "Done");
}

#[test]
fn a_step_is_drawn_as_a_line_of_the_page() {
    assert_eq!(step_html("Pedir *hora*"), "Pedir <em>hora</em>");
    assert_eq!(step_html(""), "");
}

#[test]
fn a_blurb_reads_the_words_under_the_title() {
    let body = "# Huerta\n\n## Riego\n> Cada ::sun:: mañana\n- tomates ::leaf green::\n* * *\nfin";
    assert_eq!(
        blurb(body),
        "Riego · Cada :sun: mañana · tomates :leaf: · fin"
    );
    assert_eq!(blurb("Solo título"), "");
    assert_eq!(
        blurb("t\n:: no es :: un ícono ::x y::"),
        ":: no es :: un ícono :x:"
    );
    assert_eq!(
        blurb("t\n:::a::: ::b-1 X:: ::c::d::"),
        "::a:: ::b-1 X:: :c:d::"
    );
}

#[test]
fn a_blurb_stops_where_the_window_cut_it() {
    let long = format!("t\n{}", "a".repeat(200));
    assert_eq!(blurb(&long).len(), 160);
    let faces = format!("t\n{}", "😀".repeat(100));
    assert_eq!(blurb(&faces).chars().count(), 80);
}

#[test]
fn an_example_card_is_drawn_by_the_words_it_was_written_with() {
    let drawn = doc_card(&DocCard::example("ab12-0001", "Viajes & más"), &SPANISH);
    assert!(drawn.contains(r#"data-doc="ab12-0001""#));
    assert!(drawn.contains(
        r#"<span class="card-name">Viajes &amp; más</span><span class="card-blurb"></span>"#
    ));
    assert!(drawn.contains(r#"<span class="kind">Documento</span>"#));
    assert!(!drawn.contains("card-more"));
}
