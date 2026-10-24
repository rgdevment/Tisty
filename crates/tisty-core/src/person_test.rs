use super::*;
use crate::event::DeviceKind;

fn room() -> (tempfile::TempDir, crate::Paths) {
    let room = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(room.path().join("data"), room.path().join("config"));
    (room, paths)
}

fn at(second: i64) -> Timestamp {
    Timestamp::from_second(second).unwrap()
}

fn device(name: &str) -> DeviceId {
    DeviceId(name.into())
}

fn joined(name: &str, kind: DeviceKind) -> Event {
    Event::new(
        device(name),
        at(1),
        Op::DeviceJoin {
            d: device(name),
            k: Some(kind),
            p: None,
        },
    )
}

fn said(by: &str, op: Op, second: i64) -> Event {
    Event::new(device(by), at(second), op)
}

fn seed() -> Seed {
    Seed::from_bytes(&[7; 32])
}

/// Two machines of one person: the first holds the seed, the second has been vouched for.
struct Two {
    _rooms: (tempfile::TempDir, tempfile::TempDir),
    first: crate::Paths,
    second: crate::Paths,
    events: Vec<Event>,
}

impl Two {
    fn new() -> Self {
        let (one, first) = room();
        let (two, second) = room();
        let mut two = Self {
            _rooms: (one, two),
            first,
            second,
            events: vec![
                joined("dev_a", DeviceKind::Machine),
                joined("dev_b", DeviceKind::Machine),
            ],
        };
        let ops = begun(
            &two.first,
            &device("dev_a"),
            &seed(),
            &two.state().persons,
            at(100),
        );
        two.wrote("dev_a", ops.unwrap(), 100);
        let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
        two.wrote("dev_b", vec![op], 110);
        two.vouch(120);
        two
    }

    fn state(&self) -> State {
        State::replay(&self.events)
    }

    fn wrote(&mut self, by: &str, ops: Vec<Op>, second: i64) {
        self.events
            .extend(ops.into_iter().map(|op| said(by, op, second)));
    }

    fn vouch(&mut self, second: i64) -> usize {
        let state = self.state();
        let ops = certified(&seed(), &state.persons, &machines(&state), at(second));
        let made = ops.len();
        self.wrote("dev_a", ops, second);
        made
    }
}

#[test]
fn both_machines_of_one_person_name_the_same_person() {
    let two = Two::new();
    let state = two.state();

    assert_eq!(
        state.persons.person(&device("dev_a")),
        Some(seed().root().person())
    );
    assert_eq!(
        state.persons.person(&device("dev_b")),
        Some(seed().root().person())
    );
}

#[test]
fn a_machine_says_no_key_until_the_log_names_a_person() {
    let (_room, paths) = room();

    assert!(introduced(&paths, &device("dev_b"), &Persons::default()).is_none());
    assert!(!kept_at(&paths, &device("dev_b")).unwrap().exists());
}

#[test]
fn a_key_already_said_is_not_said_again_and_nothing_waits_twice() {
    let mut two = Two::new();

    assert!(introduced(&two.second, &device("dev_b"), &two.state().persons).is_none());
    assert_eq!(two.vouch(130), 0);
}

#[test]
fn assistants_never_get_a_key_nor_speak_for_the_person() {
    let mut two = Two::new();
    two.events.push(joined("dev_bot", DeviceKind::Agent));
    let (_room, bot) = room();
    let key = mine(&bot, &device("dev_bot")).unwrap();
    let stranger = Seed::from_bytes(&[9; 32]).root();
    let taken = stranger.revoke(
        &mine(&two.first, &device("dev_a")).unwrap().public(),
        at(140),
    );
    two.wrote(
        "dev_bot",
        vec![
            Op::ShareKey { p: key.shown() },
            Op::ShareCertified {
                c: seed().root().certify(&key.public(), at(140)),
            },
            Op::ShareRevoked { r: taken },
        ],
        140,
    );

    let state = two.state();
    assert!(!state.persons.keys.contains_key(&device("dev_bot")));
    assert_eq!(state.persons.revocations.len(), 0);
    assert_eq!(state.persons.person(&device("dev_bot")), None);
    assert_eq!(
        two.vouch(150),
        0,
        "an assistant was waiting to be vouched for"
    );
}

#[test]
fn a_machine_taken_back_is_never_vouched_for_again_on_its_own() {
    let mut two = Two::new();
    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(200));
    two.wrote("dev_a", ops, 200);
    assert_eq!(two.state().persons.person(&device("dev_b")), None);
    assert_eq!(
        two.state().persons.person(&device("dev_a")),
        Some(seed().root().person()),
        "taking one machine back left the other alone"
    );

    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 210);

    assert_eq!(
        two.vouch(220),
        0,
        "the machine taken back was vouched for again"
    );
    assert_eq!(two.state().persons.person(&device("dev_b")), None);
}

#[test]
fn typing_the_phrase_on_a_machine_taken_back_brings_it_in_with_a_new_key() {
    let mut two = Two::new();
    let old = mine(&two.second, &device("dev_b")).unwrap().public();
    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(200));
    two.wrote("dev_a", ops, 200);

    let ops = begun(
        &two.second,
        &device("dev_b"),
        &seed(),
        &two.state().persons,
        at(210),
    )
    .unwrap();
    two.wrote("dev_b", ops, 210);

    let fresh = mine(&two.second, &device("dev_b")).unwrap().public();
    assert_ne!(fresh, old);
    assert_eq!(
        two.state().persons.person(&device("dev_b")),
        Some(seed().root().person())
    );
}

#[test]
fn beginning_again_where_everything_holds_says_nothing() {
    let two = Two::new();

    let ops = begun(
        &two.first,
        &device("dev_a"),
        &seed(),
        &two.state().persons,
        at(300),
    )
    .unwrap();

    assert!(ops.is_empty(), "{ops:?}");
}

