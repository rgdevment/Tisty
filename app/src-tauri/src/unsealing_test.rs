use super::*;
use tisty_core::Op;
use tisty_core::config::{CloudAt, Sync};
use tisty_core::paths::Paths;

fn account() -> CloudAt {
    CloudAt {
        provider: tisty_core::config::WEBDAV.into(),
        account: "https://mario@cloud.example.com/dav/Tisty/".into(),
        email: "mario@cloud.example.com".into(),
    }
}

fn opened(kept: &tempfile::TempDir) -> Session {
    let paths = Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    let here = session.config.device_id.clone();
    session
        .commit(Op::DeviceJoin {
            d: here,
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        })
        .unwrap();
    session.config.sync = Some(Sync::Cloud(account()));
    session
}

fn word_here(session: &Session, word: &str) {
    let tree = tisty_carrier::mirrored_at(&session.reach.home_of(&account()));
    std::fs::create_dir_all(&tree).unwrap();
    std::fs::write(tree.join("tisty.toml"), word).unwrap();
}

fn sealed_in_the_log(session: &mut Session) {
    session
        .commit(Op::CloudSealed {
            s: "cloud_one".into(),
            k: Some("named".into()),
        })
        .unwrap();
}

#[test]
fn a_place_said_plain_while_the_log_names_it_sealed_is_heard_first() {
    let kept = tempfile::tempdir().unwrap();
    let mut session = opened(&kept);
    sealed_in_the_log(&mut session);
    word_here(&session, "shape = 3\n");

    assert!(hears(&session));
}

#[test]
fn a_place_still_said_sealed_is_heard_only_where_no_key_opens_it() {
    let kept = tempfile::tempdir().unwrap();
    let mut session = opened(&kept);
    sealed_in_the_log(&mut session);
    word_here(&session, "shape = 3\nsealed = \"cloud_one\"\n");
    assert!(!hears(&session));

    let locked = crate::reaching::Locked {
        at: account(),
        cloud: "cloud_one".into(),
        named: "named".into(),
    };
    session.reach.tried_for(Some((locked, 0)));

    assert!(hears(&session));
}

#[test]
fn a_place_the_log_never_sealed_or_a_sync_that_is_no_cloud_is_never_heard_first() {
    let kept = tempfile::tempdir().unwrap();
    let mut session = opened(&kept);
    word_here(&session, "shape = 3\n");
    assert!(!hears(&session));

    sealed_in_the_log(&mut session);
    session.config.sync = None;

    assert!(!hears(&session));
}
