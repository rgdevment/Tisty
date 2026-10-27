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

#[test]
fn a_wait_says_whether_the_way_is_cut_blocked_or_the_cloud_asked_for_it() {
    let cut = Hitch::Unreachable(None, "no route".into());
    let filtered = Hitch::Unreachable(Some(tisty_carrier::Cause::Filtered), "tls".into());
    let asked = Hitch::Limited {
        wait: Duration::from_secs(3),
    };

    assert_eq!(waiting_on(&cut), "offline");
    assert_eq!(waiting_on(&filtered), "filtered");
    assert_eq!(waiting_on(&asked), "cloud");
    assert_eq!(
        waiting_on(&Hitch::Busy {
            wait: Duration::from_secs(5)
        }),
        "busy"
    );
    assert_eq!(waiting_on(&Hitch::Spent), "budget");
}

#[test]
fn an_old_copy_is_left_until_this_computer_wiped_it_for_this_cloud() {
    let home = tempfile::tempdir().unwrap();
    assert!(plain_left(home.path(), "cloud-a"));
    std::fs::write(home.path().join(PLAIN_WIPED), "cloud-a").unwrap();
    assert!(!plain_left(home.path(), "cloud-a"));
    assert!(plain_left(home.path(), "cloud-b"));
}
