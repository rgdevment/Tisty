use super::*;

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
