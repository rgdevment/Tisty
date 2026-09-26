use super::*;
use crate::event::{DeviceId, Event, LogAdd, Resolve};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn through(ms: i64, who: &str, via: Option<&str>, op: Op) -> Event {
    let mut event = Event::new(DeviceId(who.into()), at(ms), op);
    event.via = via.map(str::to_string);
    event
}

fn with_an_agent() -> State {
    let mut state = State::default();
    state.apply(&through(
        1,
        "dev_agent",
        None,
        Op::DeviceJoin {
            d: DeviceId("dev_agent".into()),
            k: Some(crate::event::DeviceKind::Agent),
        },
    ));
    state
}

#[test]
fn what_the_agent_wrote_carries_the_client_it_spoke_through() {
    let mut state = with_an_agent();
    let id = Ulid::generate();
    state.apply(&through(
        2,
        "dev_agent",
        Some("claude-code"),
        Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new("pasar biome", "a0"),
        },
    ));
    let entry = Ulid::generate();
    state.apply(&through(
        3,
        "dev_agent",
        Some("codex"),
        Op::TaskLog {
            id,
            d: LogAdd::new(entry, "0 errores"),
        },
    ));
    state.apply(&through(
        4,
        "dev_agent",
        Some("codex"),
        Op::TaskResolve {
            id,
            d: Resolve::new(entry),
        },
    ));

    let task = &state.tasks[&id];
    assert_eq!(task.created_via.as_deref(), Some("claude-code"));
    assert_eq!(task.log[0].via.as_deref(), Some("codex"));
    assert_eq!(
        task.resolved.as_ref().unwrap().via.as_deref(),
        Some("codex"),
        "the mark takes the envelope's client when the payload names none"
    );
}

#[test]
fn a_mark_rewritten_by_the_person_keeps_the_client_its_payload_names() {
    let mut state = with_an_agent();
    let id = Ulid::generate();
    state.apply(&through(
        2,
        "dev_agent",
        Some("codex"),
        Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new("pasar biome", "a0"),
        },
    ));
    let entry = Ulid::generate();
    state.apply(&through(
        3,
        "dev_laptop",
        None,
        Op::TaskResolve {
            id,
            d: Resolve::new(entry)
                .said_by(at(2), DeviceId("dev_agent".into()))
                .through(Some("codex".into())),
        },
    ));

    let said = state.tasks[&id].resolved.as_ref().unwrap();
    assert_eq!(said.by, DeviceId("dev_agent".into()));
    assert_eq!(said.via.as_deref(), Some("codex"));
}

#[test]
fn where_an_agent_lives_is_its_own_word_or_its_hosts_and_nobody_elses() {
    let mut state = with_an_agent();
    let agent = DeviceId("dev_agent".into());
    let host = DeviceId("dev_laptop".into());

    state.apply(&through(
        2,
        "dev_stranger",
        None,
        Op::DeviceHost {
            d: agent.clone(),
            of: host.clone(),
        },
    ));
    assert!(
        state.hosts.is_empty(),
        "a third device does not say where it lives"
    );

    state.apply(&through(
        3,
        "dev_agent",
        None,
        Op::DeviceHost {
            d: agent.clone(),
            of: host.clone(),
        },
    ));
    assert_eq!(state.hosts.get(&agent), Some(&host));

    let other = DeviceId("dev_desktop".into());
    state.apply(&through(
        4,
        "dev_desktop",
        None,
        Op::DeviceHost {
            d: agent.clone(),
            of: other.clone(),
        },
    ));
    assert_eq!(
        state.hosts.get(&agent),
        Some(&other),
        "the host may say so itself"
    );

    state.apply(&through(
        5,
        "dev_agent",
        None,
        Op::DeviceHost {
            d: agent.clone(),
            of: agent.clone(),
        },
    ));
    assert_eq!(
        state.hosts.get(&agent),
        Some(&other),
        "an agent is hosted on a machine, never on an agent"
    );

    state.apply(&through(
        6,
        "dev_laptop",
        None,
        Op::DeviceHost {
            d: host.clone(),
            of: other.clone(),
        },
    ));
    assert!(
        !state.hosts.contains_key(&host),
        "a machine is nobody's guest"
    );

    state.apply(&through(
        7,
        "dev_laptop",
        None,
        Op::DeviceRemove { d: agent.clone() },
    ));
    assert!(state.hosts.is_empty(), "a retired agent lives nowhere");
}

#[test]
fn nothing_read_at_replay_turns_on_the_client_named() {
    let mut state = with_an_agent();
    let id = Ulid::generate();
    state.apply(&through(
        2,
        "dev_laptop",
        Some("claude-code"),
        Op::TaskAdd {
            id,
            d: crate::event::TaskAdd::new("mine", "a0"),
        },
    ));
    state.apply(&through(
        3,
        "dev_agent",
        Some("i-am-the-person"),
        Op::TaskDone { id, filled: false },
    ));

    let task = &state.tasks[&id];
    assert_eq!(
        task.status,
        Status::Open,
        "a name on the envelope opens no door"
    );
    assert_eq!(task.created_via.as_deref(), Some("claude-code"));
    assert!(!state.filed_by_agents(task), "and makes nobody an agent");
}
