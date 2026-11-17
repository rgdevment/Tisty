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
fn every_write_tells_the_brain_to_look_again() {
    let kept = tempfile::tempdir().unwrap();
    let mut session = Session::at(somewhere(&kept)).unwrap();

    session
        .commit(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("comprar pan".to_string(), "a0"),
        })
        .unwrap();
    assert!(session.brain_nudge.taken(), "a commit nudges the brain");

    session
        .commit_all(vec![Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("y leche".to_string(), "a0"),
        }])
        .unwrap();
    assert!(session.brain_nudge.taken(), "a batch nudges the brain");
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

#[test]
fn a_document_that_cannot_be_read_counts_nothing_loose_and_names_itself() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let session = Session::at(paths.clone()).unwrap();
    let device = tisty_core::DeviceId("dev_a".into());
    std::fs::create_dir_all(paths.data().join("attachments")).unwrap();
    std::fs::write(paths.data().join("attachments/foto-91f2ab00.png"), b"png").unwrap();
    let broken = tisty_core::docs::create(&paths.docs(), &device, "# Plan").unwrap();
    let at = tisty_core::docs::resolve(&paths.docs(), &broken.id).unwrap();
    std::fs::write(at, b"# Plan\n\n\xff\xfe").unwrap();

    let (adrift, unread) = session.checked_adrift();

    assert_eq!(unread.as_deref(), Some("Plan"));
    assert_eq!(adrift.files(), 0);
}

#[test]
fn letting_go_while_a_document_cannot_be_read_names_the_document_not_its_id() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let mut session = Session::at(paths.clone()).unwrap();
    let device = tisty_core::DeviceId("dev_a".into());
    let broken = tisty_core::docs::create(&paths.docs(), &device, "# Plan").unwrap();
    let at = tisty_core::docs::resolve(&paths.docs(), &broken.id).unwrap();
    std::fs::write(at, b"# Plan\n\n\xff\xfe").unwrap();

    let refused = session
        .retire(
            &["attachments/foto-91f2ab00.png".to_string()],
            tisty_core::attach::Retired::Unused,
        )
        .unwrap_err();

    assert_eq!(refused.code, "retireUnread");
    assert_eq!(refused.name.as_deref(), Some("Plan"));
}

#[test]
fn what_tisty_made_takes_its_colours_on_the_first_opening_and_a_colour_taken_off_stays_off() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let work = ulid::Ulid::generate();
    {
        let mut before = Session::at(paths.clone()).unwrap();
        assert_eq!(
            before.config.dressed, None,
            "an empty store waits for what it will hold"
        );
        before
            .commit_all(vec![
                Op::ListAdd {
                    id: work,
                    d: tisty_core::event::ListAdd {
                        name: "Trabajo".into(),
                        order: "a0".into(),
                        color: None,
                    },
                },
                Op::ListLook {
                    id: work,
                    d: tisty_core::event::Look {
                        icon: Some(Some("work".into())),
                        color: None,
                    },
                },
            ])
            .unwrap();
    }

    let mut opened = Session::at(paths.clone()).unwrap();
    assert_eq!(opened.state.lists[&work].color.as_deref(), Some("blue"));
    assert_eq!(opened.config.dressed, Some(true));

    opened
        .commit(Op::ListLook {
            id: work,
            d: tisty_core::event::Look {
                icon: None,
                color: Some(None),
            },
        })
        .unwrap();
    drop(opened);

    let again = Session::at(paths).unwrap();
    assert_eq!(again.state.lists[&work].color, None);
}

fn added(title: &str) -> Op {
    Op::TaskAdd {
        id: ulid::Ulid::generate(),
        d: TaskAdd::new(title.to_string(), "a0"),
    }
}

// Opening the door for assistants on a first launch writes this machine's history through a store of its own.
fn written_beside(paths: &Paths, session: &Session) -> ulid::Ulid {
    let theirs = ulid::Ulid::generate();
    tisty_core::Store::open(paths.store(), session.config.device_id.clone())
        .unwrap()
        .append(Op::TaskAdd {
            id: theirs,
            d: TaskAdd::new("lo que escribió la puerta".to_string(), "a0"),
        })
        .unwrap();
    theirs
}

#[test]
fn a_commit_that_finds_another_hand_on_this_machine_reads_it_in_on_the_next_reload() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let mut session = Session::at(paths.clone()).unwrap();
    session.commit(added("comprar pan")).unwrap();
    let theirs = written_beside(&paths, &session);

    session.commit(added("y leche")).unwrap();

    assert!(
        session.reload().unwrap(),
        "the commit found a line it had not read"
    );
    assert!(session.state.tasks.contains_key(&theirs));
}

#[test]
fn once_caught_up_a_commit_carries_the_cache_instead_of_throwing_it_away() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let mut session = Session::at(paths.clone()).unwrap();
    session.commit(added("comprar pan")).unwrap();
    written_beside(&paths, &session);
    session.commit(added("y leche")).unwrap();
    session.reload().unwrap();

    session.commit(added("y huevos")).unwrap();

    let cache = tisty_core::cache::Cache::open(paths.cache())
        .unwrap()
        .expect("the window keeps a cache");
    assert!(
        cache.load(&session.print, false).is_some(),
        "otherwise every reader rebuilds the cache from the whole history after each task"
    );
}

#[test]
fn opening_the_door_on_a_first_launch_leaves_the_next_commit_carrying_the_cache() {
    let kept = tempfile::tempdir().unwrap();
    let paths = somewhere(&kept);
    let mut session = Session::at(paths.clone()).unwrap();
    crate::answers::agents::opened_by_default(&mut session);
    assert!(
        session.config.agent_id.is_some(),
        "the door opened, or this proves nothing"
    );

    session.commit(added("comprar pan")).unwrap();

    assert!(
        !session.behind(),
        "what the door wrote is this window's own"
    );
    let cache = tisty_core::cache::Cache::open(paths.cache())
        .unwrap()
        .expect("the window keeps a cache");
    assert!(cache.load(&session.print, false).is_some());
}
