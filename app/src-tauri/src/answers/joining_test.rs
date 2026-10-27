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

    let backup = quietly_joined(&paths, true).expect("a new machine was asked anyway");

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

    assert!(quietly_joined(&paths, true).is_none());
    assert_eq!(
        tisty_core::store::read_all(paths.store()).unwrap().len(),
        before
    );
    assert!(!paths.private().join("before-joining").exists());
}

#[test]
fn a_place_that_names_no_history_is_never_taken_without_asking() {
    let kin = tisty_carrier::Kin::Strangers;

    assert!(!may_adopt("wouldReset", Some("01THEIRS"), &kin, None));
    assert!(!may_adopt("otherStore", None, &kin, None));
    assert!(!may_adopt("otherStore", Some("  "), &kin, None));
}

#[test]
fn a_new_machine_takes_a_named_history_it_has_never_taken() {
    assert!(may_adopt(
        "otherStore",
        Some("01THEIRS"),
        &tisty_carrier::Kin::Strangers,
        None
    ));
    assert!(may_adopt(
        "otherStore",
        Some("01THEIRS"),
        &tisty_carrier::Kin::SameLineage,
        Some("01EARLIER")
    ));
}

#[test]
fn a_history_whose_names_clash_or_cannot_be_read_still_opens_the_doors() {
    for kin in [
        tisty_carrier::Kin::Clash("mac_a".into()),
        tisty_carrier::Kin::Unsure("mac_a".into()),
    ] {
        assert!(!may_adopt("otherStore", Some("01THEIRS"), &kin, None));
    }
}

#[test]
fn the_same_history_is_never_taken_twice_without_asking() {
    assert!(!may_adopt(
        "otherStore",
        Some("01THEIRS"),
        &tisty_carrier::Kin::Strangers,
        Some("01THEIRS\n")
    ));
}

#[test]
fn a_machine_already_set_aside_is_not_set_aside_again() {
    let (_room, paths, store) = opened_once();
    drop(store);
    quietly_joined(&paths, true).expect("a new machine was asked anyway");

    assert!(quietly_joined(&paths, true).is_none());
    let kept = std::fs::read_dir(paths.private().join("before-joining"))
        .unwrap()
        .count();
    assert_eq!(kept, 1);
}

#[test]
fn a_machine_let_into_the_encrypted_cloud_joins_quietly_under_the_name_it_was_let_in_with() {
    let (_room, paths, store) = opened_once();
    drop(store);
    let was = tisty_core::Config::load_or_init(&paths).unwrap().device_id;

    quietly_joined(&paths, false).expect("a new machine was asked anyway");

    assert!(!tisty_core::store::inhabited(paths.store()));
    assert_eq!(
        tisty_core::Config::load_or_init(&paths).unwrap().device_id,
        was
    );
}

#[test]
fn only_the_key_handed_for_the_encrypted_cloud_it_syncs_with_keeps_a_machine_s_name() {
    let room = tempfile::tempdir().unwrap();
    let paths = Paths::new(room.path().join("data"), room.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    let at = tisty_core::config::CloudAt {
        provider: tisty_core::config::WEBDAV.into(),
        account: "https://mario@cloud.example.com/dav/Tisty/".into(),
        email: "mario@cloud.example.com".into(),
    };
    session.config.key_handed = Some("store_cloud".into());
    assert!(renaming(&session)(), "a machine on no cloud kept its name");

    session.config.sync = Some(tisty_core::config::Sync::Cloud(at.clone()));
    let tree = tisty_carrier::mirrored_at(&session.reach.home_of(&at));
    std::fs::create_dir_all(&tree).unwrap();
    std::fs::write(tree.join(tisty_carrier::NAMED), "shape = 3\n").unwrap();
    assert!(renaming(&session)(), "a plain cloud kept its name");

    std::fs::write(
        tree.join(tisty_carrier::NAMED),
        "shape = 3\nneeds = \"1.35.0\"\nsealed = \"store_cloud\"\n",
    )
    .unwrap();
    assert!(!renaming(&session)());

    let home = session.reach.home_of(&at);
    std::fs::write(home.join(tisty_carrier::CARRIED), "1").unwrap();
    assert!(
        renaming(&session)(),
        "a machine that already synced with the cloud kept its name"
    );
    std::fs::remove_file(home.join(tisty_carrier::CARRIED)).unwrap();
    assert!(!renaming(&session)());

    session.config.key_handed = None;
    assert!(
        renaming(&session)(),
        "a machine nobody let in kept its name"
    );
}

#[test]
fn a_look_at_the_place_tells_a_store_already_there_from_an_empty_place() {
    let room = tempfile::tempdir().unwrap();
    let place = room.path().join("place");
    let mine = room.path().join("mine").join("store");
    let folder = tisty_carrier::considering(&place);

    assert_eq!(
        peeked(&*folder, &mine, true),
        Peeked {
            fresh: true,
            holds: Some(false)
        }
    );

    Store::open(place.join(tisty_carrier::STORE), DeviceId("win_b".into()))
        .unwrap()
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: TaskAdd::new("Lo de otro equipo", "a0"),
        })
        .unwrap();

    assert_eq!(
        peeked(&*folder, &mine, true),
        Peeked {
            fresh: true,
            holds: Some(true)
        }
    );
}
