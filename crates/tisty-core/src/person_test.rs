use super::*;
use crate::event::{DeviceKind, Event};

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

#[test]
fn the_machine_where_the_seed_is_born_vouches_for_itself() {
    let (_room, paths) = room();
    let ops = begun(&paths, &device("dev_a"), &seed(), at(100)).unwrap();
    let mut events = vec![joined("dev_a", DeviceKind::Machine)];
    events.extend(ops.into_iter().map(|op| said("dev_a", op, 100)));

    let state = State::replay(&events);

    assert_eq!(
        state.persons.person(&device("dev_a")),
        Some(seed().root().person())
    );
}

#[test]
fn a_machine_says_its_key_only_once_the_log_names_a_person() {
    let (_room, paths) = room();
    assert!(introduced(&paths, &device("dev_b"), &Persons::default()).is_none());

    let (_other, born) = room();
    let ops = begun(&born, &device("dev_a"), &seed(), at(100)).unwrap();
    let state = State::replay(
        &ops.into_iter()
            .map(|op| said("dev_a", op, 100))
            .collect::<Vec<_>>(),
    );

    let Some(Op::ShareKey { p }) = introduced(&paths, &device("dev_b"), &state.persons) else {
        panic!("the second machine said no key");
    };
    assert_eq!(
        Some(p),
        mine(&paths, &device("dev_b")).map(|key| key.shown())
    );
}

#[test]
fn a_key_already_said_is_not_said_again() {
    let (_room, paths) = room();
    let mut events: Vec<Event> = begun(&paths, &device("dev_a"), &seed(), at(100))
        .unwrap()
        .into_iter()
        .map(|op| said("dev_a", op, 100))
        .collect();
    let (_other, second) = room();
    let state = State::replay(&events);
    let op = introduced(&second, &device("dev_b"), &state.persons).unwrap();
    events.push(said("dev_b", op, 110));

    let state = State::replay(&events);

    assert!(introduced(&second, &device("dev_b"), &state.persons).is_none());
}

#[test]
fn the_root_vouches_for_every_machine_waiting_and_skips_assistants() {
    let (_a, first) = room();
    let (_b, second) = room();
    let (_c, agent) = room();
    let mut events = vec![
        joined("dev_a", DeviceKind::Machine),
        joined("dev_b", DeviceKind::Machine),
        joined("dev_bot", DeviceKind::Agent),
    ];
    events.extend(
        begun(&first, &device("dev_a"), &seed(), at(100))
            .unwrap()
            .into_iter()
            .map(|op| said("dev_a", op, 100)),
    );
    let state = State::replay(&events);
    events.push(said(
        "dev_b",
        introduced(&second, &device("dev_b"), &state.persons).unwrap(),
        110,
    ));
    events.push(said(
        "dev_bot",
        introduced(&agent, &device("dev_bot"), &state.persons).unwrap(),
        111,
    ));
    let state = State::replay(&events);

    let ops = certified(&seed(), &state.persons, &machines(&state), at(120));
    assert_eq!(ops.len(), 1, "only the second machine was waiting");
    events.extend(ops.into_iter().map(|op| said("dev_a", op, 120)));
    let state = State::replay(&events);

    assert_eq!(
        state.persons.person(&device("dev_b")),
        Some(seed().root().person())
    );
    assert_eq!(state.persons.person(&device("dev_bot")), None);
    assert!(certified(&seed(), &state.persons, &machines(&state), at(130)).is_empty());
}

#[test]
fn a_machine_taken_back_loses_its_person_and_comes_back_with_a_new_key() {
    let (_a, first) = room();
    let (_b, second) = room();
    let mut events = vec![
        joined("dev_a", DeviceKind::Machine),
        joined("dev_b", DeviceKind::Machine),
    ];
    events.extend(
        begun(&first, &device("dev_a"), &seed(), at(100))
            .unwrap()
            .into_iter()
            .map(|op| said("dev_a", op, 100)),
    );
    let state = State::replay(&events);
    events.push(said(
        "dev_b",
        introduced(&second, &device("dev_b"), &state.persons).unwrap(),
        110,
    ));
    let state = State::replay(&events);
    events.extend(
        certified(&seed(), &state.persons, &machines(&state), at(120))
            .into_iter()
            .map(|op| said("dev_a", op, 120)),
    );
    let state = State::replay(&events);
    let old = mine(&second, &device("dev_b")).unwrap().public();
    events.push(said(
        "dev_a",
        revoked(&seed(), &state.persons, &device("dev_b"), at(200)).unwrap(),
        200,
    ));
    let state = State::replay(&events);

    assert_eq!(state.persons.person(&device("dev_b")), None);
    assert_eq!(
        state.persons.person(&device("dev_a")),
        Some(seed().root().person()),
        "taking one machine back left the other alone"
    );
    let Some(Op::ShareKey { p }) = introduced(&second, &device("dev_b"), &state.persons) else {
        panic!("the machine taken back did not make a new key");
    };
    assert_ne!(share_key(&p), Some(old));
}

#[test]
fn only_the_person_who_vouched_can_take_a_machine_back() {
    let (_room, paths) = room();
    let events: Vec<Event> = begun(&paths, &device("dev_a"), &seed(), at(100))
        .unwrap()
        .into_iter()
        .map(|op| said("dev_a", op, 100))
        .collect();
    let state = State::replay(&events);

    let stranger = Seed::from_bytes(&[9; 32]);
    assert!(revoked(&stranger, &state.persons, &device("dev_a"), at(200)).is_none());
}

#[test]
fn a_key_nobody_can_read_is_not_taken_as_one() {
    let state = State::replay(&[said(
        "dev_a",
        Op::ShareKey {
            p: "not a key".into(),
        },
        100,
    )]);

    assert!(state.persons.keys.is_empty());
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
    for op in begun(&paths, &device("dev_a"), &seed(), at(100)).unwrap() {
        assert!(op.is_optional(), "{op:?}");
        assert!(op.settles(), "{op:?}");
        assert!(!op.destroys(), "{op:?}");
    }
}
