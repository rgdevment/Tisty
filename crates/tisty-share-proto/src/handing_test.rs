use super::*;
use crate::seed::Seed;

const CLOUD: &str = "01JSTORE";

fn machine(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn cloud_key() -> CloudKey {
    CloudKey::of(&Seed::from_bytes(&[9; 32]))
}

fn asked(one_use: &OneUse) -> Ask {
    Ask::signed(
        "dev_b",
        "MacBook",
        &machine(2),
        one_use,
        CLOUD,
        1_761_400_000,
    )
}

#[test]
fn the_key_handed_to_an_ask_opens_only_with_that_ask_s_one_use_key() {
    let one_use = OneUse::minted().unwrap();
    let ask = asked(&one_use);
    assert!(ask.holds());

    let handed = handed(&ask, &cloud_key(), "dev_a", &machine(1)).unwrap();

    let got = handed.opened(&ask, &one_use).unwrap();
    assert_eq!(got.named_for(CLOUD), cloud_key().named_for(CLOUD));
    assert_eq!(handed.giver(), Some(machine(1).verifying_key()));
    assert!(handed.opened(&ask, &OneUse::minted().unwrap()).is_none());
}

#[test]
fn an_ask_changed_on_the_way_no_longer_holds() {
    let ask = asked(&OneUse::minted().unwrap());
    let other = OneUse::minted().unwrap();
    for changed in [
        Ask {
            one_use: coded(&other.public()),
            ..ask.clone()
        },
        Ask {
            device: "dev_c".into(),
            ..ask.clone()
        },
        Ask {
            cloud: "01JOTHER".into(),
            ..ask.clone()
        },
        Ask {
            at: ask.at + 1,
            ..ask.clone()
        },
        Ask {
            v: SAID + 1,
            ..ask.clone()
        },
    ] {
        assert!(!changed.holds(), "{changed:?}");
    }
}

#[test]
fn an_answer_changed_on_the_way_or_meant_for_another_ask_never_opens() {
    let one_use = OneUse::minted().unwrap();
    let ask = asked(&one_use);
    let handed = handed(&ask, &cloud_key(), "dev_a", &machine(1)).unwrap();
    let swapped_from = Handed {
        from: coded(&OneUse::minted().unwrap().public()),
        ..handed.clone()
    };
    let resigned = Handed {
        key: coded(machine(3).verifying_key().as_bytes()),
        ..handed.clone()
    };
    let later = Ask::signed(
        "dev_b",
        "MacBook",
        &machine(2),
        &one_use,
        CLOUD,
        ask.at + 60,
    );

    assert!(swapped_from.opened(&ask, &one_use).is_none());
    assert!(resigned.opened(&ask, &one_use).is_none());
    assert!(handed.opened(&later, &one_use).is_none());
}

#[test]
fn a_one_use_key_of_low_order_is_refused_rather_than_answered() {
    let ask = Ask {
        one_use: coded(&[0u8; 32]),
        ..asked(&OneUse::minted().unwrap())
    };

    assert!(handed(&ask, &cloud_key(), "dev_a", &machine(1)).is_none());
}

#[test]
fn both_machines_read_one_code_and_another_ask_or_another_giver_reads_another() {
    let ask = asked(&OneUse::minted().unwrap());
    let carried: Ask = serde_json::from_str(&serde_json::to_string(&ask).unwrap()).unwrap();
    let giver = machine(1).verifying_key();

    let code = ask.code(&giver).unwrap();

    assert_eq!(carried.code(&giver).as_deref(), Some(code.as_str()));
    assert_eq!(code.len(), 23);
    assert!(code.split(' ').all(|five| five.len() == 5));
    assert_ne!(ask.code(&machine(3).verifying_key()), Some(code.clone()));
    assert_ne!(asked(&OneUse::minted().unwrap()).code(&giver), Some(code));
}

#[test]
fn a_giver_carries_the_key_it_names() {
    let giver = Giver::of("dev_a", "Mac", &machine(1).verifying_key());
    let carried: Giver = serde_json::from_str(&serde_json::to_string(&giver).unwrap()).unwrap();

    assert_eq!(carried.key(), Some(machine(1).verifying_key()));
}
