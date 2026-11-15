use super::*;
use crate::Op;
use crate::event::{DeviceId, Event};

fn ev(ms: i64, op: Op) -> Event {
    Event::new(
        DeviceId("dev_here".into()),
        jiff::Timestamp::from_millisecond(ms).unwrap(),
        op,
    )
}

const SAID: Said = Said {
    today: "Hoy",
    one: "1 tarea cerrada",
    many: "{name} tareas cerradas",
};

fn diary(state: &mut State, root: &std::path::Path) -> Kept {
    let device = DeviceId("dev_here".into());
    let maker = crate::diary::Maker {
        root,
        device: &device,
        by: None,
    };
    let date: Date = "2026-11-15".parse().unwrap();
    let crate::diary::Opened::Ready(opening) =
        crate::diary::opened(&maker, state, date, &crate::diary::SPANISH).unwrap()
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

#[test]
fn a_day_shows_under_its_title_what_was_closed_on_it_and_today_says_so() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let page = diary(&mut state, room.path());
    closed(&mut state, "Vacuna de <Roger> & Rina", "2026-11-15");
    closed(&mut state, "Pagar la luz", "2026-11-13");
    let body = "# Noviembre\n\n## Domingo 15\n\nPaseo.\n\n## Sábado 14\n\nNada.\n\n## Viernes 13\n\nTrabajo.\n";
    let html = tisty_markdown::html(&tisty_markdown::read(body));

    let dressed = dressed(&html, body, &seen(room.path(), &state, &page), &SAID);

    let sunday = dressed.find("Domingo 15</h2>").unwrap();
    let saturday = dressed.find("Sábado 14</h2>").unwrap();
    let friday = dressed.find("Viernes 13</h2>").unwrap();
    let today = dressed
        .find("<span class=\"diary-today\">Hoy</span>")
        .unwrap();
    let vet = dressed
        .find("Vacuna de &lt;Roger&gt; &amp; Rina</a>")
        .unwrap();
    let light = dressed.find("Pagar la luz</a>").unwrap();
    assert!(sunday < today && today < vet && vet < saturday, "{dressed}");
    assert!(friday < light, "{dressed}");
    assert_eq!(dressed.matches("<div class=\"diary-day\">").count(), 2);
    assert_eq!(dressed.matches("1 tarea cerrada<").count(), 2);
    assert!(dressed.contains("<a href=\"tisty:task/"));
    assert_eq!(undressed(&dressed), html, "the words were left as drawn");
}

fn undressed(html: &str) -> String {
    let mut out = html.to_string();
    while let Some(from) = out.find("<div class=\"diary-day\">") {
        let to = from + out[from..].find("</div>").unwrap() + "</div>".len();
        out.replace_range(from..to, "");
    }
    out
}

#[test]
fn a_page_outside_the_diary_is_drawn_as_it_was() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let mut page = diary(&mut state, room.path());
    closed(&mut state, "Pagar la luz", "2026-11-15");
    page.page_of = None;
    let body = "# Notas\n\n## Domingo 15\n\nPaseo.\n";
    let html = tisty_markdown::html(&tisty_markdown::read(body));

    let dressed = dressed(&html, body, &seen(room.path(), &state, &page), &SAID);

    assert_eq!(dressed, html);
    assert_eq!(print(body, &seen(room.path(), &state, &page)), "");
}

#[test]
fn the_print_changes_only_when_what_the_strips_say_does() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let page = diary(&mut state, room.path());
    let body = "# Noviembre\n\n## Domingo 15\n\nPaseo.\n\n## Sábado 14\n\nNada.\n";
    let before = print(body, &seen(room.path(), &state, &page));

    state.apply(&ev(
        2,
        Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: crate::event::TaskAdd::new("Sigue abierta", "a1"),
        },
    ));
    assert_eq!(print(body, &seen(room.path(), &state, &page)), before);

    closed(&mut state, "Pagar la luz", "2026-11-14");
    assert_ne!(print(body, &seen(room.path(), &state, &page)), before);
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
