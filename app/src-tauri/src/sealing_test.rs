use super::*;
use tisty_core::paths::Paths;

fn claimed_at(secs_ago: u64) -> String {
    let at = tisty_carrier::policy::now().saturating_sub(secs_ago) as u128 * 1_000_000_000;
    format!("shape = 3\nsealing = \"copying:seal:dev_b:{at}\"\n")
}

#[test]
fn a_claim_stands_still_from_its_nonce_or_from_its_last_stir() {
    let idle = |word: &str| tisty_carrier::idle_for(word);
    let old = claimed_at(ORPHANED_AFTER + 60);
    assert!(idle(&claimed_at(60)).is_some_and(|idle| idle < ORPHANED_AFTER));
    assert!(idle(&old).is_some_and(|idle| idle >= ORPHANED_AFTER));
    let stirred = format!(
        "{old}sealing_stirred = {}\n",
        tisty_carrier::policy::now() - 60
    );
    assert!(idle(&stirred).is_some_and(|idle| idle < ORPHANED_AFTER));
    assert_eq!(idle("shape = 3\n"), None);
}

#[test]
fn this_machine_is_never_waited_for_and_one_on_an_old_release_is() {
    let kept = tempfile::tempdir().unwrap();
    let paths = Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    let here = session.config.device_id.clone();
    let other = tisty_core::DeviceId("dev_zz".into());
    for device in [here.clone(), other.clone()] {
        session
            .commit(Op::DeviceJoin {
                d: device,
                k: Some(tisty_core::DeviceKind::Machine),
                p: None,
            })
            .unwrap();
    }

    let waited = behind(&session);

    assert_eq!(waited, vec![tisty_core::config::nicknamed(&other.0)]);
}
