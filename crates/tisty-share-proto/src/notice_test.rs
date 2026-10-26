use super::*;

fn machine(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn notice() -> Unsealed {
    Unsealed::signed("store_one", "named", "dev_a", &machine(1), 1_761_400_000)
}

#[test]
fn a_notice_holds_whole_and_carries_its_key() {
    let said = notice();
    let carried: Unsealed = serde_json::from_str(&serde_json::to_string(&said).unwrap()).unwrap();

    assert!(carried.holds());
    assert_eq!(carried.key(), Some(machine(1).verifying_key()));
}

#[test]
fn a_notice_changed_on_the_way_no_longer_holds() {
    let said = notice();
    for changed in [
        Unsealed {
            cloud: "store_two".into(),
            ..said.clone()
        },
        Unsealed {
            named: "other".into(),
            ..said.clone()
        },
        Unsealed {
            by: "dev_b".into(),
            ..said.clone()
        },
        Unsealed {
            at: said.at + 1,
            ..said.clone()
        },
        Unsealed {
            key: Unsealed::signed("store_one", "named", "dev_a", &machine(2), said.at).key,
            ..said.clone()
        },
        Unsealed {
            v: SAID + 1,
            ..said.clone()
        },
    ] {
        assert!(!changed.holds(), "{changed:?}");
    }
}
