use serde_json::json;

use super::*;

fn parsed(said: &str) -> Value {
    serde_json::from_str(said).unwrap()
}

#[test]
fn each_client_is_told_its_context_in_its_own_shape() {
    let claude = parsed(&Client::ClaudeCode.started("hi"));
    let gemini = parsed(&Client::GeminiCli.started("hi"));
    let cursor = parsed(&Client::Cursor.started("hi"));
    let copilot = parsed(&Client::Copilot.started("hi"));

    for nested in [claude, gemini] {
        assert_eq!(
            nested["hookSpecificOutput"]["hookEventName"],
            "SessionStart"
        );
        assert_eq!(nested["hookSpecificOutput"]["additionalContext"], "hi");
    }
    assert_eq!(cursor, json!({ "additional_context": "hi" }));
    assert_eq!(copilot, json!({ "additionalContext": "hi" }));
}

#[test]
fn each_client_is_handed_its_turn_back_in_its_own_shape() {
    for blocking in [Client::ClaudeCode, Client::GeminiCli, Client::Copilot] {
        assert_eq!(
            parsed(&blocking.handed_back("why")),
            json!({ "decision": "block", "reason": "why" })
        );
    }
    assert_eq!(
        parsed(&Client::Cursor.handed_back("why")),
        json!({ "followup_message": "why" })
    );
}

#[test]
fn every_client_names_its_session_its_own_way() {
    assert_eq!(session(&json!({ "session_id": "a" })), "a");
    assert_eq!(session(&json!({ "sessionId": "b" })), "b");
    assert_eq!(session(&json!({ "conversation_id": "c" })), "c");
    assert_eq!(session(&json!({})), "");
    assert_eq!(session(&Value::Null), "");
}

#[test]
fn a_tool_of_tisty_is_known_however_the_client_spells_it() {
    let tool = |heard: Value| tisty_tool(&heard).map(str::to_string);
    assert_eq!(
        tool(json!({ "tool_name": "mcp__tisty__tick" })),
        Some("tick".into())
    );
    assert_eq!(
        tool(json!({ "tool_name": "mcp__plugin_tisty_tisty__say_done" })),
        Some("say_done".into())
    );
    assert_eq!(
        tool(json!({ "tool_name": "note", "mcp_server_name": "tisty" })),
        Some("note".into())
    );
    assert_eq!(
        tool(json!({ "tool_name": "mcp_tisty_catch_up" })),
        Some("catch_up".into())
    );
    assert_eq!(
        tool(json!({ "toolName": "tisty-tick" })),
        Some("tick".into())
    );
    assert_eq!(
        tool(json!({ "tool_name": "MCP:say_done" })),
        Some("say_done".into())
    );
}

#[test]
fn a_tool_of_another_server_is_not_tisty() {
    let tool = |heard: Value| tisty_tool(&heard).map(str::to_string);
    assert_eq!(tool(json!({ "tool_name": "mcp__sereno__note" })), None);
    assert_eq!(
        tool(json!({ "tool_name": "note", "mcp_server_name": "sereno" })),
        None
    );
    assert_eq!(tool(json!({ "tool_name": "MCP:fetch_page" })), None);
    assert_eq!(tool(json!({ "tool_name": "Edit" })), None);
    assert_eq!(tool(json!({ "tool_name": "mcp__tisty__" })), None);
    assert_eq!(tool(json!({ "toolName": "tisty-" })), None);
}

#[test]
fn the_edits_of_every_client_count_and_nothing_else_does() {
    for named in [
        "Edit",
        "Write",
        "NotebookEdit",
        "write_file",
        "replace",
        "edit",
        "create",
    ] {
        assert!(edits(&json!({ "tool_name": named })), "{named}");
    }
    assert!(edits(&json!({ "toolName": "apply_patch" })));
    for named in ["Read", "Bash", "view", "grep", "mcp__tisty__tick"] {
        assert!(!edits(&json!({ "tool_name": named })), "{named}");
    }
}

#[test]
fn a_client_already_continuing_is_heard_in_either_spelling() {
    assert!(again(&json!({ "stop_hook_active": true })));
    assert!(again(&json!({ "loop_count": 1 })));
    assert!(!again(&json!({ "stop_hook_active": false })));
    assert!(!again(&json!({ "loop_count": 0 })));
    assert!(!again(&json!({})));
}

#[test]
fn hooks_run_by_cursor_from_claude_settings_speak_for_cursor() {
    assert_eq!(
        Client::ClaudeCode.hand(&json!({ "cursor_version": "3.2" })),
        "cursor"
    );
    assert_eq!(Client::ClaudeCode.hand(&json!({})), "claude-code");
    assert_eq!(Client::Copilot.hand(&Value::Null), "copilot");
}

#[test]
fn every_client_id_reads_back() {
    for client in [
        Client::ClaudeCode,
        Client::Cursor,
        Client::GeminiCli,
        Client::Copilot,
    ] {
        assert_eq!(Client::named(client.id()), Some(client));
    }
    assert_eq!(Client::named("codex"), None);
}
