use super::*;
use tisty_core::Op;
use tisty_core::event::{DeviceId, Event, ListAdd, StepAdd, StepRef, TaskAdd};

fn applied(state: &mut State, ms: i64, op: Op) {
    let at = jiff::Timestamp::from_millisecond(ms).unwrap();
    state.apply(&Event::new(DeviceId("dev_a".into()), at, op));
}

fn written(
    state: &mut State,
    title: &str,
    list: Option<ulid::Ulid>,
    part_of: Option<TaskId>,
) -> TaskId {
    let id = ulid::Ulid::generate();
    let mut d = TaskAdd::new(title, "a0");
    d.list = list;
    d.part_of = part_of;
    applied(state, 1, Op::TaskAdd { id, d });
    id
}

fn kept(
    state: &mut State,
    room: &tempfile::TempDir,
    file: &str,
    title: &str,
    page_of: Option<ulid::Ulid>,
    archived: bool,
) -> ulid::Ulid {
    let id = ulid::Ulid::generate();
    let one: tisty_core::model::Kept = serde_json::from_value(serde_json::json!({
        "id": id.to_string(), "file": file, "order": file, "title": title, "print": "v1",
        "page_of": page_of.map(|of| of.to_string()), "archived": archived,
    }))
    .unwrap();
    state.docs.insert(id, one);
    std::fs::write(
        room.path().join(format!("{file}.md")),
        format!("# {title}\n\nDe {title}.\n"),
    )
    .unwrap();
    id
}

fn asked(kind: Named, id: &str, words: &str) -> Asked {
    Asked {
        kind,
        id: id.into(),
        words: words.into(),
    }
}

fn papers(known: &[Card]) -> Vec<&Paper> {
    known
        .iter()
        .filter_map(|one| match one {
            Card::Read { card, .. } | Card::Shown(card) => Some(&card.paper),
            Card::Task(_) => None,
        })
        .collect()
}

fn task_of(known: &[Card], at: usize) -> &TaskCard {
    match &known[at] {
        Card::Task(card) => card,
        Card::Read { .. } | Card::Shown(_) => panic!("a document card"),
    }
}

#[test]
fn a_documents_cards_say_what_each_names_as_the_main_window_does() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let here = kept(&mut state, &room, "here", "Aquí", None, false);
    kept(&mut state, &room, "p1", "Primera", Some(here), false);
    kept(&mut state, &room, "p2", "Segunda", Some(here), false);
    kept(&mut state, &room, "p3", "Guardada", Some(here), true);
    kept(&mut state, &room, "other", "Otra", None, false);
    kept(&mut state, &room, "old", "Vieja", None, true);
    kept(
        &mut state,
        &room,
        "unwritten-0001",
        "Sin archivo",
        None,
        false,
    );
    std::fs::remove_file(room.path().join("unwritten-0001.md")).unwrap();
    let files = [
        "p2",
        "p1",
        "other",
        "old",
        "here",
        "missing",
        "unwritten-0001",
    ];
    let all: Vec<Asked> = files
        .iter()
        .map(|one| asked(Named::Doc, one, "dicho"))
        .collect();
    let seen = Seen {
        here: Some("here".into()),
        ..Seen::default()
    };

    let known = known(&state, &all, &seen);

    let title = |said: &str| said.to_string();
    assert_eq!(
        papers(&known),
        vec![
            &Paper::Page {
                title: title("Segunda"),
                at: 1,
                of: 2
            },
            &Paper::Page {
                title: title("Primera"),
                at: 2,
                of: 2
            },
            &Paper::Doc {
                title: title("Otra")
            },
            &Paper::Away {
                title: title("Vieja")
            },
            &Paper::Itself {
                title: title("Aquí")
            },
            &Paper::Gone,
            &Paper::Doc {
                title: title("Sin archivo")
            },
        ]
    );
    let drawn = said(known, room.path(), &Glimpses::default(), &cards::SPANISH);
    assert!(
        drawn[6].starts_with(r#"<span class="doc-card is-gone" contenteditable="false">"#),
        "{}",
        drawn[6]
    );
}

