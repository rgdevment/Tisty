use super::{file_of, id_of, made, of, seed, task_in};
use crate::event::{DeviceId, Event, Op, TaskAdd};
use crate::state::State;
use ulid::Ulid;

fn applied(state: &mut State, op: Op) {
    state.apply(&Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_millisecond(1).unwrap(),
        op,
    ));
}

#[test]
fn a_task_names_its_document_the_same_way_everywhere_and_never_by_its_own_id() {
    let task = Ulid::from_string("01M563T6T541A3Q2PP5YQW5XK1").unwrap();

    assert_eq!(file_of(task), "t_01m563t6t541a3q2pp5yqw5xk1-0001");
    assert!(crate::docs::resolve(std::path::Path::new("docs"), &file_of(task)).is_ok());
    assert_eq!(id_of(task), id_of(task));
    assert_ne!(
        id_of(task),
        task,
        "the document would answer to the task's permissions"
    );
    assert_eq!(id_of(task).timestamp_ms(), task.timestamp_ms());
}

#[test]
fn the_seed_names_its_task_and_reads_back() {
    let task = Ulid::generate();
    assert_eq!(task_in(&seed(task)), Some(task));
    assert_eq!(task_in("# Notes\n\nNo properties here.\n"), None);
    assert_eq!(task_in("---\ntask: not-an-id\n---\n"), None);
}

#[test]
fn a_document_made_twice_for_one_task_is_one_file_that_belongs_to_it() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    applied(
        &mut state,
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("Rina's booster", "a0"),
        },
    );

    let first = made(room.path(), &state, task, None).unwrap();
    std::fs::write(
        room.path().join(format!("{}.md", file_of(task))),
        format!("{}\nWords already written.\n", seed(task)),
    )
    .unwrap();
    let again = made(room.path(), &state, task, None).unwrap();

    let body = crate::docs::read(room.path(), &file_of(task)).unwrap();
    assert!(body.contains("Words already written."), "{body}");
    applied(&mut state, first);
    applied(&mut state, again);
    let kept = of(&state, task).expect("the task's document");
    assert_eq!(kept.task_of, Some(task));
    assert_eq!(kept.file, file_of(task));
    assert_eq!(state.docs.len(), 1);
}
