use serde_json::json;

use super::*;

struct Home {
    _tmp: tempfile::TempDir,
    paths: Paths,
}

fn home(settings: &str) -> Home {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.config_file().parent().unwrap()).unwrap();
    std::fs::write(paths.config_file(), settings).unwrap();
    Home { _tmp: tmp, paths }
}

const LET_IN: &str = "agent_id = \"dev_agent01\"\n";

fn said(home: &Home, moment: &str, heard: Value, at: i128) -> Option<String> {
    answered(&home.paths, Client::ClaudeCode, moment, &heard, at)
}

fn edit(home: &Home, at: i128) {
    let heard = json!({ "session_id": "s1", "tool_name": "Edit" });
    assert_eq!(said(home, "tool", heard, at), None);
}

fn stop(home: &Home, at: i128) -> Option<String> {
    said(
        home,
        "stop",
        json!({ "session_id": "s1", "stop_hook_active": false }),
        at,
    )
}

#[test]
fn a_turn_that_edited_without_writing_in_tisty_is_handed_back_once() {
    let home = home(LET_IN);
    edit(&home, 10);

    let handed: Value = serde_json::from_str(&stop(&home, 20).unwrap()).unwrap();

    assert_eq!(handed["decision"], "block");
    assert_eq!(handed["reason"], crate::mcp::TURN_LEFT_UNWRITTEN);
    assert_eq!(stop(&home, 30), None);
}

#[test]
fn a_write_in_tisty_after_the_edits_lets_the_turn_end() {
    let home = home(LET_IN);
    edit(&home, 10);
    let tick = json!({ "session_id": "s1", "tool_name": "mcp__tisty__tick" });
    assert_eq!(said(&home, "tool", tick, 15), None);

    assert_eq!(stop(&home, 20), None);
}

#[test]
fn a_tool_of_tisty_in_a_plugin_counts_as_tisty() {
    let home = home(LET_IN);
    edit(&home, 10);
    let read = json!({ "session_id": "s1", "tool_name": "mcp__plugin_tisty_tisty__read" });
    said(&home, "tool", read, 12);
    assert!(stop(&home, 20).is_some(), "a read is no write");

    edit(&home, 30);
    let done = json!({ "session_id": "s1", "tool_name": "mcp__plugin_tisty_tisty__say_done" });
    said(&home, "tool", done, 35);
    assert_eq!(stop(&home, 40), None);
}

#[test]
fn a_closed_door_never_hands_a_turn_back() {
    let home = home("");
    edit(&home, 10);

    assert_eq!(stop(&home, 20), None);
}

#[test]
fn the_switch_turned_off_silences_every_moment() {
    let home = home(&format!("{LET_IN}{SWITCH} = false\n"));
    edit(&home, 10);

    assert_eq!(stop(&home, 20), None);
    assert_eq!(
        said(&home, "start", json!({ "session_id": "s1" }), 30),
        None
    );
    let marks = Session::of(home.paths.cache(), "s1").marks();
    assert_eq!(marks, marks::Marks::default());
}

#[test]
fn the_client_continuing_already_lets_the_turn_end() {
    let home = home(LET_IN);
    edit(&home, 10);

    let again = json!({ "session_id": "s1", "stop_hook_active": true });
    assert_eq!(said(&home, "stop", again, 20), None);
}

#[test]
fn hooks_installed_twice_hand_a_turn_back_once() {
    let home = home(LET_IN);
    edit(&home, 10);
    let marks = Session::of(home.paths.cache(), "s1").marks();
    let session = Session::of(home.paths.cache(), "s1");
    assert!(session.claim(&format!("stop-{}", marks.edited.unwrap())));

    assert_eq!(stop(&home, 20), None);
}

#[test]
fn a_start_behind_a_closed_door_says_nothing() {
    let home = home("");
    assert_eq!(
        said(&home, "start", json!({ "session_id": "s1" }), 30),
        None
    );
}

#[test]
fn an_unknown_moment_or_shape_says_nothing() {
    let home = home(LET_IN);
    assert_eq!(said(&home, "later", json!({}), 1), None);
    assert_eq!(said(&home, "stop", Value::Null, 2), None);
    assert_eq!(said(&home, "tool", json!([1, 2]), 3), None);
}
