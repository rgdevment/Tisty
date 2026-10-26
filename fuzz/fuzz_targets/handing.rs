#![no_main]

use std::sync::LazyLock;

use arbitrary::Arbitrary;
use jiff::Timestamp;
use libfuzzer_sys::fuzz_target;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tisty_share_proto::VerifyingKey;
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{self, Ask, Giver, Handed, OneUse, SigningKey};
use tisty_share_proto::identity::{Revocation, Root, ShareKey, Verdict};

const KEY: [u8; 32] = [5; 32];

struct Genuine {
    one_use: OneUse,
    by: VerifyingKey,
    ask: Ask,
    handed: Handed,
    asked: Vec<u8>,
    given: Vec<u8>,
    answered: Vec<u8>,
}

static GENUINE: LazyLock<Genuine> = LazyLock::new(|| {
    let one_use = OneUse::minted().expect("minted");
    let machine = SigningKey::from_bytes(&[1; 32]);
    let signer = SigningKey::from_bytes(&[2; 32]);
    let share = ShareKey::from_bytes(&[3; 32]);
    let root = Root::from_bytes(&[4; 32]);
    let certificate = root.certify(&share.public(), Timestamp::UNIX_EPOCH);
    let ask = Ask::signed("device", "machine", &machine, &one_use, "Tisty", 0);
    let giver = Giver::of("giver", "desk", &signer.verifying_key());
    let mut handed =
        handing::handed(&ask, &CloudKey::from_bytes(&KEY), "giver", &signer).expect("handed");
    assert!(handed.vouch(&ask, &share, &certificate));
    Genuine {
        asked: serde_json::to_vec(&ask).expect("written"),
        given: serde_json::to_vec(&giver).expect("written"),
        answered: serde_json::to_vec(&handed).expect("written"),
        by: signer.verifying_key(),
        one_use,
        ask,
        handed,
    }
});

#[derive(Arbitrary, Debug)]
enum Edit {
    Flip { at: usize, by: u8 },
    Put { at: usize, byte: u8 },
    Drop { at: usize },
}

#[derive(Arbitrary, Debug)]
enum Said<'a> {
    Edited(Vec<Edit>),
    Anything(&'a [u8]),
}

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    ask: Said<'a>,
    giver: Said<'a>,
    handed: Said<'a>,
    revoked: &'a [u8],
    reveal: &'a str,
}

fn said(genuine: &[u8], said: &Said) -> Vec<u8> {
    let edits = match said {
        Said::Anything(bytes) => return bytes.to_vec(),
        Said::Edited(edits) => edits,
    };
    let mut out = genuine.to_vec();
    for edit in edits {
        match *edit {
            Edit::Flip { at, by } if !out.is_empty() => {
                let at = at % out.len();
                out[at] ^= by.max(1);
            }
            Edit::Put { at, byte } => out.insert(at % (out.len() + 1), byte),
            Edit::Drop { at } if !out.is_empty() => {
                out.remove(at % out.len());
            }
            _ => {}
        }
    }
    out
}

fn again<T: Serialize + DeserializeOwned>(value: &T) -> T {
    let written = serde_json::to_vec(value).expect("written");
    serde_json::from_slice(&written).expect("read back")
}

fn says_alike(one: &Handed, other: &Handed) -> bool {
    (&one.v, &one.by, &one.key, &one.from, &one.sealed)
        == (&other.v, &other.by, &other.key, &other.from, &other.sealed)
}

fn holds(verdict: Verdict) -> bool {
    matches!(verdict, Verdict::Holds { .. })
}

fuzz_target!(|input: Input| {
    let genuine = &*GENUINE;

    if let Ok(ask) = serde_json::from_slice::<Ask>(&said(&genuine.asked, &input.ask)) {
        assert_eq!(again(&ask), ask);
        if ask.holds() && ask.key == genuine.ask.key {
            assert_eq!(ask, genuine.ask);
        }
        if ask.commit == genuine.ask.commit {
            assert_eq!(ask.code(&genuine.by, input.reveal), None);
        }
    }

    if let Ok(read) = serde_json::from_slice::<Giver>(&said(&genuine.given, &input.giver)) {
        assert_eq!(again(&read), read);
        assert_eq!(read.key(), again(&read).key());
    }

    let revoked = serde_json::from_slice::<Vec<Revocation>>(input.revoked).unwrap_or_default();
    if let Ok(handed) = serde_json::from_slice::<Handed>(&said(&genuine.answered, &input.handed)) {
        assert_eq!(again(&handed), handed);
        let signed = handed.signed_for(&genuine.ask);
        let opened = handed
            .opened(&genuine.ask, &genuine.one_use)
            .map(|key| *key.to_bytes());
        assert!(opened.is_none() || signed);
        if handed.key == genuine.handed.key {
            let alike = says_alike(&handed, &genuine.handed) && handed.sig == genuine.handed.sig;
            assert_eq!(signed, alike);
            assert_eq!(opened, alike.then_some(KEY));
        }
        let vouched = holds(handed.vouched_for(&genuine.ask, &revoked));
        if vouched && handed.vouched == genuine.handed.vouched {
            assert!(says_alike(&handed, &genuine.handed));
        }
    }
});
