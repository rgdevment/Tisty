use super::*;
use crate::Op;
use crate::diary::SPANISH;
use crate::event::{DeviceId, Event};

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_here".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

const SAID: Said = Said {
    one: "1 tarea cerrada",
    many: "{name} tareas cerradas",
    empty: "Lo que escribas hoy aparece aquí",
    writing: "Las palabras de este día",
    keep: "Guardar",
    cancel: "Cancelar",
};

fn diary(state: &mut State, root: &std::path::Path) -> Kept {
    let device = DeviceId("dev_here".into());
    let maker = crate::diary::Maker {
        root,
        device: &device,
        by: None,
        writer: &crate::history::Writer::now(root, None),
    };
    let date: Date = "2026-11-15".parse().unwrap();
    let crate::diary::Opened::Ready(opening) =
        crate::diary::opened(&maker, state, date, &SPANISH).unwrap()
    else {
        panic!("the diary was locked");
    };
    for (n, op) in opening.ops.into_iter().enumerate() {
        state.apply(&ev(1_000 + n as i64, op));
    }
    state
        .docs
        .values()
        .find(|one| one.file == opening.page)
        .unwrap()
        .clone()
}

fn closed(state: &mut State, title: &str, on: &str) {
    let id = ulid::Ulid::generate();
    let noon: Date = on.parse().unwrap();
    let ms = noon
        .at(12, 0, 0, 0)
        .to_zoned(jiff::tz::TimeZone::UTC)
        .unwrap()
        .timestamp()
        .as_millisecond();
    state.apply(&ev(
        ms - 1,
        Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new(title, "a0"),
        },
    ));
    state.apply(&ev(ms, Op::TaskDone { id, filled: false }));
}

fn seen<'a>(root: &'a std::path::Path, state: &'a State, kept: &'a Kept) -> Page<'a> {
    Page {
        root,
        state,
        kept,
        today: "2026-11-15".parse().unwrap(),
        zone: &UTC,
    }
}

static UTC: jiff::tz::TimeZone = jiff::tz::TimeZone::UTC;

fn shown(body: &str, root: &std::path::Path, state: &State, kept: &Kept) -> String {
    shown_open(body, root, state, kept, None)
}

fn shown_open(
    body: &str,
    root: &std::path::Path,
    state: &State,
    kept: &Kept,
    open: Option<DayAt>,
) -> String {
    let days = days(body);
    laid(
        &days,
        &strips(&days, &seen(root, state, kept)),
        &SPANISH,
        &SAID,
        open,
    )
}

const MONTH: &str = "# Noviembre\n\nUn mes tranquilo.\n\n## Domingo 15\n\nPaseo con Rina.\n\n### Tarde\n\nLlamé a Jaime.\n\n## Sábado 14\n\nCena con Marisol.\n";

#[test]
fn a_month_is_drawn_as_its_days_newest_on_top_each_with_its_number_and_weekday() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let page = diary(&mut state, room.path());

    let laid = shown(MONTH, room.path(), &state, &page);

    assert!(
        !laid.contains("Noviembre"),
        "the month is the view's, not a title: {laid}"
    );
    let intro = laid.find("Un mes tranquilo.").unwrap();
    let sunday = laid.find("diary-entry diary-is-today").unwrap();
    let walk = laid.find("Paseo con Rina.").unwrap();
    let call = laid.find("Llamé a Jaime.").unwrap();
    let saturday = laid.find("<span class=\"diary-number\">14</span>").unwrap();
    let dinner = laid.find("Cena con Marisol.").unwrap();
    assert!(intro < sunday && sunday < walk && walk < call && call < saturday && saturday < dinner);
    assert!(laid.contains("<span class=\"diary-weekday\">dom</span>"));
    assert!(laid.contains("<span class=\"diary-weekday\">sáb</span>"));
    assert_eq!(laid.matches("diary-is-today").count(), 1);
    assert!(!laid.contains("## "), "{laid}");
}

