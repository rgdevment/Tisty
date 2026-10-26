use super::*;
use tisty_core::Op;
use tisty_core::config::{CloudAt, Sync};
use tisty_core::paths::Paths;

const CLOUD: &str = "store_one";

fn account() -> CloudAt {
    CloudAt {
        provider: tisty_core::config::WEBDAV.into(),
        account: "https://mario@cloud.example.com/dav/Tisty/".into(),
        email: "mario@cloud.example.com".into(),
    }
}

struct Here {
    session: Mutex<Session>,
    signer: tisty_share_proto::handing::SigningKey,
    sealed_at: i64,
    _kept: tempfile::TempDir,
}

fn sealed_here() -> Here {
    let kept = tempfile::tempdir().unwrap();
    let paths = Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    let device = session.config.device_id.clone();
    session
        .commit(Op::DeviceJoin {
            d: device.clone(),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        })
        .unwrap();
    session
        .commit(Op::CloudSealed {
            s: CLOUD.into(),
            k: Some("named".into()),
        })
        .unwrap();
    let signer = tisty_core::signing::mine(&session.paths, &device).unwrap();
    session.state.keys.insert(
        device,
        tisty_core::signing::shown_of(&signer.verifying_key()),
    );
    session.config.sync = Some(Sync::Cloud(account()));
    let sealed_at = session.state.sealed_at[CLOUD];
    Here {
        session: Mutex::new(session),
        signer,
        sealed_at,
        _kept: kept,
    }
}

fn notice(here: &Here, named: &str, at: i64) -> Unsealed {
    let by = held(&here.session).config.device_id.0.clone();
    Unsealed::signed(CLOUD, named, &by, &here.signer, at)
}

fn word_here(here: &Here, word: &str) {
    let tree = {
        let session = held(&here.session);
        tisty_carrier::mirrored_at(&session.reach.home_of(&account()))
    };
    std::fs::create_dir_all(&tree).unwrap();
    std::fs::write(tree.join("tisty.toml"), word).unwrap();
}

#[test]
fn a_notice_from_a_seated_machine_retiring_the_logged_key_after_the_seal_is_believed() {
    let here = sealed_here();
    let said = notice(&here, "named", here.sealed_at + 60);

    assert!(believed(&held(&here.session), &said));
}

#[test]
fn a_notice_from_before_the_seal_for_another_key_or_from_a_stranger_is_never_believed() {
    let here = sealed_here();
    let (early, other) = (
        notice(&here, "named", here.sealed_at - 60),
        notice(&here, "other", here.sealed_at + 60),
    );
    let session = held(&here.session);

    assert!(!believed(&session, &early));
    assert!(!believed(&session, &other));
    let stranger = Unsealed::signed(
        CLOUD,
        "named",
        &session.config.device_id.0,
        &tisty_share_proto::handing::SigningKey::from_bytes(&[5; 32]),
        here.sealed_at + 60,
    );
    assert!(!believed(&session, &stranger));
    let unseated = Unsealed::signed(CLOUD, "named", "dev_zz", &here.signer, here.sealed_at + 60);
    assert!(!believed(&session, &unseated));
}

#[test]
fn a_believed_notice_lets_a_machine_without_the_key_carry_the_plain_cloud_again() {
    let here = sealed_here();
    word_here(&here, "shape = 3\n");
    tauri::async_runtime::block_on(crate::identity::seal_the_cloud(&here.session));
    assert_eq!(
        held(&here.session)
            .carrying()
            .err()
            .map(|refusal| refusal.code),
        Some("cloudSealed")
    );
    assert!(hears(&held(&here.session)));

    held(&here.session)
        .reach
        .unsealed_is(Some((CLOUD.into(), here.sealed_at + 60)));
    tauri::async_runtime::block_on(crate::identity::seal_the_cloud(&here.session));

    assert!(held(&here.session).reach.tried().is_none());
    assert_ne!(
        held(&here.session)
            .carrying()
            .err()
            .map(|refusal| refusal.code),
        Some("cloudSealed")
    );
    assert!(!hears(&held(&here.session)));
}

#[test]
fn a_notice_believed_before_a_later_seal_counts_for_nothing() {
    let here = sealed_here();
    held(&here.session)
        .reach
        .unsealed_is(Some((CLOUD.into(), here.sealed_at - 60)));

    assert_eq!(believed_now(&held(&here.session)), None);
}
