#![no_main]

use std::sync::LazyLock;

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tisty_share_proto::handing::{Ask, Giver, Handed, OneUse, SigningKey};
use tisty_share_proto::identity::Revocation;

struct Asked {
    one_use: OneUse,
    ask: Ask,
}

static ASKED: LazyLock<Asked> = LazyLock::new(|| {
    let one_use = OneUse::minted().expect("minted");
    let machine = SigningKey::from_bytes(&[1; 32]);
    let ask = Ask::signed("device", "machine", &machine, &one_use, "Tisty", 0);
    Asked { one_use, ask }
});

#[derive(Arbitrary, Debug)]
struct Input<'a> {
    ask: &'a [u8],
    giver: &'a [u8],
    handed: &'a [u8],
    revoked: &'a [u8],
    reveal: &'a str,
}

fn again<T: Serialize + DeserializeOwned>(value: &T) -> T {
    let written = serde_json::to_vec(value).expect("written");
    serde_json::from_slice(&written).expect("read back")
}

fuzz_target!(|input: Input| {
    let asked = &*ASKED;
    let giver = SigningKey::from_bytes(&[2; 32]).verifying_key();

    if let Ok(ask) = serde_json::from_slice::<Ask>(input.ask) {
        assert_eq!(again(&ask).holds(), ask.holds());
        if let Some(code) = ask.code(&giver, input.reveal) {
            let groups: Vec<&str> = code.split(' ').collect();
            assert_eq!(groups.len(), 4);
            assert!(
                groups
                    .iter()
                    .all(|five| five.len() == 5 && five.bytes().all(|d| d.is_ascii_digit()))
            );
        }
    }

    if let Ok(read) = serde_json::from_slice::<Giver>(input.giver) {
        assert_eq!(again(&read).key(), read.key());
    }

    let revoked = serde_json::from_slice::<Vec<Revocation>>(input.revoked).unwrap_or_default();
    if let Ok(handed) = serde_json::from_slice::<Handed>(input.handed) {
        let signed = handed.signed_for(&asked.ask);
        let opened = handed
            .opened(&asked.ask, &asked.one_use)
            .map(|key| *key.to_bytes());
        let vouched = handed.vouched_for(&asked.ask, &revoked);
        assert!(opened.is_none() || signed);
        let read = again(&handed);
        assert_eq!(read.signed_for(&asked.ask), signed);
        assert_eq!(
            read.opened(&asked.ask, &asked.one_use)
                .map(|key| *key.to_bytes()),
            opened
        );
        assert_eq!(read.vouched_for(&asked.ask, &revoked), vouched);
    }
});
