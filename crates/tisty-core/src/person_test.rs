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
            Op::ShareRevoked {
                r: taken,
                machine: true,
            },
        ],
        140,
    );

    let state = two.state();
    assert!(!state.persons.keys.contains_key(&device("dev_bot")));
    assert!(
        state.persons.taken.is_empty(),
        "an assistant took a machine back"
    );
    assert_eq!(
        state.persons.person(&device("dev_a")),
        Some(seed().root().person())
    );
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
    let mut two = Two::new();
    let stranger = Seed::from_bytes(&[9; 32]);
    let ops = revoked(&stranger, &two.state().persons, &device("dev_a"), at(200));
    two.wrote("dev_b", ops, 200);

    let persons = two.state().persons;
    assert_eq!(
        persons.person(&device("dev_a")),
        Some(seed().root().person())
    );
    assert!(!persons.taken_back_by(&device("dev_a"), &seed().root().person()));
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
        said(
            "dev_a",
            Op::ShareRevoked {
                r: revocation,
                machine: true,
            },
            100,
        ),
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
fn a_machine_brought_back_that_loses_its_key_needs_the_phrase_again() {
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

    assert_eq!(
        two.vouch(230),
        0,
        "a machine once taken back came in without the phrase"
    );
    let ops = begun(
        &two.second,
        &device("dev_b"),
        &seed(),
        &two.state().persons,
        at(240),
    )
    .unwrap();
    two.wrote("dev_b", ops, 240);
    assert_eq!(
        two.state().persons.person(&device("dev_b")),
        Some(seed().root().person())
    );
}

#[test]
fn a_clock_ahead_on_another_machine_does_not_let_a_machine_taken_back_in() {
    let mut two = Two::new();
    let key = mine(&two.second, &device("dev_b")).unwrap().public();
    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(200));
    two.wrote("dev_a", ops, 200);
    two.wrote(
        "dev_a",
        vec![Op::ShareCertified {
            c: seed().root().certify(&key, at(900)),
        }],
        900,
    );
    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 910);

    assert_eq!(two.vouch(920), 0);
    assert_eq!(two.state().persons.person(&device("dev_b")), None);
}

#[test]
fn a_key_said_again_does_not_come_back_ahead_of_a_newer_one() {
    let mut two = Two::new();
    let old = mine(&two.second, &device("dev_b")).unwrap().shown();
    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 130);
    let newer = mine(&two.second, &device("dev_b")).unwrap().shown();

    two.wrote("dev_b", vec![Op::ShareKey { p: old }], 140);

    assert_eq!(
        two.state().persons.keys[&device("dev_b")].last(),
        Some(&newer)
    );
}

#[test]
fn taking_a_machine_back_takes_back_every_key_it_ever_had() {
    let mut two = Two::new();
    let old = mine(&two.second, &device("dev_b")).unwrap().public();
    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 130);
    assert_eq!(
        two.vouch(140),
        2,
        "the new key was vouched for and the old one retired"
    );

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

#[test]
fn a_machine_taken_back_before_the_root_vouched_for_it_stays_out() {
    let mut two = Two::new();
    two.events.push(joined("dev_c", DeviceKind::Machine));
    let (_room, third) = room();
    let op = introduced(&third, &device("dev_c"), &two.state().persons).unwrap();
    two.wrote("dev_c", vec![op], 300);

    let ops = revoked(&seed(), &two.state().persons, &device("dev_c"), at(310));
    assert_eq!(ops.len(), 1);
    two.wrote("dev_a", ops, 310);

    assert_eq!(two.vouch(320), 0);
    assert_eq!(two.state().persons.person(&device("dev_c")), None);
}

#[test]
fn a_machine_that_only_changed_its_key_keeps_coming_back_on_its_own() {
    let mut two = Two::new();
    for (second, retired) in [(130, 2), (150, 2)] {
        std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
        let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
        two.wrote("dev_b", vec![op], second);
        assert_eq!(two.vouch(second + 5), retired);
    }

    assert_eq!(
        two.state().persons.person(&device("dev_b")),
        Some(seed().root().person())
    );
}

#[test]
fn typing_the_phrase_where_the_key_was_named_by_another_makes_a_new_one() {
    let mut two = Two::new();
    two.events.push(joined("dev_c", DeviceKind::Machine));
    let (_room, third) = room();
    let borrowed = mine(&two.second, &device("dev_b")).unwrap();
    std::fs::create_dir_all(third.private()).unwrap();
    std::fs::write(
        kept_at(&third, &device("dev_c")).unwrap(),
        *borrowed.to_bytes(),
    )
    .unwrap();

    let ops = begun(
        &third,
        &device("dev_c"),
        &seed(),
        &two.state().persons,
        at(300),
    )
    .unwrap();
    two.wrote("dev_c", ops, 300);

    assert_ne!(
        mine(&third, &device("dev_c")).unwrap().public(),
        borrowed.public()
    );
    assert_eq!(
        two.state().persons.person(&device("dev_c")),
        Some(seed().root().person())
    );
}

