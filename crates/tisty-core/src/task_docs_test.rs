use super::{file_at, id_at, made, of, seed, task_in, task_of_file};
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

    assert_eq!(file_at(task, 1), "t_01m563t6t541a3q2pp5yqw5xk1-0001");
    assert!(crate::docs::resolve(std::path::Path::new("docs"), &file_at(task, 1)).is_ok());
    assert_eq!(task_of_file(&file_at(task, 2)), Some(task));
    assert_eq!(task_of_file("dev_a-0001"), None);
    assert_eq!(id_at(task, 1), id_at(task, 1));
    assert_ne!(id_at(task, 1), id_at(task, 2));
    assert_ne!(
        id_at(task, 1),
        task,
        "the document would answer to the task's permissions"
    );
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
        room.path().join(format!("{}.md", file_at(task, 1))),
        format!("{}\nWords already written.\n", seed(task)),
    )
    .unwrap();
    let again = made(room.path(), &state, task, None).unwrap();

    let body = crate::docs::read(room.path(), &file_at(task, 1)).unwrap();
    assert!(body.contains("Words already written."), "{body}");
    applied(&mut state, first);
    applied(&mut state, again);
    let kept = of(&state, task).expect("the task's document");
    assert_eq!(kept.task_of, Some(task));
    assert_eq!(kept.file, file_at(task, 1));
    assert_eq!(state.docs.len(), 1);
}

#[test]
fn a_document_made_again_after_its_delete_takes_the_next_turn_and_belongs_to_the_task() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    let first = made(room.path(), &state, task, None).unwrap();
    applied(&mut state, first);
    let gone = of(&state, task).unwrap().id;
    applied(&mut state, Op::DocDelete { id: gone });
    std::fs::remove_file(room.path().join(format!("{}.md", file_at(task, 1)))).unwrap();

    let again = made(room.path(), &state, task, None).unwrap();
    applied(&mut state, again);

    let kept = of(&state, task).expect("the task's document again");
    assert_eq!(kept.file, file_at(task, 2));
    assert_eq!(kept.id, id_at(task, 2));
}

#[test]
fn a_second_making_keeps_what_was_built_on_the_first() {
    let room = tempfile::tempdir().unwrap();
    let mut state = State::default();
    let task = Ulid::generate();
    let first = made(room.path(), &state, task, None).unwrap();
    let elsewhere = first.clone();
    applied(&mut state, first);
    let id = of(&state, task).unwrap().id;
    applied(&mut state, Op::DocArchive { id });

    applied(&mut state, elsewhere);

    assert!(
        of(&state, task).unwrap().archived,
        "the second making undid the first"
    );
}
