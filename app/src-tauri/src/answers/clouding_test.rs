use super::*;

fn ana() -> Sync {
    Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    })
}

fn config_with(sync: Option<Sync>) -> tisty_core::Config {
    let home = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(home.path().join("data"), home.path().join("config"));
    let mut config = tisty_core::Config::load_or_init(&paths).unwrap();
    config.sync = sync;
    config
}

#[test]
fn only_a_copy_that_shares_with_nobody_starts_on_the_cloud_here() {
    for alone in [None, Some(Sync::alone())] {
        let mut config = config_with(alone);
        assert!(choosing_cloud(ana(), true)(&mut config).is_none());
        assert_eq!(config.sync, Some(ana()));
    }

    let mut again = config_with(Some(ana()));
    assert!(choosing_cloud(ana(), true)(&mut again).is_none());

    let mut folder = config_with(Some(Sync::folder("G:/compartida")));
    let refused = choosing_cloud(ana(), true)(&mut folder);
    assert_eq!(refused.map(|one| one.code), Some("cloudNeedsMove"));
    assert_eq!(folder.sync, Some(Sync::folder("G:/compartida")));

    let other = Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "9999".into(),
        email: "otra@example.com".into(),
    });
    let mut moved = config_with(Some(other.clone()));
    let refused = choosing_cloud(ana(), true)(&mut moved);
    assert_eq!(refused.map(|one| one.code), Some("cloudNeedsMove"));
    assert_eq!(moved.sync, Some(other.clone()));

    let mut never_joined = config_with(Some(other));
    assert!(choosing_cloud(ana(), false)(&mut never_joined).is_none());
    assert_eq!(never_joined.sync, Some(ana()));
}

#[test]
fn what_google_answers_reads_as_what_the_window_can_say() {
    assert_eq!(refused(Hitch::Lost, "Google Drive").code, "cloudLost");
    assert_eq!(refused(Hitch::Full, "Google Drive").code, "cloudFull");
    let elsewhere = refused(
        Hitch::Elsewhere {
            found: "otra@example.com".into(),
        },
        "Google Drive",
    );
    assert_eq!(elsewhere.code, "cloudElsewhere");
    assert_eq!(elsewhere.name.as_deref(), Some("otra@example.com"));
    assert_eq!(
        refused(Hitch::Unreachable("sin red".into()), "Google Drive").code,
        "cloudNotNow"
    );
    assert_eq!(
        refused(Hitch::Broke("x".into()), "Google Drive").code,
        "syncBroke"
    );
}

#[test]
fn a_grant_waiting_for_its_account_is_held_only_in_memory() {
    let unfiled = Unfiled(Mutex::new(Some("1//refresh".into())));

    assert_eq!(
        unfiled.read(KEPT_AS).unwrap().as_deref(),
        Some("1//refresh")
    );
    unfiled.write(KEPT_AS, "1//rotated").unwrap();
    assert_eq!(
        unfiled.read(KEPT_AS).unwrap().as_deref(),
        Some("1//rotated")
    );
    unfiled.forget(KEPT_AS).unwrap();
    assert_eq!(unfiled.read(KEPT_AS).unwrap(), None);
}

#[test]
fn the_same_account_under_a_new_address_is_kept_and_the_address_written_down() {
    let renamed = Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana.nueva@example.com".into(),
    });
    let mut config = config_with(Some(ana()));

    assert!(choosing_cloud(renamed.clone(), true)(&mut config).is_none());
    assert_eq!(config.sync, Some(renamed));
}

#[test]
fn the_status_of_a_cloud_says_the_account_the_day_s_spend_the_switch_and_the_last_round() {
    let home = tempfile::tempdir().unwrap();
    let at = CloudAt {
        provider: GOOGLE.into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    };
    tisty_carrier::policy::pause_here(home.path(), true).unwrap();
    tisty_carrier::round_noted(
        home.path(),
        &tisty_carrier::LastRound {
            at: 1,
            code: "cloudFull".into(),
            name: Some("Google Drive".into()),
            left: 0,
            names: Vec::new(),
        },
    );

    let status = status_of(&at, home.path());

    assert_eq!(status.email, "ana@example.com");
    assert!(status.paused_here);
    assert_eq!(status.units_spent, 0);
    assert_eq!(
        status.last.map(|one| one.code),
        Some("cloudFull".to_string())
    );
}
