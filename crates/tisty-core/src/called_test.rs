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
