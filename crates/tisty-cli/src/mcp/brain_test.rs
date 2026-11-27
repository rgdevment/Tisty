use super::project::{Stage, last_moved, stage};
use super::ranking::{newest_line, newest_paper_line};
use super::*;
use tisty_core::event::{DeviceId, DocAdd, Event, Op, TaskAdd};
use tisty_core::model::{LogEntry, Resolved, Status, Step, Task};
use ulid::Ulid;

fn at(second: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_second(1_800_000_000 + second).unwrap()
}

fn noted(second: i64, body: &str) -> LogEntry {
    LogEntry {
        id: Ulid::generate(),
        at: at(second),
        tz: None,
        body: body.into(),
        by: None,
        via: None,
        replaces: None,
    }
}

fn step(done: bool) -> Step {
    Step {
        id: Ulid::generate(),
        text: "a step".into(),
        done,
        order: "a0".into(),
        by_agent: false,
        said: false,
    }
}

fn home() -> (tempfile::TempDir, Paths) {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    (room, paths)
}

fn added(state: &mut State, op: Op) {
    state.apply(&Event::new(DeviceId("dev_person".into()), at(0), op));
}

fn with_a_task(state: &mut State, title: &str) -> TaskId {
    let id = Ulid::generate();
    added(
        state,
        Op::TaskAdd {
            id,
            d: TaskAdd::new(title.to_string(), "a0"),
        },
    );
    id
}

fn with_a_doc(state: &mut State, file: &str, task_of: Option<TaskId>) -> Ulid {
    let id = task_of.map_or_else(Ulid::generate, |task| tisty_core::task_docs::id_at(task, 1));
    added(
        state,
        Op::DocAdd {
            id,
            d: DocAdd {
                file: file.into(),
                order: "a0".into(),
                task_of,
                ..Default::default()
            },
        },
    );
    id
}

#[test]
fn the_gate_lets_through_only_what_find_and_read_would_show() {
    let (_room, paths) = home();
    let mut state = State::default();
    let seen = with_a_task(&mut state, "Seen");
    let hidden = with_a_task(&mut state, "Hidden");
    let dropped = with_a_task(&mut state, "Dropped");
    let hidden_page = tisty_core::task_docs::file_at(hidden, 1);
    with_a_doc(&mut state, &hidden_page, Some(hidden));
    let doc = with_a_doc(&mut state, "dev_person-0001", None);
    if let Some(one) = state.tasks.get_mut(&hidden) {
        one.hidden = true;
    }
    if let Some(one) = state.tasks.get_mut(&dropped) {
        one.status = Status::Dropped;
    }
    let gate = Gate::new(&paths, &state);

    assert!(gate.sees(&seen.to_string()));
    assert!(!gate.sees(&hidden.to_string()), "a hidden task");
    assert!(!gate.sees(&dropped.to_string()), "a dropped task");
    assert!(!gate.sees(&Ulid::generate().to_string()), "an unknown task");
    assert!(gate.sees("dev_person-0001"), "a document in sight");
    assert!(gate.sees(&doc.to_string()), "a document by its ulid");
    assert!(!gate.sees(&hidden_page), "the page of a hidden task");
    assert!(
        !gate.sees("dev_person-0099"),
        "a document the state does not hold, as the shut diary"
    );
    assert!(gate.sees("sereno"), "a chat a source names");
    assert!(gate.sees("#casa"), "a tag");
}

#[test]
fn each_tool_brain_took_over_names_its_way_and_no_other_does() {
    for name in ["find", "docs", "outline_doc", "lists", "tags"] {
        let Some(Refused::Tool(said)) = moved(name) else {
            panic!("{name} has no notice");
        };
        assert!(
            said.starts_with(&format!("`{name}` moved into `brain`")),
            "{said}"
        );
        assert!(said.ends_with("Nothing was done."), "{said}");
    }
    assert!(moved("brain").is_none());
    assert!(moved("read").is_none());
}

#[test]
fn read_doc_names_read_as_its_way() {
    let Some(Refused::Tool(said)) = moved("read_doc") else {
        panic!("read_doc has no notice");
    };
    assert!(said.starts_with("`read_doc` moved into `read`: call `read` with the same `doc`"));
    assert!(said.ends_with("Nothing was done."), "{said}");
}