#[test]
fn an_open_card_brings_its_blurb_read_once_for_each_version() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let id = kept(&mut state, &room, "huerta-0001", "Huerta", None, false);
    let glimpses = Glimpses::default();
    let all = [
        asked(Named::Doc, "huerta-0001", "Huerta"),
        asked(Named::Doc, "lejos", "Lejos"),
    ];
    let drawn = |state: &State| {
        let known = known(state, &all, &Seen::default());
        said(known, room.path(), &glimpses, &cards::SPANISH)
    };

    let first = drawn(&state);
    assert!(first[0].contains(r#"<span class="card-blurb">De Huerta.</span>"#));
    assert!(first[1].contains(r#"<span class="card-name">Lejos</span><span class="card-blurb">Ese documento no está en esta máquina</span>"#));

    std::fs::write(room.path().join("huerta-0001.md"), "# Huerta\n\nRiego.\n").unwrap();
    assert!(drawn(&state)[0].contains("De Huerta."));
    state.docs.get_mut(&id).unwrap().print = Some("v2".into());
    assert!(drawn(&state)[0].contains(r#"<span class="card-blurb">Riego.</span>"#));
}

#[test]
fn a_cited_task_brings_its_list_its_steps_and_its_parts() {
    let mut state = State::default();
    let list = ulid::Ulid::generate();
    let d = ListAdd {
        name: "Casa".into(),
        order: "a0".into(),
        color: None,
    };
    applied(&mut state, 1, Op::ListAdd { id: list, d });
    let move_house = written(&mut state, "Mudanza", Some(list), None);
    let quotes = ulid::Ulid::generate();
    let d = StepAdd {
        step: quotes,
        text: "Pedir **tres** presupuestos".into(),
        order: "a0".into(),
    };
    applied(&mut state, 2, Op::StepAdd { id: move_house, d });
    applied(
        &mut state,
        3,
        Op::StepDone {
            id: move_house,
            d: StepRef { step: quotes },
        },
    );
    let pack = written(&mut state, "Embalar", Some(list), Some(move_house));
    written(&mut state, "Contratar el camión", None, Some(move_house));
    applied(
        &mut state,
        4,
        Op::TaskDone {
            id: pack,
            filled: false,
        },
    );
    let all = [
        asked(Named::Task, &move_house.to_string(), "la mudanza"),
        asked(Named::Task, &pack.to_string(), "embalar"),
    ];
    let seen = Seen {
        here: Some(move_house.to_string()),
        acting: true,
        ..Seen::default()
    };

    let known = known(&state, &all, &seen);

    let whole = task_of(&known, 0);
    assert!(whole.acting);
    let facts = whole.task.as_ref().unwrap();
    assert_eq!(facts.title, "Mudanza");
    assert_eq!(facts.list.as_deref(), Some("Casa"));
    assert_eq!(facts.parts, Some((1, 2)));
    assert_eq!(
        facts.steps,
        vec![Step {
            id: quotes.to_string(),
            text: "Pedir **tres** presupuestos".into(),
            done: true,
        }]
    );
    let part = task_of(&known, 1).task.as_ref().unwrap();
    assert_eq!(part.standing, Standing::Done);
    assert_eq!(part.list, None);
    assert!(part.closed.is_some());
}

#[test]
fn a_task_not_here_keeps_its_words_and_a_dated_one_leaves_its_days_to_the_window() {
    let mut state = State::default();
    let tyres = written(&mut state, "Cambiar los neumáticos", None, None);
    let at: jiff::civil::DateTime = "2026-11-20T18:30:00".parse().unwrap();
    state.tasks.get_mut(&tyres).unwrap().date = Some(DateSpec {
        at,
        tz: "America/Santiago".into(),
        floating: true,
        has_time: true,
    });
    let erased = written(&mut state, "Borrada", None, None);
    applied(&mut state, 6, Op::TaskDelete { id: erased });
    let all = [
        asked(Named::Task, &tyres.to_string(), "neumáticos"),
        asked(Named::Task, &erased.to_string(), "la borrada"),
        asked(Named::Task, "not an id", "nada"),
    ];

    let known = known(&state, &all, &Seen::default());

    let dated = task_of(&known, 0).task.as_ref().unwrap();
    assert_eq!(
        dated.date,
        Some(When {
            at: "2026-11-20T18:30:00".into(),
            timed: true,
        })
    );
    assert_eq!(task_of(&known, 1).task, None);
    assert_eq!(task_of(&known, 1).words, "la borrada");
    assert_eq!(task_of(&known, 2).task, None);
}

#[test]
fn a_help_page_draws_its_examples_by_their_words_and_reads_no_store() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    kept(&mut state, &room, "huerta", "Huerta", None, false);
    let seen = Seen {
        examples: true,
        ..Seen::default()
    };
    let all = [
        asked(Named::Doc, "huerta", "Viajes"),
        asked(Named::Task, "01JC", "Rina"),
    ];

    let known = known(&state, &all, &seen);

    assert_eq!(
        papers(&known),
        vec![&Paper::Doc {
            title: "Viajes".into()
        }]
    );
    let drawn = said(known, room.path(), &Glimpses::default(), &cards::ENGLISH);
    assert!(drawn[0].contains(r#"<span class="card-blurb"></span>"#));
    assert!(drawn[1].contains("is-gone"));
}
