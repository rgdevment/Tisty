use super::machine;
use tisty_core::{DeviceId, State};

fn device(named: &str) -> DeviceId {
    DeviceId(named.into())
}

fn with_an_assistant_on(host: Option<&str>) -> State {
    let mut state = State::default();
    state.assistants.insert(device("agent"));
    if let Some(host) = host {
        state.hosts.insert(device("agent"), device(host));
    }
    state
}

#[test]
fn an_assistant_writes_from_the_machine_that_hosts_it() {
    let state = with_an_assistant_on(Some("laptop"));

    assert_eq!(machine(&state, &device("agent")), Some(device("laptop")));
    assert_eq!(machine(&state, &device("laptop")), Some(device("laptop")));
}

#[test]
fn an_assistant_nobody_hosts_is_on_no_machine_known() {
    let state = with_an_assistant_on(None);

    assert_eq!(machine(&state, &device("agent")), None);
}
