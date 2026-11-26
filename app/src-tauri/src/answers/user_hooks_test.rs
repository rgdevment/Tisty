use super::*;

#[test]
fn a_hooks_state_reads_from_the_gate_and_its_yes() {
    assert_eq!(state_of(Seen::HookRegistered, None), "registered");
    assert_eq!(state_of(Seen::HookWaiting, None), "waiting");
    assert_eq!(state_of(Seen::Unread, None), "away");
}

#[test]
fn every_event_has_a_sample_an_agent_would_send() {
    for event in Event::ALL {
        let said: serde_json::Value = serde_json::from_slice(&sample(event, Tools::Shell)).unwrap();
        assert_eq!(said["session_id"], TRY_AS);
    }
}