#[test]
fn a_machine_the_log_never_seated_is_not_heard() {
    let two = Two::new();
    let mut events = two.events.clone();
    let (_room, stray) = room();
    let key = mine(&stray, &device("dev_x")).unwrap();
    events.push(said("dev_x", Op::ShareKey { p: key.shown() }, 300));

    assert!(
        !State::replay(&events)
            .persons
            .keys
            .contains_key(&device("dev_x"))
    );
}

#[test]
fn typing_the_phrase_on_a_machine_with_a_new_key_retires_the_old_one() {
    let mut two = Two::new();
    let old = mine(&two.second, &device("dev_b")).unwrap().public();
    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();

    let ops = begun(
        &two.second,
        &device("dev_b"),
        &seed(),
        &two.state().persons,
        at(300),
    )
    .unwrap();
    two.wrote("dev_b", ops, 300);

    let persons = two.state().persons;
    assert!(persons.key_taken_back_by(&old, &seed().root().person()));
    assert_eq!(
        persons.person(&device("dev_b")),
        Some(seed().root().person())
    );
}

#[test]
fn an_old_key_brought_back_from_a_backup_is_never_said_again() {
    let mut two = Two::new();
    let at_b = kept_at(&two.second, &device("dev_b")).unwrap();
    let backup = std::fs::read(&at_b).unwrap();
    std::fs::remove_file(&at_b).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 130);
    two.vouch(140);

    std::fs::write(&at_b, backup).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 150);

    let state = two.state();
    let current = state.persons.keys[&device("dev_b")].last().cloned();
    assert_eq!(
        current,
        mine(&two.second, &device("dev_b")).map(|key| key.shown())
    );
    assert_eq!(state.persons.keys[&device("dev_b")].len(), 3);
}

#[test]
fn a_revocation_carried_by_a_machine_removed_since_still_holds() {
    let mut two = Two::new();
    let ops = revoked(&seed(), &two.state().persons, &device("dev_b"), at(200));
    let mut events = two.events.clone();
    events.push(said("dev_a", Op::DeviceRemove { d: device("dev_a") }, 190));
    events.extend(ops.into_iter().map(|op| said("dev_a", op, 200)));
    two.events = events;

    assert_eq!(two.state().persons.person(&device("dev_b")), None);
    assert!(
        two.state()
            .persons
            .taken_back_by(&device("dev_b"), &seed().root().person())
    );
}

#[test]
fn a_machine_removed_without_the_seed_is_taken_back_by_the_one_that_has_it() {
    let mut two = Two::new();
    two.wrote("dev_b", vec![Op::DeviceRemove { d: device("dev_b") }], 300);
    let state = two.state();

    let ops = left(&seed(), &state.persons, &state.devices, at(310));
    assert_eq!(ops.len(), 1);
    two.wrote("dev_a", ops, 310);

    let state = two.state();
    assert_eq!(state.persons.person(&device("dev_b")), None);
    assert!(left(&seed(), &state.persons, &state.devices, at(320)).is_empty());
}

#[test]
fn the_log_names_the_person_and_no_other() {
    let two = Two::new();
    let persons = two.state().persons;

    assert!(persons.names_a_person());
    assert!(persons.names(&seed().root().person()));
    assert!(!persons.names(&Seed::from_bytes(&[9; 32]).root().person()));
    assert!(!Persons::default().names_a_person());
}

#[test]
fn the_seed_is_only_wanted_when_there_is_something_to_do() {
    let mut two = Two::new();
    let state = two.state();
    assert!(!state.persons.awaits_the_seed(&state.devices));

    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &state.persons).unwrap();
    two.wrote("dev_b", vec![op], 130);
    let state = two.state();
    assert!(
        state.persons.awaits_the_seed(&state.devices),
        "a new key waits"
    );
    two.vouch(140);
    let state = two.state();
    assert!(!state.persons.awaits_the_seed(&state.devices));

    two.wrote("dev_b", vec![Op::DeviceRemove { d: device("dev_b") }], 150);
    let state = two.state();
    assert!(
        state.persons.awaits_the_seed(&state.devices),
        "a machine gone waits"
    );
}

#[test]
fn a_machine_gone_with_an_older_key_still_standing_waits_for_the_seed() {
    let mut two = Two::new();
    std::fs::remove_file(kept_at(&two.second, &device("dev_b")).unwrap()).unwrap();
    let op = introduced(&two.second, &device("dev_b"), &two.state().persons).unwrap();
    two.wrote("dev_b", vec![op], 130);
    two.wrote("dev_b", vec![Op::DeviceRemove { d: device("dev_b") }], 140);

    let state = two.state();
    assert!(state.persons.awaits_the_seed(&state.devices));
    let ops = left(&seed(), &state.persons, &state.devices, at(150));
    assert!(!ops.is_empty(), "the older key was left standing");
}