#[test]
fn another_root_vouching_later_does_not_stop_the_person_taking_a_machine_back() {
    let mut two = Two::new();
    let other = Seed::from_bytes(&[9; 32]).root();
    let key = mine(&two.second, &device("dev_b")).unwrap().public();
    two.wrote(
        "dev_b",
        vec![Op::ShareCertified {
            c: other.certify(&key, at(500)),
        }],
        500,
    );

    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(600));

    assert!(
        !ops.is_empty(),
        "the person could not take their own machine back"
    );
}

#[test]
fn only_the_person_who_vouched_can_take_a_machine_back() {
    let two = Two::new();
    let stranger = Seed::from_bytes(&[9; 32]);

    assert!(revoked(&stranger, &two.state().persons, &device("dev_a"), at(200)).is_empty());
}

#[test]
fn a_key_another_machine_already_named_is_not_taken() {
    let mut two = Two::new();
    two.events.push(joined("dev_c", DeviceKind::Machine));
    let borrowed = mine(&two.first, &device("dev_a")).unwrap().shown();
    two.wrote("dev_c", vec![Op::ShareKey { p: borrowed }], 300);

    let state = two.state();
    assert!(!state.persons.keys.contains_key(&device("dev_c")));
    assert_eq!(state.persons.person(&device("dev_c")), None);
}

#[test]
fn what_does_not_hold_is_not_kept() {
    let mut certificate = seed()
        .root()
        .certify(&ShareKey::from_bytes(&[3; 32]).public(), at(100));
    certificate.since = at(5);
    let mut revocation = seed()
        .root()
        .revoke(&ShareKey::from_bytes(&[3; 32]).public(), at(100));
    revocation.at = at(5);
    let state = State::replay(&[
        said(
            "dev_a",
            Op::ShareKey {
                p: "not a key".into(),
            },
            100,
        ),
        said("dev_a", Op::ShareCertified { c: certificate }, 100),
        said("dev_a", Op::ShareRevoked { r: revocation }, 100),
    ]);

    assert_eq!(state.persons, Persons::default());
    let (_room, paths) = room();
    assert!(
        introduced(&paths, &device("dev_b"), &state.persons).is_none(),
        "a forged certificate made a machine mint a key"
    );
}

#[test]
fn the_share_key_never_lands_anywhere_that_travels() {
    let (_room, paths) = room();

    mine(&paths, &device("dev_a")).unwrap();

    let at = kept_at(&paths, &device("dev_a")).unwrap();
    assert!(at.is_file());
    assert!(at.starts_with(paths.private()));
}

#[test]
fn the_new_ops_are_optional_and_settle() {
    let (_room, paths) = room();
    for op in begun(
        &paths,
        &device("dev_a"),
        &seed(),
        &Persons::default(),
        at(100),
    )
    .unwrap()
    {
        assert!(op.is_optional(), "{op:?}");
        assert!(op.settles(), "{op:?}");
        assert!(!op.destroys(), "{op:?}");
    }
}

#[test]
fn typing_the_phrase_again_on_a_machine_brought_back_keeps_its_key() {
    let mut two = Two::new();
    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(200));
    two.wrote("dev_a", ops, 200);
    let ops = begun(
        &two.second,
        &device("dev_b"),
        &seed(),
        &two.state().persons,
        at(210),
    )
    .unwrap();
    two.wrote("dev_b", ops, 210);
    let brought = mine(&two.second, &device("dev_b")).unwrap().public();

    let again = begun(
        &two.second,
        &device("dev_b"),
        &seed(),
        &two.state().persons,
        at(220),
    )
    .unwrap();

    assert!(again.is_empty(), "{again:?}");
    assert_eq!(
        mine(&two.second, &device("dev_b")).unwrap().public(),
        brought
    );
}

#[test]
fn a_machine_brought_back_that_loses_its_key_is_vouched_for_again_on_its_own() {
    let mut two = Two::new();
    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(200));
    two.wrote("dev_a", ops, 200);
    let ops = begun(
        &two.second,
        &device("dev_b"),
        &seed(),
        &two.state().persons,
        at(210),
    )
    .unwrap();
    two.wrote("dev_b", ops, 210);

    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 220);

    assert_eq!(two.vouch(230), 1);
    assert_eq!(
        two.state().persons.person(&device("dev_b")),
        Some(seed().root().person())
    );
}

#[test]
fn taking_a_machine_back_takes_back_every_key_it_ever_had() {
    let mut two = Two::new();
    let old = mine(&two.second, &device("dev_b")).unwrap().public();
    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 130);
    assert_eq!(two.vouch(140), 1);

    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(200));
    assert_eq!(ops.len(), 2, "an older key was left signing");
    two.wrote("dev_a", ops, 200);

    let persons = two.state().persons;
    assert!(persons.key_taken_back_by(&old, &seed().root().person()));
    assert!(revoked(&seed(), &persons, &device("dev_b"), at(300)).is_empty());
}

#[test]
fn a_machine_removed_from_the_store_answers_for_nobody() {
    let mut two = Two::new();
    two.wrote("dev_a", vec![Op::DeviceRemove { d: device("dev_b") }], 300);

    let state = two.state();
    assert_eq!(person_of(&state, &device("dev_b")), None);
    assert_eq!(
        person_of(&state, &device("dev_a")),
        Some(seed().root().person())
    );
}

#[test]
fn the_same_certificate_said_twice_is_kept_once() {
    let mut two = Two::new();
    let again = two.state().persons.certificates[0].clone();
    two.wrote("dev_b", vec![Op::ShareCertified { c: again }], 300);

    assert_eq!(two.state().persons.certificates.len(), 2);
}
