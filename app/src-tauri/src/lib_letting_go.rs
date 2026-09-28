use super::{Session, erasing, opening_to_agents, reading_as};
use tisty_core::{DeviceId, Op, Paths, Reading, TaskId};

struct Desk {
    _tmp: tempfile::TempDir,
    paths: Paths,
}

fn desk() -> Desk {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    Desk { _tmp: tmp, paths }
}

fn closed(session: &mut Session, title: &str) -> TaskId {
    let id = ulid::Ulid::generate();
    session
        .commit(Op::TaskAdd {
            id,
            d: tisty_core::event::TaskAdd::new(title, "a0"),
        })
        .unwrap();
    session.commit(Op::TaskDone { id, filled: false }).unwrap();
    id
}

fn code(said: Result<impl std::fmt::Debug, super::Refusal>) -> String {
    match said {
        Ok(_) => "ok".into(),
        Err(refusal) => refusal.code.to_string(),
    }
}

#[test]
fn erasing_keeps_the_rule_the_core_keeps() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let open = ulid::Ulid::generate();
    session
        .commit(Op::TaskAdd {
            id: open,
            d: tisty_core::event::TaskAdd::new("still open", "a0"),
        })
        .unwrap();
    let errand = closed(&mut session, "buy bread");
    let kept = closed(&mut session, "the certificate");
    reading_as(&mut session, kept, Reading::Story).unwrap();

    assert_eq!(code(erasing(&mut session, open)), "onlyArchivedGoes");
    assert_eq!(code(erasing(&mut session, kept)), "storyStays");
    assert_eq!(
        code(erasing(&mut session, ulid::Ulid::generate())),
        "notATaskId"
    );
    assert_eq!(code(erasing(&mut session, errand)), "ok");
    assert!(session.state.is_erased(errand));
}

#[test]
fn converting_is_for_what_is_closed_and_not_a_routine() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let open = ulid::Ulid::generate();
    session
        .commit(Op::TaskAdd {
            id: open,
            d: tisty_core::event::TaskAdd::new("still open", "a0"),
        })
        .unwrap();
    let turn = ulid::Ulid::generate();
    let mut d = tisty_core::event::TaskAdd::new("pills", "a1");
    d.after = Some(ulid::Ulid::generate());
    session.commit(Op::TaskAdd { id: turn, d }).unwrap();
    session
        .commit(Op::TaskDone {
            id: turn,
            filled: false,
        })
        .unwrap();
    let errand = closed(&mut session, "buy bread");

    assert_eq!(
        code(reading_as(&mut session, open, Reading::Story)),
        "onlyClosedConverts"
    );
    assert_eq!(
        code(reading_as(&mut session, turn, Reading::Trace)),
        "routineReadsAsRoutine"
    );
    let told = reading_as(&mut session, errand, Reading::Story).unwrap();
    assert_eq!(told.read_as, Some(Reading::Story));
    assert_eq!(code(erasing(&mut session, errand)), "storyStays");
    assert_eq!(code(erasing(&mut session, turn)), "routineStays");
}

/// A closed root with no repeat left reads as a trace by itself; the turn hanging from it
/// makes it a routine's to the state, so the window refuses to erase it.
#[test]
fn a_bare_root_a_turn_hangs_from_is_never_erased() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let root = closed(&mut session, "pills");
    let turn = ulid::Ulid::generate();
    let mut d = tisty_core::event::TaskAdd::new("pills", "a1");
    d.after = Some(root);
    session.commit(Op::TaskAdd { id: turn, d }).unwrap();
    session
        .commit(Op::TaskDone {
            id: turn,
            filled: false,
        })
        .unwrap();
    let errand = closed(&mut session, "buy bread");

    assert_eq!(code(erasing(&mut session, root)), "routineStays");
    assert_eq!(code(erasing(&mut session, errand)), "ok");
    assert!(session.state.tasks.contains_key(&root));
}

#[test]
fn opening_to_agents_is_for_an_open_task_the_person_wrote() {
    let desk = desk();
    let mut session = Session::at(desk.paths.clone()).unwrap();
    let mine = ulid::Ulid::generate();
    session
        .commit(Op::TaskAdd {
            id: mine,
            d: tisty_core::event::TaskAdd::new("renew the certificate", "a0"),
        })
        .unwrap();
    let errand = closed(&mut session, "buy bread");
    let theirs = ulid::Ulid::generate();
    let agent = DeviceId("dev_agent".into());
    let mut wrote = tisty_core::Store::open(desk.paths.store(), agent.clone()).unwrap();
    wrote
        .append(Op::DeviceJoin {
            d: agent,
            k: Some(tisty_core::event::DeviceKind::Agent),
        })
        .unwrap();
    wrote
        .append(Op::TaskAdd {
            id: theirs,
            d: tisty_core::event::TaskAdd::new("pasar biome", "a1"),
        })
        .unwrap();

    assert_eq!(
        code(opening_to_agents(&mut session, errand, true)),
        "onlyOpenOpens"
    );
    assert_eq!(
        code(opening_to_agents(&mut session, theirs, true)),
        "alreadyTheirs"
    );
    assert_eq!(
        code(opening_to_agents(
            &mut session,
            ulid::Ulid::generate(),
            true
        )),
        "notATaskId"
    );
    let told = opening_to_agents(&mut session, mine, true).unwrap();
    assert!(told.open_to_agents);
    assert!(
        session
            .state
            .attended_by_agents(&session.state.tasks[&mine])
    );
    let told = opening_to_agents(&mut session, mine, false).unwrap();
    assert!(!told.open_to_agents);
}
