use super::*;
use crate::event::Event;

fn named(name: &str) -> Named {
    Named {
        name: name.into(),
        os: Some("macOS".into()),
    }
}

#[test]
fn a_machine_says_its_name_once_and_again_only_when_it_changes() {
    let who = DeviceId("dev_a".into());
    let mut state = State::default();

    let first = told(&state, &who, Some(named("MacBook Pro de Rodrigo"))).unwrap();
    state.apply(&Event::new(who.clone(), jiff::Timestamp::now(), first));

    assert!(told(&state, &who, Some(named("MacBook Pro de Rodrigo"))).is_none());
    assert!(told(&state, &who, Some(named("Mac del trabajo"))).is_some());
    assert!(told(&state, &who, None).is_none());
}

#[test]
fn nobody_names_a_machine_but_the_machine_itself() {
    let who = DeviceId("dev_a".into());
    let mut state = State::default();

    state.apply(&Event::new(
        DeviceId("dev_b".into()),
        jiff::Timestamp::now(),
        Op::DeviceNamed {
            d: who.clone(),
            name: "otro nombre".into(),
            os: None,
        },
    ));

    assert!(!state.named.contains_key(&who));
}

#[test]
fn this_computer_has_a_name() {
    assert!(here().is_some_and(|one| !one.name.is_empty()));
}

#[test]
fn a_long_name_is_kept_the_way_it_is_read_so_it_is_not_said_again() {
    let long = "MacBook Pro de Rodrigo ".repeat(6);
    let who = DeviceId("dev_a".into());
    let mut state = State::default();
    let first = told(
        &state,
        &who,
        Some(Named {
            name: cleaned(&long),
            os: Some("macOS".into()),
        }),
    )
    .unwrap();
    state.apply(&Event::new(who.clone(), jiff::Timestamp::now(), first));

    assert!(cleaned(&long).chars().count() <= AT_MOST);
    assert!(
        told(
            &state,
            &who,
            Some(Named {
                name: cleaned(&long),
                os: Some("macOS".into()),
            }),
        )
        .is_none(),
        "every round would write the name again"
    );
}

#[test]
fn the_name_the_person_gave_comes_before_the_systems_and_an_empty_one_gives_it_back() {
    assert_eq!(
        chosen(Some("  Roble 42 ")).map(|one| one.name),
        Some("Roble 42".to_string())
    );
    assert_eq!(
        chosen(Some(SYSTEM)).and_then(|one| one.os),
        Some(SYSTEM.to_string())
    );
    assert_eq!(chosen(Some("   ")), here());
    assert_eq!(chosen(None), here());
}
