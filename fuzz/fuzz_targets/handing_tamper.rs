#![no_main]

use std::sync::LazyLock;

use arbitrary::Arbitrary;
use jiff::Timestamp;
use libfuzzer_sys::fuzz_target;
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{self, Ask, Handed, OneUse, SigningKey};
use tisty_share_proto::identity::{Certificate, Revocation, Root, ShareKey, Verdict};

struct Fixed {
    one_use: OneUse,
    machine: SigningKey,
    giver: SigningKey,
    share: ShareKey,
    key: CloudKey,
    certificate: Certificate,
    revoked: Revocation,
}

static FIXED: LazyLock<Fixed> = LazyLock::new(|| {
    let share = ShareKey::from_bytes(&[3; 32]);
    let root = Root::from_bytes(&[4; 32]);
    Fixed {
        one_use: OneUse::minted().expect("minted"),
        machine: SigningKey::from_bytes(&[1; 32]),
        giver: SigningKey::from_bytes(&[2; 32]),
        certificate: root.certify(&share.public(), Timestamp::UNIX_EPOCH),
        revoked: root.revoke(&share.public(), Timestamp::UNIX_EPOCH),
        key: CloudKey::from_bytes(&[5; 32]),
        share,
    }
});

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
    let fixed = &*FIXED;
    let ask = Ask::signed(
        &input.device,
        &input.name,
        &fixed.machine,
        &fixed.one_use,
        &input.cloud,
        input.at,
    );
    assert!(ask.holds());
    let mut handed = handing::handed(&ask, &fixed.key, &input.by, &fixed.giver).expect("handed");
    assert!(handed.vouch(&ask, &fixed.share, &fixed.certificate));
    assert!(handed.signed_for(&ask));
    assert_eq!(
        handed
            .opened(&ask, &fixed.one_use)
            .map(|one| *one.to_bytes()),
        Some([5; 32])
    );
    assert!(holds(handed.vouched_for(&ask, &[])));
    assert!(matches!(
        handed.vouched_for(&ask, std::slice::from_ref(&fixed.revoked)),
        Verdict::Revoked { .. }
    ));

    let other = ask_changed(&ask, &input);
    if other != ask {
        assert!(!other.holds());
        assert!(!handed.signed_for(&other));
        assert!(handed.opened(&other, &fixed.one_use).is_none());
        assert!(!holds(handed.vouched_for(&other, &[])));
    }

    let changed = handed_changed(&handed, &input);
    if changed != handed {
        assert!(!changed.signed_for(&ask));
        assert!(changed.opened(&ask, &fixed.one_use).is_none());
        // The vouching signs everything the answer says but its own signature, which it stands in for.
        if changed.sig == handed.sig {
            assert!(!holds(changed.vouched_for(&ask, &[])));
        }
    }
});
