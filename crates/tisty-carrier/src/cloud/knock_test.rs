use super::*;
use crate::fakes::Fake;
use tisty_share_proto::cloud::CloudKey;
use tisty_share_proto::handing::{OneUse, SigningKey, handed};
use tisty_share_proto::seed::Seed;

const CLOUD: &str = "store_one";

fn machine(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn asked(device: &str, one_use: &OneUse) -> Ask {
    Ask::signed(
        device,
        "MacBook",
        &machine(2),
        one_use,
        CLOUD,
        1_761_400_000,
    )
}

fn put(remote: &dyn Remote, name: &str, body: &[u8]) {
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path().join("body");
    std::fs::write(&at, body).unwrap();
    remote.put(name, &at, Expect::Any).unwrap();
}

#[test]
fn an_ask_is_found_answered_once_and_taken_away_whole() {
    let (fake, home) = (Fake::drive(), tempfile::tempdir().unwrap());
    let one_use = OneUse::minted().unwrap();
    let ask = asked("dev_b", &one_use);
    let key = CloudKey::of(&Seed::from_bytes(&[9; 32]));

    knocked(&fake, home.path(), &ask).unwrap();
    assert_eq!(
        knocks(&fake, home.path()).unwrap(),
        vec![Knock {
            device: "dev_b".into(),
            ask: Some(ask.clone()),
        }]
    );
    let first = handed(&ask, &key, "dev_a", &machine(1)).unwrap();
    let second = handed(&ask, &key, "dev_c", &machine(3)).unwrap();
    answer(&fake, home.path(), "dev_b", &first).unwrap();
    answer(&fake, home.path(), "dev_b", &second).unwrap();

    assert_eq!(answered(&fake, home.path(), "dev_b").unwrap(), Some(first));
    assert_eq!(knocks(&fake, home.path()).unwrap()[0].ask, None);
    withdrawn(&fake, home.path(), "dev_b").unwrap();
    assert!(knocks(&fake, home.path()).unwrap().is_empty());
    assert_eq!(answered(&fake, home.path(), "dev_b").unwrap(), None);
}

#[test]
fn asking_again_takes_away_the_answer_to_the_ask_before() {
    let (fake, home) = (Fake::drive(), tempfile::tempdir().unwrap());
    let old = asked("dev_b", &OneUse::minted().unwrap());
    knocked(&fake, home.path(), &old).unwrap();
    let key = CloudKey::of(&Seed::from_bytes(&[9; 32]));
    answer(
        &fake,
        home.path(),
        "dev_b",
        &handed(&old, &key, "dev_a", &machine(1)).unwrap(),
    )
    .unwrap();

    knocked(
        &fake,
        home.path(),
        &asked("dev_b", &OneUse::minted().unwrap()),
    )
    .unwrap();

    assert_eq!(answered(&fake, home.path(), "dev_b").unwrap(), None);
}

#[test]
fn what_is_not_a_whole_ask_for_the_machine_it_is_named_for_is_passed_over() {
    let (fake, home) = (Fake::drive(), tempfile::tempdir().unwrap());
    let theirs = serde_json::to_vec(&asked("dev_c", &OneUse::minted().unwrap())).unwrap();
    put(&fake, "knock/dev_b.ask", &theirs);
    put(&fake, "knock/dev_d.ask", b"not json");
    put(&fake, "knock/dev_e.ask", &vec![b' '; 5000]);
    put(&fake, "knock/notes.txt", b"hello");

    assert!(knocks(&fake, home.path()).unwrap().is_empty());
    assert!(
        knocked(
            &fake,
            home.path(),
            &asked("../store", &OneUse::minted().unwrap())
        )
        .is_err()
    );
}

#[test]
fn a_key_brought_by_a_knock_is_taken_only_if_it_opens_the_cloud_s_mark() {
    let (fake, home) = (
        std::sync::Arc::new(Fake::drive()),
        tempfile::tempdir().unwrap(),
    );
    let key = CloudKey::of(&Seed::from_bytes(&[9; 32]));
    crate::Sealed::new(fake.clone(), key.clone(), CLOUD)
        .mark_it()
        .unwrap();
    let one_use = OneUse::minted().unwrap();
    let ask = asked("dev_b", &one_use);
    knocked(&*fake, home.path(), &ask).unwrap();
    let forged = CloudKey::of(&Seed::from_bytes(&[4; 32]));

    answer(
        &*fake,
        home.path(),
        "dev_b",
        &handed(&ask, &key, "dev_a", &machine(1)).unwrap(),
    )
    .unwrap();

    let got = answered(&*fake, home.path(), "dev_b")
        .unwrap()
        .and_then(|answer| answer.opened(&ask, &one_use))
        .unwrap();
    assert!(opens(&*fake, home.path(), &got, CLOUD));
    assert!(!opens(&*fake, home.path(), &forged, CLOUD));
    assert!(!opens(&*fake, home.path(), &got, "store_two"));
}

#[test]
fn an_answer_that_cannot_be_read_is_taken_away_and_one_ask_unread_never_hides_the_others() {
    let (fake, home) = (Fake::drive(), tempfile::tempdir().unwrap());
    let ask = asked("dev_b", &OneUse::minted().unwrap());
    knocked(&fake, home.path(), &ask).unwrap();
    put(&fake, "knock/dev_b.key", b"{ not an answer");
    put(&fake, "knock/dev_c.ask", &vec![b' '; 5000]);

    assert_eq!(answered(&fake, home.path(), "dev_b").unwrap(), None);
    assert!(knocks(&fake, home.path()).unwrap()[0].ask.is_some());
    assert_eq!(knocks(&fake, home.path()).unwrap().len(), 1);
}
