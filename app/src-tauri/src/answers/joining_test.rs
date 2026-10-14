use tisty_core::event::{DocAdd, FolderAdd, Said, TaskAdd};
use tisty_core::untouched::{GUIDE_FOLDERS, SOWN_BY};
use tisty_core::{DeviceId, Op, Paths, Store};

use super::*;

fn opened_once() -> (tempfile::TempDir, Paths, Store) {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    let who = DeviceId("mac_a".into());
    let mut store = Store::open(paths.store(), who.clone()).unwrap();
    store.append_batch(tisty_core::model::sown("es")).unwrap();
    let folder = ulid::Ulid::generate();
    store
        .append(Op::FolderAdd {
            id: folder,
            d: FolderAdd {
                name: GUIDE_FOLDERS[1].into(),
                order: "a0".into(),
                parent: None,
                icon: None,
                color: None,
            },
        })
        .unwrap();
    let body = "# Guía\n\nAsí se usa.\n";
    let made = tisty_core::docs::create(&paths.docs(), &who, body).unwrap();
    store
        .append(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: DocAdd {
                file: made.id,
                order: "a0".into(),
                by: Some(SOWN_BY.into()),
                guest: true,
                said: Some(Said::of(body)),
                folder: Some(folder),
                ..DocAdd::default()
            },
        })
        .unwrap();
    (room, paths, store)
}

#[test]
fn only_a_place_holding_another_history_opens_the_doors() {
    assert!(apart("wouldReset"));
    assert!(apart("otherStore"));
    assert!(!apart("restoredApart"));
    assert!(!apart("syncFailed"));
}

#[test]
fn a_machine_that_only_opened_once_joins_without_asking_and_keeps_a_backup() {
    let (_room, paths, store) = opened_once();
    drop(store);

    let backup = quietly_joined(&paths).expect("a new machine was asked anyway");

    assert!(backup.is_file());
    assert!(backup.starts_with(paths.private()));
    assert!(!tisty_core::store::inhabited(paths.store()));
}

#[test]
fn a_machine_with_a_single_task_of_the_persons_is_left_exactly_as_it_was() {
    let (_room, paths, mut store) = opened_once();
    store
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("Pagar la luz", "a0"),
        })
        .unwrap();
    drop(store);
    let before = tisty_core::store::read_all(paths.store()).unwrap().len();

    assert!(quietly_joined(&paths).is_none());
    assert_eq!(
        tisty_core::store::read_all(paths.store()).unwrap().len(),
        before
    );
    assert!(!paths.private().join("before-joining").exists());
}
