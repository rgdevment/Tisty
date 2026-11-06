use tisty_core::event::Remembered;
use tisty_core::{Op, Paths, Store, config::Config};

fn one(version: u64) -> Remembered {
    Remembered {
        id: "Ab3xK9pQ".to_owned(),
        version,
        link_key: Some("key".to_owned()),
        files_print: None,
        files_hash: None,
        files_key: None,
        admin_token: Some("admin".to_owned()),
        edit_token: None,
        words_print: None,
        signed_as: None,
        owner: None,
        owner_name: None,
        cited: Vec::new(),
        held_by: None,
        ttl_days: None,
        carried: None,
    }
}

fn kept(version: u64) -> Op {
    Op::SharingKept {
        file: "file".to_owned(),
        d: one(version),
    }
}

fn opened() -> (tempfile::TempDir, Paths, Store) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let device = Config::load_or_init(&paths).unwrap().device_id;
    let store = Store::open(paths.store(), device).unwrap();
    (tmp, paths, store)
}

#[test]
fn what_a_share_needs_comes_back_from_the_log_and_from_the_cache() {
    let (_tmp, paths, mut store) = opened();
    store.append(kept(1)).unwrap();

    let first = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
    let again = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();

    assert_eq!(first.sharing["file"], one(1));
    assert_eq!(again.sharing["file"], one(1));
}

#[test]
fn an_older_version_from_another_computer_never_replaces_a_newer_one() {
    let (_tmp, paths, mut store) = opened();
    store.append(kept(3)).unwrap();
    store.append(kept(2)).unwrap();

    let state = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();

    assert_eq!(state.sharing["file"].version, 3);
}

#[test]
fn a_stopped_share_is_forgotten_and_another_computers_late_word_does_not_bring_it_back() {
    let (_tmp, paths, mut store) = opened();
    store.append(kept(3)).unwrap();
    store
        .append(Op::SharingForgotten {
            file: "file".to_owned(),
            stopped: Some("Ab3xK9pQ".to_owned()),
        })
        .unwrap();
    store.append(kept(4)).unwrap();

    let state = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
    assert!(state.sharing.is_empty());
    let again = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
    assert!(again.sharing_stopped.contains("Ab3xK9pQ"));
}

#[test]
fn leaving_a_link_one_only_edited_lets_it_come_back() {
    let (_tmp, paths, mut store) = opened();
    store.append(kept(3)).unwrap();
    store
        .append(Op::SharingForgotten {
            file: "file".to_owned(),
            stopped: None,
        })
        .unwrap();
    store.append(kept(3)).unwrap();

    let state = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
    assert_eq!(state.sharing["file"].version, 3);
}

#[test]
fn an_earlier_build_is_told_it_may_skip_them() {
    assert!(kept(1).is_optional());
    assert!(
        Op::SharingForgotten {
            file: "file".to_owned(),
            stopped: None,
        }
        .is_optional()
    );
}
