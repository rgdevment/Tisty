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
        sealed_under: None,
        version_by: None,
        version_root: None,
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

#[test]
fn only_a_token_without_the_link_to_manage_it_is_editing_through_a_link() {
    let owner = Remembered {
        edit_token: Some("token".to_owned()),
        ..one(1)
    };
    let editor = Remembered {
        admin_token: None,
        ..owner.clone()
    };
    let reader = Remembered {
        admin_token: None,
        ..one(1)
    };
    assert!(!owner.edited_through_a_link());
    assert!(editor.edited_through_a_link());
    assert!(!reader.edited_through_a_link());
    assert!(!owner.read_only());
    assert!(!editor.read_only());
    assert!(reader.read_only());
}

#[test]
fn what_an_older_tisty_remembered_still_reads_without_the_newer_fields() {
    let older = r#"{"id":"Ab3xK9pQ","version":2,"link_key":"key"}"#;
    let read: Remembered = serde_json::from_str(older).unwrap();
    assert_eq!(read.sealed_under, None);
    assert_eq!(read.version_by, None);
}

#[test]
fn who_edited_a_share_outlives_it_in_the_log_and_in_the_cache() {
    let (_tmp, paths, mut store) = opened();
    let editors = r#"[{"name":"Mario","owned":true},{"name":"Ana"}]"#;
    store
        .append(Op::SharingKept {
            file: "file".to_owned(),
            d: Remembered {
                carried: Some(format!(r#"{{"editors":{editors}}}"#)),
                ..one(3)
            },
        })
        .unwrap();
    store
        .append(Op::SharingForgotten {
            file: "file".to_owned(),
            stopped: Some("Ab3xK9pQ".to_owned()),
        })
        .unwrap();
    store
        .append(Op::SharingKept {
            file: "other".to_owned(),
            d: Remembered {
                admin_token: None,
                carried: Some(format!(r#"{{"editors":{editors}}}"#)),
                ..one(1)
            },
        })
        .unwrap();
    store
        .append(Op::SharingForgotten {
            file: "other".to_owned(),
            stopped: None,
        })
        .unwrap();

    let first = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
    let again = tisty_core::cache::project(&paths.store(), paths.cache()).unwrap();
    for state in [first, again] {
        let kept: serde_json::Value = serde_json::from_str(&state.sharing_editors["file"]).unwrap();
        assert_eq!(
            kept,
            serde_json::from_str::<serde_json::Value>(editors).unwrap()
        );
        assert!(!state.sharing_editors.contains_key("other"));
    }
}