fn runs(version: &str) -> Op {
    Op::DeviceRelease { r: version.into() }
}

#[test]
fn a_machine_says_the_version_it_runs_and_the_last_word_stands() {
    let events = vec![
        joined("dev_a", DeviceKind::Machine),
        joined("dev_b", DeviceKind::Machine),
        said("dev_a", runs("1.33.0"), 100),
        said("dev_b", runs("1.34.0"), 100),
        said("dev_a", runs("1.34.0"), 200),
    ];

    let state = State::replay(&events);

    assert_eq!(state.runs.get(&device("dev_a")).unwrap(), "1.34.0");
    assert_eq!(state.runs.get(&device("dev_b")).unwrap(), "1.34.0");
}

#[test]
fn a_version_is_only_taken_from_a_machine_on_the_list_and_goes_when_it_does() {
    let mut events = vec![
        joined("dev_a", DeviceKind::Machine),
        said("dev_stranger", runs("9.9.9"), 100),
        said("dev_a", runs("   "), 100),
        said("dev_a", runs("1.34.0"), 110),
    ];

    assert!(
        !State::replay(&events)
            .runs
            .contains_key(&device("dev_stranger"))
    );
    assert_eq!(State::replay(&events).runs.len(), 1);

    events.push(said("dev_a", Op::DeviceRemove { d: device("dev_a") }, 120));
    assert!(State::replay(&events).runs.is_empty());
}

#[test]
fn the_version_op_is_optional_and_settles() {
    let op = runs("1.34.0");

    assert!(op.is_optional());
    assert!(op.settles());
    assert!(!op.destroys());
}

#[test]
fn a_machine_says_its_version_only_when_it_is_on_the_list_and_it_changed() {
    let mut events = vec![joined("dev_a", DeviceKind::Machine)];

    let state = State::replay(&events);
    assert!(state.runs_to_say(&device("dev_x"), "1.34.0").is_none());
    assert_eq!(
        state.runs_to_say(&device("dev_a"), "1.34.0"),
        Some(runs("1.34.0"))
    );

    events.push(said("dev_a", runs("1.34.0"), 100));
    let state = State::replay(&events);
    assert!(state.runs_to_say(&device("dev_a"), "1.34.0").is_none());
    assert_eq!(
        state.runs_to_say(&device("dev_a"), "1.35.0"),
        Some(runs("1.35.0"))
    );
}

#[test]
fn a_version_the_log_would_clean_is_not_said_again_every_round() {
    let long = format!("1.34.0-{}", "x".repeat(100));
    let mut events = vec![joined("dev_a", DeviceKind::Machine)];
    let Some(Op::DeviceRelease { r }) = State::replay(&events).runs_to_say(&device("dev_a"), &long)
    else {
        panic!("a machine on the list says its version");
    };
    events.push(said("dev_a", Op::DeviceRelease { r }, 100));

    assert!(
        State::replay(&events)
            .runs_to_say(&device("dev_a"), &long)
            .is_none()
    );
}

fn sealed(store: &str, key: Option<&str>) -> Op {
    Op::CloudSealed {
        s: store.into(),
        k: key.map(Into::into),
    }
}

#[test]
fn a_machine_on_the_list_says_a_store_is_sealed_and_the_last_word_stands() {
    let mut events = vec![
        joined("dev_a", DeviceKind::Machine),
        said("dev_a", sealed("store_one", Some("key_a")), 100),
        said("dev_a", sealed("store_two", Some("key_b")), 110),
    ];

    let state = State::replay(&events);
    assert_eq!(state.sealed.get("store_one").unwrap(), "key_a");
    assert_eq!(state.sealed.len(), 2);

    events.push(said("dev_a", sealed("store_one", None), 200));
    let state = State::replay(&events);
    assert!(!state.sealed.contains_key("store_one"));
    assert_eq!(state.sealed.get("store_two").unwrap(), "key_b");
}

#[test]
fn only_a_machine_on_the_list_can_say_a_store_is_sealed() {
    let events = vec![
        joined("dev_a", DeviceKind::Machine),
        joined("dev_bot", DeviceKind::Agent),
        said("dev_stranger", sealed("store_one", Some("key_x")), 100),
        said("dev_bot", sealed("store_one", Some("key_y")), 110),
        said("dev_a", sealed("", Some("key_z")), 120),
    ];

    assert!(State::replay(&events).sealed.is_empty());
}

#[test]
fn the_sealed_op_is_optional_and_settles() {
    let op = sealed("store_one", Some("key_a"));

    assert!(op.is_optional());
    assert!(op.settles());
    assert!(!op.destroys());
    assert_eq!(
        serde_json::to_value(sealed("store_one", None)).unwrap(),
        serde_json::json!({"op": "cloud.sealed", "s": "store_one"})
    );
}
