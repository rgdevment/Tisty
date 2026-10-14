use std::path::Path;

use tisty_core::event::{DocAdd, FolderAdd, Said, TaskAdd};
use tisty_core::untouched::{GUIDE_FOLDERS, SOWN_BY, untouched};
use tisty_core::{DeviceId, Op, Paths, Store};
use ulid::Ulid;

fn copied(from: &Path, into: &Path) {
    std::fs::create_dir_all(into).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let to = into.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copied(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), to).unwrap();
        }
    }
}

fn opened_once(paths: &Paths) -> Store {
    let who = DeviceId("mac_a".into());
    let mut store = Store::open(paths.store(), who.clone()).unwrap();
    store.append_batch(tisty_core::model::sown("es")).unwrap();
    let folder = Ulid::generate();
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
            id: Ulid::generate(),
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
    store
}

#[test]
fn no_store_an_earlier_release_wrote_with_tasks_in_it_is_ever_taken_for_new() {
    for schema in [15, 16, 17] {
        let room = tempfile::tempdir().unwrap();
        let store = room.path().join("store");
        let from = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/stores")
            .join(format!("v{schema}"));
        copied(&from, &store);

        assert!(
            !untouched(&store, &room.path().join("docs")),
            "the v{schema} store was taken for new"
        );
    }
}

#[test]
fn a_machine_opened_once_is_new_until_the_person_writes_the_first_task() {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    let mut store = opened_once(&paths);
    assert!(untouched(&paths.store(), &paths.docs()));

    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("Comprar pan", "a0"),
        })
        .unwrap();

    assert!(!untouched(&paths.store(), &paths.docs()));
}

#[test]
fn taking_the_places_history_keeps_what_a_new_machine_had_in_a_backup_first() {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    drop(opened_once(&paths));
    assert!(untouched(&paths.store(), &paths.docs()));
    let into = tisty_core::backup::kept_before_joining(&paths, jiff::Timestamp::now()).unwrap();

    let made = tisty_core::backup::reset(&paths, &into, paths.cache(), None).unwrap();

    assert!(into.is_file());
    assert!(made.files > 0);
    assert!(!tisty_core::store::inhabited(paths.store()));
    assert!(tisty_core::docs::names(&paths.docs()).is_empty());
}
