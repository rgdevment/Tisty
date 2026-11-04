//! A task as Tisty keeps it, told in the words the contract gives it. Whoever needs tasks reads this shape.

use jiff::civil::date;
use serde_json::{Value, json};
use tisty_contracts::check;
use tisty_core::model::{
    Cadence, DateSpec, LogEntry, Priority, Repeat, Status, Step, Tag, Task, Unit,
};
use tisty_core::task_front::{day, repeat};
use ulid::Ulid;

fn told(task: &Task) -> Value {
    let mut out = json!({
        "id": task.id.to_string(),
        "title": task.title,
        "status": serde_json::to_value(task.status).unwrap(),
        "priority": task.priority.name(),
        "tags": task.tags.iter().map(|one| serde_json::to_value(one).unwrap()).collect::<Vec<_>>(),
        "steps": task.steps.iter().map(|one| json!({ "text": one.text, "done": one.done })).collect::<Vec<_>>(),
        "notes": task.log.iter().map(|one| json!({ "at": one.at.to_string(), "body": one.body })).collect::<Vec<_>>(),
        "reminders": task.reminders.iter().map(|one| json!({ "at": day(one) })).collect::<Vec<_>>(),
    });
    if let Some(one) = &task.date {
        out["date"] = json!(day(one));
    }
    if let Some(one) = &task.deadline {
        out["deadline"] = json!(day(one));
    }
    if let Some(one) = &task.list {
        out["list"] = json!(one.to_string());
    }
    if let Some(one) = &task.source {
        out["source"] = json!(one);
    }
    if let Some(one) = &task.description {
        out["description"] = json!(one);
    }
    if let Some(one) = task.repeat {
        out["repeat"] = json!(repeat(one));
    }
    if let Some(one) = &task.part_of {
        out["part_of"] = json!(one.to_string());
    }
    if let Some(one) = &task.after {
        out["after"] = json!(one.to_string());
    }
    out
}

fn a_full_task() -> Task {
    let at = date(2026, 10, 2).at(9, 0, 0, 0);
    let mut task = Task::new(Ulid::from_parts(1, 1), "Vacunar a Rina", "a");
    task.priority = Priority::Do;
    task.tags = vec![serde_json::from_value::<Tag>(json!("rina")).unwrap()];
    task.date = Some(DateSpec::all_day(date(2026, 9, 28), "America/Santiago"));
    task.deadline = Some(DateSpec::floating(at, "America/Santiago"));
    task.reminders = vec![DateSpec::fixed(at, "Europe/Madrid")];
    task.source = Some("sereno#1".to_owned());
    task.description = Some("La de la rabia".to_owned());
    task.repeat = Some(Repeat::due(Cadence {
        every: 1,
        unit: Unit::Year,
    }));
    task.part_of = Some(Ulid::from_parts(2, 2));
    task.after = Some(Ulid::from_parts(3, 3));
    task.steps = vec![Step {
        id: Ulid::from_parts(1, 1),
        text: "Pedir hora".to_owned(),
        done: false,
        order: "a".to_owned(),
        by_agent: false,
        said: false,
    }];
    // A moment with its fraction of a second, as the log keeps every note.
    task.log = vec![LogEntry {
        id: Ulid::from_parts(1, 1),
        at: "2026-11-04T03:05:01.1571702Z".parse().unwrap(),
        tz: None,
        body: "Llamé".to_owned(),
        by: None,
        via: None,
    }];
    task
}

#[test]
fn a_task_with_all_it_can_hold_speaks_the_contract() {
    assert_eq!(check("task", &told(&a_full_task())), Vec::<String>::new());
}

#[test]
fn every_status_and_priority_tisty_keeps_is_one_the_contract_knows() {
    let mut task = a_full_task();
    for status in [Status::Open, Status::Done, Status::Dropped] {
        task.status = status;
        for priority in [
            Priority::Do,
            Priority::Decide,
            Priority::Delegate,
            Priority::Unset,
            Priority::Minor,
        ] {
            task.priority = priority;
            assert_eq!(
                check("task", &told(&task)),
                Vec::<String>::new(),
                "{status:?} {priority:?}"
            );
        }
    }
}

#[test]
fn every_tag_tisty_makes_is_one_the_contract_knows() {
    let cases: Value = serde_json::from_str(include_str!("../../../fixtures/tags.json")).unwrap();
    for one in cases["named"].as_array().unwrap() {
        let said = one[0].as_str().unwrap();
        let tag = Tag::new(said).unwrap();
        assert_eq!(
            check("tag", &json!({ "name": tag.as_str() })),
            Vec::<String>::new(),
            "{said}"
        );
    }
}

#[test]
fn a_task_with_the_least_it_holds_speaks_it_too() {
    let task = Task::new(Ulid::from_parts(1, 1), "x", "a");
    assert_eq!(check("task", &told(&task)), Vec::<String>::new());
}