#[test]
fn a_day_folds_what_was_closed_on_it_and_a_title_is_escaped() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let page = diary(&mut state, room.path());
    closed(&mut state, "Vacuna de <Roger> & Rina", "2026-11-15");
    closed(&mut state, "Pagar la luz", "2026-11-14");
    closed(&mut state, "Regar", "2026-11-14");

    let laid = shown(MONTH, room.path(), &state, &page);

    let vet = laid.find("Vacuna de &lt;Roger&gt; &amp; Rina</a>").unwrap();
    let saturday = laid.find("<span class=\"diary-number\">14</span>").unwrap();
    assert!(vet < saturday, "{laid}");
    assert!(laid.contains("<summary>1 tarea cerrada</summary>"));
    assert!(laid.contains("<summary>2 tareas cerradas</summary>"));
    assert!(laid.contains("<a href=\"tisty:task/"));
}

#[test]
fn an_empty_month_says_where_today_goes_and_a_page_outside_the_diary_shows_no_log() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let mut page = diary(&mut state, room.path());
    closed(&mut state, "Pagar la luz", "2026-11-15");

    let empty = shown("# Noviembre\n", room.path(), &state, &page);
    assert!(empty.contains("<p class=\"diary-empty\">Lo que escribas hoy aparece aquí</p>"));

    page.page_of = None;
    let loose = shown(MONTH, room.path(), &state, &page);
    assert!(!loose.contains("Pagar la luz") && !loose.contains("diary-is-today"));
    assert_eq!(
        print(&strips(&days(MONTH), &seen(room.path(), &state, &page))),
        ""
    );
}

#[test]
fn the_print_changes_only_when_what_the_days_show_of_the_log_does() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let page = diary(&mut state, room.path());
    let before = print(&strips(&days(MONTH), &seen(room.path(), &state, &page)));

    state.apply(&ev(
        2,
        Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: crate::event::TaskAdd::new("Sigue abierta", "a1"),
        },
    ));
    assert_eq!(
        print(&strips(&days(MONTH), &seen(room.path(), &state, &page))),
        before
    );

    closed(&mut state, "Pagar la luz", "2026-11-14");
    assert_ne!(
        print(&strips(&days(MONTH), &seen(room.path(), &state, &page))),
        before
    );
}

#[test]
fn each_day_says_which_it_is_so_a_touch_names_it_even_when_two_share_a_number() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let page = diary(&mut state, room.path());

    let laid = shown(
        "# Noviembre\n\n## Domingo 15\n\nUno.\n\n## Sábado 14\n\nDos.\n\n## Domingo 15\n\nTres.\n",
        room.path(),
        &state,
        &page,
    );

    let first = laid.find("data-day=\"15\" data-nth=\"0\"").unwrap();
    let saturday = laid.find("data-day=\"14\" data-nth=\"0\"").unwrap();
    let again = laid.find("data-day=\"15\" data-nth=\"1\"").unwrap();
    assert!(first < saturday && saturday < again, "{laid}");
    assert!(!laid.contains("textarea"), "{laid}");
}

#[test]
fn the_day_opened_is_a_box_with_its_words_its_notes_still_drawn_as_notes() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let page = diary(&mut state, room.path());
    closed(&mut state, "Pagar la luz", "2026-11-15");
    let body = "# Noviembre\n\n## Domingo 15\n\nPaseo con <Rina> & Roger.\n\n> [!IDEA] 2026-11-15 10:05 · yellow · right\n> Comprar pan\n\nDe vuelta.\n\n## Sábado 14\n\nCena.\n";

    let laid = shown_open(
        body,
        room.path(),
        &state,
        &page,
        Some(DayAt { day: 15, nth: 0 }),
    );

    let box_at = laid
        .find("<div class=\"diary-box editing\"><textarea class=\"diary-box-words\" rows=\"3\" aria-label=\"Las palabras de este día\">\nPaseo con &lt;Rina&gt; &amp; Roger.\n\nDe vuelta.</textarea></div>")
        .unwrap_or_else(|| panic!("{laid}"));
    let bar = laid
        .find("data-asked=\"dayCancel\">Cancelar</button>")
        .unwrap();
    let keep = laid
        .find("data-asked=\"dayKeep\">Guardar</button>")
        .unwrap();
    let note = laid.find("data-callout=\"idea\"").unwrap();
    let strip = laid.find("<summary>1 tarea cerrada</summary>").unwrap();
    let saturday = laid.find("data-day=\"14\"").unwrap();
    assert!(box_at < bar && bar < keep && keep < note && note < strip && strip < saturday);
    assert_eq!(laid.matches("textarea").count(), 2);
    assert!(laid.contains("<p data-at=\"0-5\">Cena.</p>"), "{laid}");
}
