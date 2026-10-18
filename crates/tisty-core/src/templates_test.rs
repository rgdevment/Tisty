use super::{Scope, Template, body, found, picked, told};
use crate::event::{DeviceId, DocAdd, Event, ListAdd, Op, TaskAdd};
use crate::state::State;
use ulid::Ulid;

fn applied(state: &mut State, op: Op) {
    state.apply(&Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(1).unwrap(),
        op,
    ));
}

fn template(file: &str, scope: Scope, r#for: Option<&str>) -> Template {
    Template {
        file: file.into(),
        title: Some(file.into()),
        scope,
        r#for: r#for.map(Into::into),
    }
}

#[test]
fn a_template_says_what_it_is_for_in_its_front_matter() {
    assert_eq!(
        told("---\ntemplate: Tasks\ntemplate_for: \"@Deploys\"\n---\nWords.\n"),
        Some((Scope::Tasks, Some("deploys".into())))
    );
    assert_eq!(
        told("---\ntemplate: both\n---\n"),
        Some((Scope::Both, None))
    );
    assert_eq!(
        told("---\ntemplate: docs\n---\n"),
        Some((Scope::Docs, None))
    );
    assert_eq!(told("---\ntemplate: maybe\n---\n"), None);
    assert_eq!(told("---\ntags: [a]\n---\n"), None);
    assert_eq!(told("No front matter.\n"), None);
}

#[test]
fn a_template_hands_over_its_words_without_its_front_matter() {
    assert_eq!(
        body("---\ntemplate: tasks\n---\n\n## Steps\n"),
        "## Steps\n"
    );
    assert_eq!(body("Only words.\n"), "Only words.\n");
}

#[test]
fn a_scope_takes_what_it_names() {
    assert!(Scope::Tasks.takes_tasks() && !Scope::Tasks.takes_docs());
    assert!(!Scope::Docs.takes_tasks() && Scope::Docs.takes_docs());
    assert!(Scope::Both.takes_tasks() && Scope::Both.takes_docs());
}

#[test]
fn a_task_takes_its_list_s_template_then_the_routine_s_then_every_task_s() {
    let mut state = State::default();
    let list = Ulid::generate();
    applied(
        &mut state,
        Op::ListAdd {
            id: list,
            d: ListAdd {
                name: "Deploys".into(),
                order: "a0".into(),
                color: None,
            },
        },
    );
    let all = vec![
        template("a-0001", Scope::Tasks, Some(super::EVERY_TASK)),
        template("a-0002", Scope::Tasks, Some(super::EVERY_ROUTINE)),
        template("a-0003", Scope::Tasks, Some("deploys")),
        template("a-0004", Scope::Docs, Some("home")),
    ];
    let mut task = crate::model::Task::new(Ulid::generate(), "Ship 4.2", "a0");

    assert_eq!(
        picked(&all, &state, &task).map(|one| one.file.as_str()),
        Some("a-0001")
    );

    task.after = Some(Ulid::generate());
    assert_eq!(
        picked(&all, &state, &task).map(|one| one.file.as_str()),
        Some("a-0002")
    );

    task.list = Some(list);
    assert_eq!(
        picked(&all, &state, &task).map(|one| one.file.as_str()),
        Some("a-0003")
    );

    let only_docs = vec![template("a-0004", Scope::Docs, Some(super::EVERY_TASK))];
    assert_eq!(picked(&only_docs, &state, &task), None);
}

#[test]
fn templates_are_found_on_disk_by_their_front_matter_alone() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    for (file, text) in [
        (
            "a3f1-0001",
            "---\ntemplate: tasks\ntemplate_for: tasks\n---\n\n## Steps\n",
        ),
        ("a3f1-0002", "---\ntags: [home]\n---\nA plain page.\n"),
    ] {
        std::fs::write(room.path().join(format!("{file}.md")), text).unwrap();
        applied(
            &mut state,
            Op::DocAdd {
                id: Ulid::generate(),
                d: DocAdd {
                    file: file.into(),
                    order: "a0".into(),
                    ..DocAdd::default()
                },
            },
        );
    }
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Ship 4.2", "a0"),
        },
    );
    let first = crate::task_docs::made(room.path(), &state, task, task, None).unwrap();
    applied(&mut state, first);

    let all = found(room.path(), &state);

    assert_eq!(all.len(), 1, "{all:?}");
    assert_eq!(all[0].file, "a3f1-0001");
    assert_eq!(all[0].scope, Scope::Tasks);
}
