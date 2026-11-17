use super::*;
use tisty_core::Op;
use tisty_core::event::{ListAdd, TaskAdd};

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    (tmp, session)
}

fn list(session: &mut Session, name: &str) -> String {
    let id = ulid::Ulid::generate();
    session
        .commit(Op::ListAdd {
            id,
            d: ListAdd {
                name: name.into(),
                order: "a0".into(),
                color: None,
            },
        })
        .unwrap();
    id.to_string()
}

fn nudge(list: &str) -> Nudge {
    Nudge {
        title: "Revisión semanal".into(),
        tag: "revisión-semanal".into(),
        list: list.into(),
        weekday: 5,
        hour: 17,
        minute: 0,
    }
}

fn monday() -> jiff::civil::DateTime {
    jiff::civil::date(2026, 11, 16).at(20, 0, 0, 0)
}

#[test]
fn reminding_every_week_files_one_routine_in_the_list_chosen_and_never_a_second() {
    let (_tmp, mut session) = session();
    let home = list(&mut session, "Personal");

    let made = reminded(&mut session, &nudge(&home), monday()).unwrap();
    let again = reminded(&mut session, &nudge(&home), monday()).unwrap();

    assert_eq!(made.id, again.id, "asking twice keeps the one there is");
    assert_eq!(made.list.map(|one| one.to_string()), Some(home));
    assert_eq!(made.reminders.len(), 1);
    assert_eq!(
        made.reminders[0].at,
        jiff::civil::date(2026, 11, 20).at(17, 0, 0, 0)
    );
    assert_eq!(reviewing(&session).routine.map(|one| one.id), Some(made.id));
}

#[test]
fn what_waits_unlisted_leaves_the_review_out() {
    let (_tmp, mut session) = session();
    let home = list(&mut session, "Personal");
    reminded(&mut session, &nudge(&home), monday()).unwrap();
    session
        .commit(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("Llamar a Yisset", "a0"),
        })
        .unwrap();

    let seen = reviewing(&session);

    let titles: Vec<&str> = seen.unlisted.iter().map(|one| one.title.as_str()).collect();
    assert_eq!(titles, vec!["Llamar a Yisset"]);
}

#[test]
fn a_day_or_an_hour_that_is_none_is_refused() {
    let (_tmp, mut session) = session();
    let home = list(&mut session, "Personal");

    let bad_day = Nudge {
        weekday: 9,
        ..nudge(&home)
    };
    let bad_hour = Nudge {
        hour: 25,
        ..nudge(&home)
    };

    assert!(reminded(&mut session, &bad_day, monday()).is_err());
    assert!(reminded(&mut session, &bad_hour, monday()).is_err());
    assert!(reminded(&mut session, &nudge("not-a-list"), monday()).is_err());
    assert!(reviewing(&session).routine.is_none());
}
