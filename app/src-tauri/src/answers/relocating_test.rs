use super::*;
use crate::answers::carried_cloud::{
    a_session, carried_cloud, cloud_that, forgotten, holding_our_store, restored,
};

fn config_with(sync: Option<Sync>) -> (tempfile::TempDir, tisty_core::Config) {
    let home = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(home.path().join("data"), home.path().join("config"));
    let mut config = tisty_core::Config::load_or_init(&paths).unwrap();
    config.sync = sync;
    (home, config)
}

fn ana() -> Sync {
    Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    })
}

#[test]
fn a_move_that_brought_everything_over_may_leave_a_cloud() {
    let (_home, mut config) = config_with(Some(ana()));

    assert!(moving_in(Sync::folder("G:/compartida"))(&mut config).is_none());

    assert_eq!(config.sync, Some(Sync::folder("G:/compartida")));
}

#[test]
fn a_move_never_leaves_a_way_a_newer_build_chose() {
    let later = Sync::Unknown(toml::from_str("how = \"nube\"").unwrap());
    let (_home, mut config) = config_with(Some(later.clone()));

    let refused = moving_in(Sync::alone())(&mut config);

    assert_eq!(refused.map(|one| one.code), Some("syncLaterToLeave"));
    assert_eq!(config.sync, Some(later));
}

#[test]
fn where_a_move_goes_is_written_down_until_it_ends() {
    let home = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(home.path().join("data"), home.path().join("config"));
    let toward = Toward::Cloud {
        account: "0123".into(),
        email: "ana@example.com".into(),
        provider: "google".into(),
    };

    pending_kept(&paths, Some(&toward));
    let kept: Toward =
        serde_json::from_slice(&std::fs::read(paths.private().join(PENDING)).unwrap()).unwrap();
    assert_eq!(kept, toward);

    pending_kept(&paths, None);
    assert!(!paths.private().join(PENDING).exists());
}

#[test]
fn the_window_names_each_way_a_move_can_go() {
    let folder: Toward = serde_json::from_str(r#"{"kind":"folder","at":"G:/x"}"#).unwrap();
    let alone: Toward = serde_json::from_str(r#"{"kind":"alone"}"#).unwrap();

    assert_eq!(folder, Toward::Folder { at: "G:/x".into() });
    assert_eq!(alone, Toward::Alone);
}

#[test]
fn a_move_to_the_cloud_written_down_by_1_30_still_goes_to_google_drive() {
    let home = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(home.path().join("data"), home.path().join("config"));
    std::fs::create_dir_all(paths.private()).unwrap();
    std::fs::write(
        paths.private().join(PENDING),
        r#"{"kind":"cloud","account":"0123","email":"ana@example.com"}"#,
    )
    .unwrap();

    let pending = pending_cloud(&paths).unwrap();

    assert_eq!(pending.provider, "google");
    assert_eq!(pending.account, "0123");
    assert_eq!(pending.shown(), "Google Drive");
}

#[test]
fn a_move_that_has_to_wait_says_how_far_and_why() {
    let said = waiting(
        "homing",
        tisty_carrier::Owed {
            files: 3,
            bytes: 900,
        },
        None,
    );

    assert_eq!(
        serde_json::to_value(said).unwrap(),
        serde_json::json!({"state": "waiting", "stage": "homing", "left": {"files": 3, "bytes": 900}, "reason": null, "wait": null})
    );
}

#[test]
fn a_move_the_provider_asked_to_pause_says_for_how_long_so_the_window_can_go_on_by_itself() {
    let said = waiting(
        "filling",
        tisty_carrier::Owed {
            files: 2,
            bytes: 40,
        },
        Some(tisty_carrier::Deferred {
            reason: tisty_carrier::Reason::Limit,
            left: 2,
            retry_after: Some(std::time::Duration::from_secs(90)),
            names: Vec::new(),
            cause: None,
        }),
    );

    assert_eq!(
        serde_json::to_value(said).unwrap(),
        serde_json::json!({"state": "waiting", "stage": "filling", "left": {"files": 2, "bytes": 40}, "reason": "limit", "wait": 90})
    );
}

#[test]
fn a_move_paused_by_something_in_the_way_names_it_for_the_window() {
    let said = waiting(
        "filling",
        tisty_carrier::Owed { files: 1, bytes: 5 },
        Some(tisty_carrier::Deferred {
            reason: tisty_carrier::Reason::Offline,
            left: 1,
            retry_after: None,
            names: Vec::new(),
            cause: Some(tisty_carrier::Cause::Filtered),
        }),
    );

    let said = serde_json::to_value(said).unwrap();
    assert_eq!(said["reason"], "offline");
    assert_eq!(said["cause"], "filtered");
}

fn toward_cloud(at: &CloudAt) -> Toward {
    Toward::Cloud {
        account: at.account.clone(),
        email: at.email.clone(),
        provider: at.provider.clone(),
    }
}

fn syncing(session: &Mutex<Session>, way: Sync) {
    held(session)
        .keep(|config| config.sync = Some(way))
        .unwrap();
}

#[test]
fn a_move_of_a_restored_store_asks_whose_store_the_cloud_holds_with_the_session_let_go_of() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (at, asked) = carried_cloud(&session, &std::sync::Arc::new(OneAtATime::idle()));
    syncing(&session, Sync::alone());
    restored(&session);

    let end = ends(&session, &toward_cloud(&at)).ok().unwrap();

    let (free, _) = asked
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("a restored store never asked whose store the cloud holds");
    assert!(
        free,
        "the window waited on the network behind the session every command needs"
    );
    assert!(end.to.is_some() && end.from.is_none() && end.left.is_none());
    assert_eq!(end.way, Sync::cloud(at.clone()));
    forgotten(&at);
}

#[test]
fn a_move_of_a_store_never_restored_asks_the_cloud_nothing() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let (at, asked) = carried_cloud(&session, &std::sync::Arc::new(OneAtATime::idle()));
    syncing(&session, Sync::alone());

    assert!(ends(&session, &toward_cloud(&at)).is_ok());

    assert!(
        asked
            .recv_timeout(std::time::Duration::from_millis(300))
            .is_err()
    );
    forgotten(&at);
}

#[test]
fn a_move_to_the_place_chosen_while_it_answered_is_no_move() {
    let home = tempfile::tempdir().unwrap();
    let session = a_session(home.path());
    let chosen = std::sync::Arc::new(Mutex::new(None));
    let (theirs, there) = (session.clone(), chosen.clone());
    let at = cloud_that(&session, move || {
        let at = there.lock().unwrap().clone();
        syncing(&theirs, Sync::cloud(Option::unwrap(at)));
    });
    *chosen.lock().unwrap() = Some(at.clone());
    syncing(&session, Sync::alone());
    restored(&session);

    let refused = ends(&session, &toward_cloud(&at)).err().unwrap();

    assert_eq!(refused.code, "moveSamePlace");
    forgotten(&at);
}

#[test]
fn a_restored_store_never_moves_into_the_folder_it_went_back_from() {
    let room = tempfile::tempdir().unwrap();
    let session = a_session(&room.path().join("one"));
    let shared = room.path().join("shared");
    holding_our_store(&session, &shared);
    restored(&session);
    let toward = Toward::Folder {
        at: shared.display().to_string(),
    };

    let refused = ends(&session, &toward).err().unwrap();

    assert_eq!(
        (refused.code, refused.name),
        ("restoredApart", Some(shared.display().to_string()))
    );
}
