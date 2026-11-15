use super::*;
use crate::event::Event;

fn day(said: &str) -> Date {
    said.parse().unwrap()
}

fn ev(ms: i64, device: &str, op: Op) -> Event {
    Event::new(
        DeviceId(device.into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

fn here() -> DeviceId {
    DeviceId("dev_here".into())
}

fn opened_into(root: &Path, state: &mut State, date: Date, words: &Words) -> Opening {
    let device = here();
    let maker = Maker {
        root,
        device: &device,
        by: None,
    };
    let Opened::Ready(opening) = opened(&maker, state, date, words).unwrap() else {
        panic!("the diary was locked");
    };
    for (n, op) in opening.ops.iter().enumerate() {
        state.apply(&ev(1_000 + n as i64, "dev_here", op.clone()));
    }
    opening
}

#[test]
fn a_book_says_diary_on_and_its_year_and_each_page_and_day_are_said_in_the_app_s_language() {
    let book = book(2026, &SPANISH);
    assert!(tisty_markdown::front::diary(&book));
    assert_eq!(crate::docs::titled(&book), "Diario 2026");
    assert_eq!(
        crate::docs::titled(&super::book(2026, &ENGLISH)),
        "Diary 2026"
    );
    assert_eq!(page(11, &SPANISH), "# Noviembre\n");
    assert_eq!(page(11, &ENGLISH), "# November\n");
    assert_eq!(day_title(day("2026-11-15"), &SPANISH), "Domingo 15");
    assert_eq!(day_title(day("2026-11-16"), &ENGLISH), "Monday 16");
    assert_eq!(day_title(day("2026-11-18"), &SPANISH), "Miércoles 18");
    assert!(std::ptr::eq(words_for("es-CL"), &SPANISH));
    assert!(std::ptr::eq(words_for("en-US"), &ENGLISH));
    assert!(std::ptr::eq(words_for("pt-BR"), &ENGLISH));
}

#[test]
fn a_month_is_read_in_any_language_tisty_speaks_and_a_day_by_the_number_in_its_heading() {
    assert_eq!(month_named("Noviembre"), Some(11));
    assert_eq!(month_named("november"), Some(11));
    assert_eq!(month_named("Mayo 2026"), Some(5));
    assert_eq!(month_named("Notas"), None);
    assert_eq!(year_named("Diario 2026"), Some(2026));
    assert_eq!(year_named("Mi diario"), None);
    assert_eq!(day_named("Domingo 15"), Some(15));
    assert_eq!(day_named("Sunday 3"), Some(3));
    assert_eq!(day_named("15 de noviembre de 2026"), Some(15));
    assert_eq!(day_named("Ideas 2027"), None);
    assert_eq!(day_named("Lunes 32"), None);
    assert_eq!(day_named("Notas"), None);
}

#[test]
fn the_first_words_of_a_day_open_its_section_under_the_month() {
    let now = written_on(
        "# Noviembre\n",
        day("2026-11-15"),
        &SPANISH,
        "Paseo con Rina.\n",
    );
    assert_eq!(now, "# Noviembre\n\n## Domingo 15\n\nPaseo con Rina.\n");
}

#[test]
fn more_words_the_same_day_go_at_the_end_of_its_section_and_nothing_else_moves() {
    let was = "# Noviembre\n\n## Domingo 15\n\nPaseo con Rina.\n\n### Tarde\n\nLlamé a Jaime.\n\n\n## Sábado 14\n\nNada.\n";
    let now = written_on(was, day("2026-11-15"), &SPANISH, "Cena con Marisol.");
    assert_eq!(
        now,
        "# Noviembre\n\n## Domingo 15\n\nPaseo con Rina.\n\n### Tarde\n\nLlamé a Jaime.\n\nCena con Marisol.\n\n## Sábado 14\n\nNada.\n"
    );
}

#[test]
fn a_new_day_goes_on_top_above_the_older_ones() {
    let was = "# Noviembre\n\nLo que fue el mes.\n\n## Sábado 14\n\nNada.\n";
    let now = written_on(was, day("2026-11-15"), &SPANISH, "Paseo.");
    assert_eq!(
        now,
        "# Noviembre\n\nLo que fue el mes.\n\n## Domingo 15\n\nPaseo.\n\n## Sábado 14\n\nNada.\n"
    );
}

#[test]
fn a_switch_of_language_writes_in_the_same_day_and_names_a_new_one_in_the_new_language() {
    let was = "# Noviembre\n\n## Domingo 15\n\nPaseo.\n";
    let same = written_on(was, day("2026-11-15"), &ENGLISH, "Dinner.");
    assert_eq!(same, "# Noviembre\n\n## Domingo 15\n\nPaseo.\n\nDinner.\n");
    let next = written_on(was, day("2026-11-16"), &ENGLISH, "Work.");
    assert_eq!(
        next,
        "# Noviembre\n\n## Monday 16\n\nWork.\n\n## Domingo 15\n\nPaseo.\n"
    );
}

#[test]
fn a_heading_written_in_the_day_sits_under_it_and_a_windows_page_keeps_its_line_ends() {
    let now = written_on(
        "# Noviembre\r\n",
        day("2026-11-15"),
        &SPANISH,
        "# Ideas\nuna\n```\n# code\n```",
    );
    assert_eq!(
        now,
        "# Noviembre\r\n\r\n## Domingo 15\r\n\r\n### Ideas\r\nuna\r\n```\r\n# code\r\n```\r\n"
    );
    assert_eq!(days_on(&now), vec![15]);
}

#[test]
fn the_first_words_of_a_year_make_its_book_and_its_month_and_the_book_names_the_month() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let opening = opened_into(room.path(), &mut state, day("2026-11-15"), &SPANISH);
    assert!(opening.book_made && opening.page_made);
    let book = state
        .docs
        .values()
        .find(|one| one.file == opening.book)
        .unwrap();
    let page = state
        .docs
        .values()
        .find(|one| one.file == opening.page)
        .unwrap();
    assert!(book.diary);
    assert_eq!(book.title.as_deref(), Some("Diario 2026"));
    assert_eq!(page.page_of, Some(book.id));
    assert_eq!(page.title.as_deref(), Some("Noviembre"));
    assert!(within(&state, page) && within(&state, book));
    assert_eq!(month_of_page(&state, page), Some((2026, 11)));

    let whole = named(room.path(), room.path(), &opening).unwrap().unwrap();
    assert!(
        whole.contains(&format!("tisty:doc/{}", opening.page)),
        "{whole}"
    );

    let again = opened_into(room.path(), &mut state, day("2026-11-30"), &ENGLISH);
    assert!(!again.book_made && !again.page_made && again.ops.is_empty());
    assert_eq!((again.book, again.page), (opening.book, opening.page));
}

#[test]
fn the_last_day_of_a_year_and_the_first_of_the_next_write_in_their_own_books() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let december = opened_into(room.path(), &mut state, day("2026-12-31"), &SPANISH);
    let january = opened_into(room.path(), &mut state, day("2027-01-01"), &SPANISH);
    assert!(january.book_made && january.page_made);
    assert_ne!(december.book, january.book);
    let book = state
        .docs
        .values()
        .find(|one| one.file == january.book)
        .unwrap();
    assert_eq!(book.title.as_deref(), Some("Diario 2027"));
    let found = book_of(room.path(), &state, 2026).unwrap();
    assert_eq!(found.file, december.book);
}

#[test]
fn of_two_books_for_one_year_the_oldest_is_the_diary_and_one_that_stopped_saying_so_is_not() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let first = opened_into(room.path(), &mut state, day("2026-11-15"), &SPANISH);
    let second = crate::docs::create(room.path(), &here(), &super::book(2026, &ENGLISH)).unwrap();
    state.apply(&ev(
        5_000,
        "dev_there",
        Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: DocAdd {
                file: second.id.clone(),
                order: "z".into(),
                said: Some(Said::of(&super::book(2026, &ENGLISH))),
                ..DocAdd::default()
            },
        },
    ));
    assert_eq!(book_of(room.path(), &state, 2026).unwrap().file, first.book);

    crate::docs::write(room.path(), &first.book, "# Diario 2026\n").unwrap();
    assert_eq!(book_of(room.path(), &state, 2026).unwrap().file, second.id);
}

