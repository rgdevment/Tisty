use serde_json::{Value, json};

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
    answered(
        &home.paths,
        Client::ClaudeCode,
        moment,
        heard.to_string().as_bytes(),
        at,
    )
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
    let home = home(&format!("{LET_IN}turn_hooks = false\n"));
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

#[test]
fn a_call_without_its_session_leaves_no_mark_to_mix_with_others() {
    let home = home(LET_IN);
    assert_eq!(
        said(&home, "tool", json!({ "tool_name": "Edit" }), 10),
        None
    );
    assert_eq!(said(&home, "stop", json!({}), 20), None);
    assert!(!home.paths.cache().join("turns").exists());
}

fn nowhere() -> std::path::PathBuf {
    match cfg!(windows) {
        true => std::path::PathBuf::from(r"X:\Nowhere\Person"),
        false => std::path::PathBuf::from("/nowhere/person"),
    }
}

fn writing(session: &str, folder: &str) -> Value {
    let at = nowhere().join(folder).join("notes.md");
    json!({
        "session_id": session,
        "tool_name": "Write",
        "tool_input": { "file_path": at.display().to_string(), "content": "x" },
    })
}

fn before(home: &Home, heard: Value, at: i128) -> Option<String> {
    said(home, "before", heard, at)
}

#[test]
fn a_new_text_file_outside_is_asked_about_once() {
    let home = home(LET_IN);

    let first: Value =
        serde_json::from_str(&before(&home, writing("s1", "Docs"), 1).unwrap()).unwrap();
    let again = before(&home, writing("s1", "Docs"), 2);

    let decision = &first["hookSpecificOutput"];
    assert_eq!(decision["permissionDecision"], "deny");
    assert_eq!(
        decision["permissionDecisionReason"],
        crate::mcp::FILE_LEFT_OUTSIDE
    );
    assert_eq!(again, None);
    assert_eq!(Session::of(home.paths.cache(), "s1").marks().asked, Some(1));
}

#[test]
fn a_file_that_needs_no_question_is_let_through_silently() {
    let home = home(LET_IN);
    let at = nowhere().join("Docs").join("main.rs");
    let heard = json!({
        "session_id": "s1",
        "tool_name": "Write",
        "tool_input": { "file_path": at.display().to_string() },
    });

    assert_eq!(before(&home, heard, 1), None);
    assert_eq!(Session::of(home.paths.cache(), "s1").marks().asked, None);
}

#[test]
fn a_closed_door_or_the_switch_off_never_asks() {
    for settings in [
        String::new(),
        "agent_id = \"  \"\n".to_string(),
        format!("{LET_IN}turn_hooks = false\n"),
    ] {
        let home = home(&settings);

        assert_eq!(
            before(&home, writing("s1", "Docs"), 1),
            None,
            "{settings:?}"
        );
        assert!(!home.paths.cache().join("turns").exists(), "{settings:?}");
    }
}

#[test]
fn claude_codes_entry_run_inside_cursor_is_silent_and_claims_nothing() {
    let home = home(LET_IN);
    let mut inside = writing("s1", "Docs");
    inside["cursor_version"] = json!("3.2");

    assert_eq!(before(&home, inside, 1), None);
    assert_eq!(Session::of(home.paths.cache(), "s1").marks().asked, None);
    assert!(before(&home, writing("s1", "Docs"), 2).is_some());
}

#[test]
fn a_call_with_no_session_is_never_asked_about() {
    let home = home(LET_IN);
    let mut heard = writing("s1", "Docs");
    heard.as_object_mut().unwrap().remove("session_id");

    assert_eq!(before(&home, heard, 1), None);
    assert!(!home.paths.cache().join("turns").exists());
}

#[test]
fn settings_that_cannot_be_read_never_ask() {
    for bytes in [
        &b"not = [ toml"[..],
        b"agent_id = \xff\xfe",
        b"agent_id = 4\n",
    ] {
        let home = home("");
        std::fs::write(home.paths.config_file(), bytes).unwrap();

        assert_eq!(before(&home, writing("s1", "Docs"), 1), None, "{bytes:?}");
    }
}

#[test]
fn a_payload_that_cannot_be_read_never_asks_and_never_panics() {
    let home = home(LET_IN);
    for raw in [&b""[..], b"not json", b"[1]", b"{\"session_id\":\"s1\"}"] {
        let said = answered(&home.paths, Client::ClaudeCode, "before", raw, 1);
        assert_eq!(said, None, "{raw:?}");
    }
}

#[test]
fn a_turn_that_was_asked_about_is_not_handed_back_and_the_next_edited_turn_is() {
    let home = home(LET_IN);
    edit(&home, 10);
    assert!(before(&home, writing("s1", "Docs"), 15).is_some());

    assert_eq!(stop(&home, 20), None);

    edit(&home, 30);
    let handed: Value = serde_json::from_str(&stop(&home, 40).unwrap()).unwrap();
    assert_eq!(handed["decision"], "block");
}
