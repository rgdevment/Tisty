use super::*;
use tisty_core::{Op, Paths};
use tisty_share_proto::seed::Seed;

#[test]
fn the_overview_never_says_a_phrase_confirmed_that_this_machine_no_longer_keeps() {
    let kept = tempfile::tempdir().unwrap();
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
    let seed = Seed::from_bytes(&[7; 32]);
    crate::identity::begun(&mut session, &seed).unwrap();
    session.config.phrase_confirmed = Some(seed.root().person().fingerprint());

    assert_eq!(standing(&Mutex::new(session)).phrase, "elsewhere");
}
