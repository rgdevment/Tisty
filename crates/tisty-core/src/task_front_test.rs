use super::{Front, day, day_in, dressed, moved, patch, read, repeat, repeat_in};
use crate::model::{Cadence, DateSpec, From, Priority, Repeat, Tag, Task, Unit};
use jiff::civil::date;
use ulid::Ulid;

const TZ: &str = "America/Santiago";

#[test]
fn a_repeat_reads_back_exactly_as_it_was_written() {
    let all = [
        Repeat::due(Cadence {
            every: 1,
            unit: Unit::Week,
        }),
        Repeat {
            from: From::Done,
            each: Cadence {
                every: 2,
                unit: Unit::Month,
            },
            until: Some(date(2026, 12, 31)),
        },
        Repeat::done(Cadence {
            every: 3,
            unit: Unit::Day,
        }),
    ];
    for one in all {
        assert_eq!(repeat_in(&repeat(one)), Some(one), "{}", repeat(one));
    }
    assert_eq!(repeat(all[0]), "every week from due");
    assert_eq!(repeat(all[1]), "every 2 months from done until 2026-12-31");
}

#[test]
fn only_the_canonical_repeat_is_read() {
    for loose in [
        "every tuesday",
        "weekly",
        "every 0 days",
        "every 2 weeks from now",
        "",
    ] {
        assert_eq!(repeat_in(loose), None, "{loose}");
    }
}

#[test]
fn a_day_keeps_whether_it_has_an_hour_and_whether_it_is_pinned_to_a_zone() {
    let all = [
        DateSpec::all_day(date(2026, 10, 3), TZ),
        DateSpec::floating(date(2026, 10, 2).at(9, 0, 0, 0), TZ),
        DateSpec::fixed(date(2026, 10, 2).at(9, 30, 0, 0), "Europe/Madrid"),
    ];
    for one in &all {
        assert_eq!(day_in(&day(one), TZ), Some(one.clone()), "{}", day(one));
    }
    assert_eq!(day(&all[2]), "2026-10-02T09:30[Europe/Madrid]");
    assert_eq!(day_in("tomorrow", TZ), None);
}

fn booster() -> Task {
    let mut task = Task::new(Ulid::generate(), "Rina's booster", "a0");
    task.date = Some(DateSpec::all_day(date(2026, 9, 28), TZ));
    task.deadline = Some(DateSpec::all_day(date(2026, 10, 3), TZ));
    task.reminders = vec![DateSpec::floating(date(2026, 10, 2).at(9, 0, 0, 0), TZ)];
    task.priority = Priority::Do;
    task.repeat = Some(Repeat::due(Cadence {
        every: 1,
        unit: Unit::Year,
    }));
    task.tags = vec![Tag::new("salud").unwrap()];
    task
}

#[test]
fn a_document_leaving_tisty_carries_its_task_and_keeps_its_own_properties() {
    let task = booster();
    let body = format!(
        "---\ntask: {}\ndue: 2020-01-01\npaper: letter\ntags:\n  - old\n---\n\nWords.\n",
        task.id
    );

    let out = dressed(&body, &task, Some("casa: home"));

    assert!(out.contains("paper: letter"), "{out}");
    assert!(!out.contains("2020-01-01"), "{out}");
    assert!(!out.contains("old"), "{out}");
    assert!(out.contains("list: \"casa: home\""), "{out}");
    assert!(out.ends_with("---\n\nWords.\n"), "{out}");
    assert_eq!(
        read(&out, TZ),
        Front {
            task: Some(task.id),
            scheduled: task.date.clone(),
            due: task.deadline.clone(),
            remind: task.reminders.clone(),
            list: Some("casa: home".into()),
            priority: Some(Priority::Do),
            repeat: task.repeat,
            tags: vec!["salud".into()],
        }
    );
}

#[test]
fn a_document_with_no_properties_gets_its_task_s_block_on_top() {
    let task = Task::new(Ulid::generate(), "Plain", "a0");
    let out = dressed("# Plain\n\nWords.\n", &task, None);
    assert_eq!(
        out,
        format!("---\ntask: {}\n---\n# Plain\n\nWords.\n", task.id)
    );
}

#[test]
fn a_document_coming_in_changes_only_what_it_says_differently() {
    let task = booster();
    let same = read(&dressed("", &task, None), TZ);
    assert_eq!(patch(&task, &same), None);

    let said = Front {
        due: Some(DateSpec::all_day(date(2026, 10, 10), TZ)),
        priority: Some(Priority::Minor),
        ..Front::default()
    };
    let changed = patch(&task, &said).expect("a change");
    assert_eq!(changed.deadline, Some(said.due.clone()));
    assert_eq!(changed.priority, Some(Priority::Minor));
    assert_eq!(changed.date, None, "a day left out was cleared");
    assert_eq!(changed.repeat, None);
    assert_eq!(changed.tags, None);
}

#[test]
fn a_list_is_found_by_its_name_and_an_unknown_one_makes_nothing() {
    let mut state = crate::State::default();
    let home = Ulid::generate();
    state
        .lists
        .insert(home, crate::model::List::new(home, "Casa", "a0"));
    let task = booster();
    let asked = |name: &str| Front {
        list: Some(name.into()),
        ..Front::default()
    };

    let into = moved(&state, &task, &asked("@casa")).expect("the list");
    assert_eq!(into.list, Some(Some(home)));
    assert_eq!(moved(&state, &task, &asked("nowhere")), None);
}
