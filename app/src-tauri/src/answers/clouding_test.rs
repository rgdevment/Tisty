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
        assert!(choosing_cloud(ana())(&mut config).is_none());
        assert_eq!(config.sync, Some(ana()));
    }

    let mut again = config_with(Some(ana()));
    assert!(choosing_cloud(ana())(&mut again).is_none());

    let mut folder = config_with(Some(Sync::folder("G:/compartida")));
    let refused = choosing_cloud(ana())(&mut folder);
    assert_eq!(refused.map(|one| one.code), Some("cloudNeedsMove"));
    assert_eq!(folder.sync, Some(Sync::folder("G:/compartida")));

    let other = Sync::cloud(CloudAt {
        provider: GOOGLE.into(),
        account: "9999".into(),
        email: "otra@example.com".into(),
    });
    let mut moved = config_with(Some(other.clone()));
    let refused = choosing_cloud(ana())(&mut moved);
    assert_eq!(refused.map(|one| one.code), Some("cloudNeedsMove"));
    assert_eq!(moved.sync, Some(other));
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

    assert!(choosing_cloud(renamed.clone())(&mut config).is_none());
    assert_eq!(config.sync, Some(renamed));
}
