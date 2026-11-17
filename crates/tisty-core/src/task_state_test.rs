use jiff::Timestamp;
use ulid::Ulid;

use super::{Since, heading_in, named, paged, said, since, stated};
use crate::event::{PageChange, Paged};
use crate::model::{LogEntry, Task};

const PAGE: &str = "---\ntask: x\n---\n\n## Descripción\n\nMover pagos.\n\n## Pasos\n\n- [ ] Firmar\n\n## Notas\n<!-- Lo que vayas averiguando mientras la haces. -->\n\n## Al cerrarla\n";

#[test]
fn a_first_state_goes_between_the_steps_and_the_notes() {
    let page = stated(PAGE, "**hoy** · Claude\nFalta subir.", true);
    assert!(
        page.contains("- [ ] Firmar\n\n## Estado\n\n**hoy** · Claude\nFalta subir.\n\n## Notas\n"),
        "{page}"
    );
    assert_eq!(said(&page), Some("**hoy** · Claude\nFalta subir."));
}

#[test]
fn a_new_state_replaces_the_last_whole_and_leaves_the_rest_as_it_was() {
    let once = stated(PAGE, "Uno.\nDos.", true);
    let twice = stated(&once, "Tres.", true);
    assert_eq!(
        twice,
        PAGE.replace("## Notas", "## Estado\n\nTres.\n\n## Notas")
    );
}

#[test]
fn without_notes_it_goes_before_the_assistants_or_at_the_end() {
    let page = stated("## Steps\n\n## Assistant notes\n\nOld.\n", "Now.", false);
    assert_eq!(
        page,
        "## Steps\n\n## State\n\nNow.\n\n## Assistant notes\n\nOld.\n"
    );
    assert_eq!(
        stated("## Steps\n", "Now.", false),
        "## Steps\n\n## State\n\nNow.\n"
    );
}

#[test]
fn a_box_or_a_heading_in_a_state_never_becomes_a_step_or_a_section() {
    let page = stated(PAGE, "## Hecho\n- [x] Firmar\n```\nabierto", true);
    let state = said(&page).unwrap();
    assert!(
        state.starts_with("### Hecho\n- \\[x] Firmar\n```\nabierto\n```"),
        "{state}"
    );
    assert!(page.contains("## Notas"), "{page}");
}

#[test]
fn a_page_in_windows_lines_keeps_them() {
    let page = stated("## Steps\r\n\r\n## Notes\r\n", "One.\r\nTwo.", false);
    assert_eq!(
        page,
        "## Steps\r\n\r\n## State\r\n\r\nOne.\r\nTwo.\r\n\r\n## Notes\r\n"
    );
}

#[test]
fn both_languages_name_the_section() {
    assert!(named("Estado") && named("State"));
    assert!(!named("Estados") && !named("Notas"));
    assert_eq!(heading_in(&stated(PAGE, "Uno.", true)), Some("Estado"));
    assert_eq!(heading_in("## Estados\n\nUno.\n"), None);
    assert_eq!(heading_in("## State\n\nOne.\n"), Some("State"));
}

fn at(second: i64) -> Timestamp {
    Timestamp::from_second(second).unwrap()
}

fn trail(section: &str, change: PageChange) -> Paged {
    Paged {
        section: section.into(),
        change,
    }
}

#[test]
fn the_trail_says_when_the_state_was_written_and_when_it_was_taken_off() {
    let mut task = Task::new(Ulid::generate(), "x", "a0");
    paged(&mut task, &trail("Notas", PageChange::Written), at(5));
    assert_eq!(task.stated_at, None);
    paged(&mut task, &trail("Estado", PageChange::Added), at(10));
    assert_eq!(task.stated_at, Some(at(10)));
    paged(&mut task, &trail("Estado", PageChange::Written), at(20));
    assert_eq!(task.stated_at, Some(at(20)));
    let away = PageChange::Renamed {
        from: "Estado".into(),
    };
    paged(&mut task, &trail("Resumen", away), at(30));
    assert_eq!(task.stated_at, None);
    let back = PageChange::Renamed {
        from: "Resumen".into(),
    };
    paged(&mut task, &trail("State", back), at(40));
    assert_eq!(task.stated_at, Some(at(40)));
    paged(&mut task, &trail("State", PageChange::Removed), at(50));
    assert_eq!(task.stated_at, None);
}

fn note(second: i64) -> LogEntry {
    LogEntry {
        id: Ulid::generate(),
        at: at(second),
        tz: None,
        body: "decidido".into(),
        by: None,
        via: None,
        replaces: None,
    }
}

#[test]
fn a_state_grows_old_with_the_steps_that_move_and_the_notes_that_come_after_it() {
    let mut task = Task::new(Ulid::generate(), "x", "a0");
    assert_eq!(since(&task), None);
    task.stated_at = Some(at(10));
    task.steps_moved_at = Some(at(10));
    task.log = vec![note(5)];
    assert_eq!(since(&task), Some(Since::default()));
    assert!(!since(&task).unwrap().old());

    task.steps_moved_at = Some(at(11));
    task.log.push(note(12));
    let later = since(&task).unwrap();
    assert_eq!(
        later,
        Since {
            steps: true,
            notes: 1
        }
    );
    assert!(later.old());
}