#[test]
fn only_the_person_opens_the_diary_to_agents() {
    let mut state = State::default();
    state.apply(&ev(
        1,
        "dev_agent",
        Op::DeviceJoin {
            d: DeviceId("dev_agent".into()),
            k: Some(crate::event::DeviceKind::Agent),
            p: None,
        },
    ));
    assert!(state.assistants.contains(&DeviceId("dev_agent".into())));
    state.apply(&ev(2, "dev_agent", Op::DiaryAgents { open: true }));
    assert!(!state.diary_open, "an agent cannot open it for itself");
    state.apply(&ev(3, "dev_here", Op::DiaryAgents { open: true }));
    assert!(state.diary_open);
    state.apply(&ev(4, "dev_here", Op::DiaryAgents { open: false }));
    assert!(!state.diary_open);
}

#[test]
fn the_tasks_closed_on_a_day_are_the_ones_done_that_day_and_not_hidden() {
    let mut state = State::default();
    let zone = jiff::tz::TimeZone::UTC;
    let noon = day("2026-11-15")
        .at(12, 0, 0, 0)
        .to_zoned(zone.clone())
        .unwrap();
    let ms = noon.timestamp().as_millisecond();
    let mut add = |title: &str, at: i64| {
        let id = ulid::Ulid::generate();
        state.apply(&ev(
            at,
            "dev_here",
            Op::TaskAdd {
                id,
                d: crate::event::TaskAdd::new(title, "a0"),
            },
        ));
        id
    };
    let walk = add("Pasear a Rina", ms - 10);
    let vet = add("Vacuna de Roger", ms - 9);
    let hidden = add("Algo mío", ms - 8);
    let dropped = add("Ya no", ms - 7);
    let yesterday = add("Ayer", ms - 6);
    let open = add("Abierta", ms - 5);
    state.apply(&ev(
        ms + 2,
        "dev_here",
        Op::TaskDone {
            id: vet,
            filled: false,
        },
    ));
    state.apply(&ev(
        ms + 1,
        "dev_here",
        Op::TaskDone {
            id: walk,
            filled: false,
        },
    ));
    state.apply(&ev(
        ms + 3,
        "dev_here",
        Op::TaskDone {
            id: hidden,
            filled: false,
        },
    ));
    state.apply(&ev(ms + 4, "dev_here", Op::TaskHide { id: hidden }));
    state.apply(&ev(ms + 5, "dev_here", Op::TaskDrop { id: dropped }));
    state.apply(&ev(
        ms - 86_400_000,
        "dev_here",
        Op::TaskDone {
            id: yesterday,
            filled: false,
        },
    ));
    let closed: Vec<_> = closed_on(&state, day("2026-11-15"), &zone)
        .into_iter()
        .map(|task| task.id)
        .collect();
    assert_eq!(closed, vec![walk, vet]);
    assert!(!closed.contains(&open));
}

#[test]
fn a_locked_book_makes_no_new_month_and_is_said_locked() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let november = opened_into(room.path(), &mut state, day("2026-11-15"), &SPANISH);
    let id = state
        .docs
        .values()
        .find(|one| one.file == november.book)
        .unwrap()
        .id;
    state.apply(&ev(9_000, "dev_here", Op::DocLock { id }));
    let device = here();
    let maker = Maker {
        root: room.path(),
        device: &device,
        by: None,
    };

    let december = opened(&maker, &state, day("2026-12-01"), &SPANISH).unwrap();

    assert!(
        matches!(&december, Opened::Locked { book } if *book == november.book),
        "{december:?}"
    );
    assert_eq!(state.docs.len(), 2);
    assert_eq!(
        crate::docs::all(room.path()).len(),
        2,
        "no page was written on disk"
    );
}
