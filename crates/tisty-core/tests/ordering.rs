use ulid::Ulid;

use tisty_core::event::{DeviceId, Filed, FolderAdd};
use tisty_core::model::FolderId;
use tisty_core::{Op, State, Store};

fn store(at: &std::path::Path) -> Store {
    Store::open(at, DeviceId("dev_a".into())).unwrap()
}

fn folder(store: &mut Store, name: &str, order: &str) -> FolderId {
    let id = Ulid::generate();
    store
        .append(Op::FolderAdd {
            id,
            d: FolderAdd {
                name: name.into(),
                order: order.into(),
                parent: None,
                icon: None,
                color: None,
            },
        })
        .unwrap();
    id
}

fn named(state: &State) -> Vec<String> {
    state
        .under(None)
        .into_iter()
        .map(|one| one.name.clone())
        .collect()
}

fn replayed(at: &std::path::Path) -> State {
    State::replay(&tisty_core::store::read_all(at).unwrap())
}

#[test]
fn a_folder_moved_before_another_stays_there_when_the_log_is_read_again() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("store");
    let mut store = store(&at);
    folder(&mut store, "trabajo", "a0");
    let personal = folder(&mut store, "personal", "a1");
    folder(&mut store, "casa", "a2");

    assert_eq!(named(&replayed(&at)), ["trabajo", "personal", "casa"]);

    store
        .append(Op::FolderMove {
            id: personal,
            d: Filed {
                folder: Some(None),
                page_of: None,
                order: Some(tisty_core::order::before("a0")),
            },
        })
        .unwrap();

    assert_eq!(
        named(&replayed(&at)),
        ["personal", "trabajo", "casa"],
        "where it was put is where it stays"
    );
}

#[test]
fn moving_a_folder_without_an_order_leaves_the_one_it_had() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("store");
    let mut store = store(&at);
    folder(&mut store, "trabajo", "a0");
    let personal = folder(&mut store, "personal", "a1");

    store
        .append(Op::FolderMove {
            id: personal,
            d: Filed {
                folder: Some(None),
                page_of: None,
                order: None,
            },
        })
        .unwrap();

    assert_eq!(named(&replayed(&at)), ["trabajo", "personal"]);
}

#[test]
fn two_machines_that_order_the_same_folders_land_on_the_same_shelf() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("store");
    let mut store = store(&at);
    let one = folder(&mut store, "uno", "a0");
    let two = folder(&mut store, "dos", "a1");

    store
        .append(Op::FolderMove {
            id: two,
            d: Filed {
                folder: Some(None),
                page_of: None,
                order: Some(tisty_core::order::between(None, Some("a0"))),
            },
        })
        .unwrap();
    store
        .append(Op::FolderMove {
            id: one,
            d: Filed {
                folder: Some(None),
                page_of: None,
                order: Some(tisty_core::order::after("a1")),
            },
        })
        .unwrap();

    let mine = named(&replayed(&at));
    let theirs = named(&State::replay(&tisty_core::store::read_all(&at).unwrap()));
    assert_eq!(mine, theirs, "the same log reads the same both times");
    assert_eq!(mine, ["dos", "uno"]);
}
