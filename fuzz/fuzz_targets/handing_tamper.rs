#![no_main]

use std::sync::LazyLock;

use arbitrary::Arbitrary;
use jiff::Timestamp;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{self, Ask, Handed, OneUse, SigningKey};
use tisty_share_proto::identity::{Root, ShareKey, Verdict};

static ONE_USE: LazyLock<OneUse> = LazyLock::new(|| OneUse::minted().expect("minted"));

#[derive(Arbitrary, Debug)]
struct Input {
    device: String,
    name: String,
    cloud: String,
    by: String,
    at: i64,
    field: u8,
    text: String,
    number: i64,
}

fn ask_changed(ask: &Ask, input: &Input) -> Ask {
    let mut changed = ask.clone();
    let text = input.text.clone();
    match input.field % 9 {
        0 => changed.v = input.number as u32,
        1 => changed.device = text,
        2 => changed.name = text,
        3 => changed.key = text,
        4 => changed.one_use = text,
        5 => changed.commit = text,
        6 => changed.cloud = text,
        7 => changed.at = input.number,
        _ => changed.sig = text,
    }
    changed
}

fn handed_changed(handed: &Handed, input: &Input) -> Handed {
    let mut changed = handed.clone();
    let text = input.text.clone();
    match input.field % 6 {
        0 => changed.v = input.number as u32,
        1 => changed.by = text,
        2 => changed.key = text,
        3 => changed.from = text,
        4 => changed.sealed = text,
        _ => changed.sig = text,
    }
    changed
}

fn holds(verdict: Verdict) -> bool {
    matches!(verdict, Verdict::Holds { .. })
}

fuzz_target!(|input: Input| {
    let machine = SigningKey::from_bytes(&[1; 32]);
    let giver = SigningKey::from_bytes(&[2; 32]);
    let share = ShareKey::from_bytes(&[3; 32]);
    let root = Root::from_bytes(&[4; 32]);
    let key = CloudKey::from_bytes(&[5; 32]);
    let certificate = root.certify(&share.public(), Timestamp::UNIX_EPOCH);

    let ask = Ask::signed(
        &input.device,
        &input.name,
        &machine,
        &ONE_USE,
        &input.cloud,
        input.at,
    );
    assert!(ask.holds());
    let mut handed = handing::handed(&ask, &key, &input.by, &giver).expect("handed");
    assert!(handed.vouch(&ask, &share, &certificate));
    assert!(handed.signed_for(&ask));
    assert_eq!(
        handed.opened(&ask, &ONE_USE).map(|one| *one.to_bytes()),
        Some([5; 32])
    );
    assert!(holds(handed.vouched_for(&ask, &[])));
    let revoked = root.revoke(&share.public(), Timestamp::UNIX_EPOCH);
    assert!(matches!(
        handed.vouched_for(&ask, &[revoked]),
        Verdict::Revoked { .. }
    ));

    let other = ask_changed(&ask, &input);
    if other != ask {
        assert!(!other.holds());
        assert!(!handed.signed_for(&other));
        assert!(handed.opened(&other, &ONE_USE).is_none());
        assert!(!holds(handed.vouched_for(&other, &[])));
    }

    let changed = handed_changed(&handed, &input);
    if changed != handed {
        assert!(!changed.signed_for(&ask));
        assert!(changed.opened(&ask, &ONE_USE).is_none());
        // The vouching signs everything the answer says but its own signature, which it stands in for.
        if changed.sig == handed.sig {
            assert!(!holds(changed.vouched_for(&ask, &[])));
        }
    }
});
