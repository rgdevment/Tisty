use super::{Session, projected};
use tisty_core::Op;
use tisty_core::event::TaskAdd;
use tisty_core::paths::Paths;

fn somewhere(kept: &tempfile::TempDir) -> Paths {
    Paths::new(kept.path().join("data"), kept.path().join("config"))
}

#[test]
fn a_commit_that_lands_while_the_store_is_read_survives_the_projection_that_missed_it() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let mut session = Session::at(paths.clone()).unwrap();

    let reading = projected(&paths, session.writes()).unwrap();

    let wrote = ulid::Ulid::generate();
    session
        .commit(Op::TaskAdd {
            id: wrote,
            d: TaskAdd::new("comprar pan".to_string(), "a0"),
        })
        .unwrap();
    assert!(session.state.tasks.contains_key(&wrote));

    session.adopt(reading);
    assert!(
        !session.state.tasks.contains_key(&wrote),
        "the reading began before the commit, so it cannot hold it"
    );

    session
        .commit(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("y leche".to_string(), "a0"),
        })
        .unwrap();

    assert!(session.reload().unwrap(), "the session knows it is behind");
    assert!(
        session.state.tasks.contains_key(&wrote),
        "what was written during the reading is back"
    );
}

#[test]
fn a_projection_nothing_interrupted_leaves_the_session_settled() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let mut session = Session::at(paths.clone()).unwrap();

    session
        .commit(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("comprar pan".to_string(), "a0"),
        })
        .unwrap();

    session.adopt(projected(&paths, session.writes()).unwrap());
    assert!(!session.reload().unwrap(), "nothing moved underneath it");
}

#[test]
fn writing_over_a_store_a_round_wrote_into_still_brings_that_machine_in() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let mut session = Session::at(paths.clone()).unwrap();

    let theirs = ulid::Ulid::generate();
    let mut other = tisty_core::Store::open(
        paths.store(),
        tisty_core::event::DeviceId("dev_other".into()),
    )
    .unwrap();
    other
        .append(Op::TaskAdd {
            id: theirs,
            d: TaskAdd::new("lo que trajo la ronda".to_string(), "a0"),
        })
        .unwrap();
    drop(other);

    session.fell_behind();
    session
        .commit(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("lo mio".to_string(), "a0"),
        })
        .unwrap();

    assert!(
        session.stale(),
        "the commit stamped a store it had not read"
    );
    assert!(session.reload().unwrap());
    assert!(session.state.tasks.contains_key(&theirs));
}