#[test]
fn a_part_is_staged_by_what_it_says_and_what_it_did() {
    let mut task = Task::new(Ulid::generate(), "A part", "a0");
    assert_eq!(stage(&task, None), Stage::NotStarted);
    assert_eq!(stage(&task, Some("Started the parser.")), Stage::UnderWay);
    task.steps = vec![step(true), step(false)];
    task.volume.steps = 2;
    task.volume.steps_done = 1;
    assert_eq!(stage(&task, None), Stage::UnderWay);
    task.resolved = Some(Resolved {
        at: at(1),
        by: DeviceId("dev_agent".into()),
        entry: Ulid::generate(),
        via: None,
        branch: None,
        pr: None,
        drop: false,
    });
    assert_eq!(stage(&task, None), Stage::Waiting);
    for blocked in [
        "Blocked: waiting for the key.",
        "**Bloqueada:** falta la clave.",
        "- blocked: on review",
        "> BLOQUEADA: sin acceso",
    ] {
        assert_eq!(stage(&task, Some(blocked)), Stage::Blocked, "{blocked}");
    }
    assert_eq!(stage(&task, Some("Not blocked: it moves.")), Stage::Waiting);
}

#[test]
fn a_task_last_moved_when_its_newest_trace_says() {
    let mut task = Task::new(Ulid::generate(), "Traced", "a0");
    let made = last_moved(&task);
    task.log = vec![noted(10, "one"), noted(30, "two")];
    assert_eq!(last_moved(&task), at(30).max(made));
    task.stated_at = Some(at(50));
    assert_eq!(last_moved(&task), at(50).max(made));
    task.completed_at = Some(at(70));
    task.steps_moved_at = Some(at(60));
    assert_eq!(last_moved(&task), at(70).max(made));
}

#[test]
fn the_line_quoted_is_the_newest_holding_every_word_else_the_most() {
    let terms: Vec<String> = vec!["cache".into(), "disk".into()];
    let mut task = Task::new(Ulid::generate(), "Settle it", "a0");
    let old = noted(10, "The cache goes to disk.");
    let mut newer = noted(20, "The cache stays on disk after all.");
    newer.replaces = Some(old.id);
    let partial = noted(30, "The cache is warm now.");
    task.log = vec![old, newer, partial];

    assert_eq!(
        newest_line(&task, &terms),
        Some(("journal", "The cache stays on disk after all.".to_string()))
    );

    task.log = vec![noted(10, "Only the cache here."), noted(20, "Nothing.")];
    task.description = Some("No words.".into());
    assert_eq!(
        newest_line(&task, &terms),
        Some(("journal", "Only the cache here.".to_string()))
    );

    task.title = "cache on disk".into();
    assert_eq!(newest_line(&task, &terms), None, "the title says it all");
}

#[test]
fn a_documents_newest_line_is_its_last_standing_one_saying_the_most() {
    let (_room, paths) = home();
    let body = "# Cache\n\nThe cache sits on disk.\n\n~~The cache sits on disk, said first.~~\n\nOnly the cache.\n\nThe cache moved to disk again.\n";
    tisty_core::docs::write(&paths.docs(), "dev_person-0001", body).unwrap();
    let terms: Vec<String> = vec!["cache".into(), "disk".into()];

    assert_eq!(
        newest_paper_line(&paths, &State::default(), "dev_person-0001", &terms, None).as_deref(),
        Some("The cache moved to disk again.")
    );

    let struck = "# Cache\n\nThe cache sits on disk.\n\n~~The cache left the disk.~~\n";
    tisty_core::docs::write(&paths.docs(), "dev_person-0002", struck).unwrap();
    assert_eq!(
        newest_paper_line(&paths, &State::default(), "dev_person-0002", &terms, None).as_deref(),
        Some("The cache sits on disk.")
    );
    assert_eq!(
        newest_paper_line(&paths, &State::default(), "dev_person-0404", &terms, None),
        None
    );
}

#[test]
fn a_documents_line_is_prose_never_its_front_matter_headings_or_code() {
    let (_room, paths) = home();
    let body = "---\ntags: sync cache\n---\n# Sync cache\n\n## Sync cache notes\n\n```\nsync cache example\n```\n\nThe sync keeps a cache.\n";
    tisty_core::docs::write(&paths.docs(), "dev_person-0003", body).unwrap();
    let terms: Vec<String> = vec!["sync".into(), "cache".into()];

    assert_eq!(
        newest_paper_line(&paths, &State::default(), "dev_person-0003", &terms, None).as_deref(),
        Some("The sync keeps a cache.")
    );
}
