use serde_json::{Value, json};

use super::*;

fn parsed(said: &str) -> Value {
    serde_json::from_str(said).unwrap()
}

fn heard(value: Value) -> Heard {
    Heard::read(value.to_string().as_bytes())
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
fn each_client_is_denied_the_call_in_its_own_shape() {
    assert_eq!(
        parsed(&Client::ClaudeCode.denied("why")),
        json!({ "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": "why",
        } })
    );
    assert_eq!(
        parsed(&Client::Cursor.denied("why")),
        json!({ "permission": "deny", "agent_message": "why" })
    );
    assert_eq!(
        parsed(&Client::GeminiCli.denied("why")),
        json!({ "decision": "deny", "reason": "why" })
    );
    assert_eq!(
        parsed(&Client::Copilot.denied("why")),
        json!({ "permissionDecision": "deny", "permissionDecisionReason": "why" })
    );
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
    assert_eq!(heard(json!({ "session_id": "a" })).session(), "a");
    assert_eq!(heard(json!({ "sessionId": "b" })).session(), "b");
    assert_eq!(heard(json!({ "conversation_id": "c" })).session(), "c");
    assert_eq!(
        heard(json!({ "conversation_id": "d", "session_id": "d" })).session(),
        "d"
    );
    assert_eq!(heard(json!({})).session(), "");
    assert_eq!(Heard::read(b"not json").session(), "");
}

#[test]
fn what_an_edit_wrote_is_skipped_whatever_its_size() {
    let big = "x".repeat(4 * 1024 * 1024);
    let said = heard(json!({
        "session_id": "s",
        "tool_name": "Write",
        "tool_input": { "file_path": "a.txt", "content": big },
        "tool_response": { "content": big },
    }));
    assert_eq!(said.session(), "s");
    assert!(said.edits());
}

#[test]
fn a_tool_of_tisty_is_known_however_the_client_spells_it() {
    let tool = |value: Value| heard(value).tisty_tool().map(str::to_string);
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
    assert_eq!(
        tool(json!({ "tool_name": "MCP:tick" })),
        Some("tick".into())
    );
}

#[test]
fn a_field_of_an_unexpected_kind_loses_only_itself() {
    let said = heard(json!({
        "session_id": "s",
        "source": { "kind": "startup" },
        "stop_hook_active": "false",
        "tool_name": 7,
        "toolName": "edit",
    }));
    assert_eq!(said.session(), "s");
    assert_eq!(said.source(), "startup");
    assert!(!said.again());
    assert!(said.edits());
}

#[test]
fn a_tool_of_another_server_is_not_tisty() {
    let tool = |value: Value| heard(value).tisty_tool().map(str::to_string);
    assert_eq!(tool(json!({ "tool_name": "mcp__sereno__note" })), None);
    assert_eq!(
        tool(json!({ "tool_name": "note", "mcp_server_name": "sereno" })),
        None
    );
    assert_eq!(tool(json!({ "tool_name": "MCP:fetch_page" })), None);
    assert_eq!(
        tool(json!({ "tool_name": "MCP:note" })),
        None,
        "a name any server could use"
    );
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
        assert!(heard(json!({ "tool_name": named })).edits(), "{named}");
    }
    assert!(heard(json!({ "toolName": "apply_patch" })).edits());
    for named in ["Read", "Bash", "view", "grep", "mcp__tisty__tick"] {
        assert!(!heard(json!({ "tool_name": named })).edits(), "{named}");
    }
}

#[test]
fn only_a_client_saying_this_turn_already_continued_is_heard_as_again() {
    assert!(heard(json!({ "stop_hook_active": true })).again());
    assert!(!heard(json!({ "stop_hook_active": false })).again());
    assert!(!heard(json!({})).again());
    assert!(
        !heard(json!({ "loop_count": 3 })).again(),
        "Cursor counts follow-ups over the whole conversation"
    );
}

#[test]
fn hooks_run_by_cursor_from_claude_settings_speak_for_cursor() {
    assert_eq!(
        Client::ClaudeCode.hand(&heard(json!({ "cursor_version": "3.2" }))),
        "cursor"
    );
    assert_eq!(Client::ClaudeCode.hand(&heard(json!({}))), "claude-code");
    assert_eq!(Client::Copilot.hand(&Heard::default()), "copilot");
}

#[test]
fn every_client_id_reads_back() {
    for client in [
        Client::ClaudeCode,
        Client::Cursor,
        Client::GeminiCli,
        Client::Copilot,
        Client::Codex,
        Client::Antigravity,
    ] {
        assert_eq!(Client::named(client.id()), Some(client));
    }
    assert_eq!(Client::Codex.id(), "codex");
    assert_eq!(Client::Antigravity.id(), "antigravity");
    assert_eq!(Client::named("windsurf"), None);
}

#[test]
fn codex_is_told_its_context_and_denied_the_call_like_claude_code() {
    assert_eq!(
        parsed(&Client::Codex.started("hi")),
        parsed(&Client::ClaudeCode.started("hi"))
    );
    assert_eq!(
        parsed(&Client::Codex.denied("why")),
        parsed(&Client::ClaudeCode.denied("why"))
    );
    assert_eq!(
        parsed(&Client::Codex.handed_back("why")),
        json!({ "decision": "block", "reason": "why" })
    );
}

#[test]
fn antigravity_is_told_its_context_as_a_step_to_inject() {
    assert_eq!(
        parsed(&Client::Antigravity.started("hi")),
        json!({ "injectSteps": [ { "systemMessage": { "systemMessage": "hi" } } ] })
    );
}

#[test]
fn antigravity_is_denied_the_call_and_kept_going_in_its_own_words() {
    assert_eq!(
        parsed(&Client::Antigravity.denied("why")),
        json!({ "decision": "deny", "reason": "why" })
    );
    assert_eq!(
        parsed(&Client::Antigravity.handed_back("why")),
        json!({ "decision": "continue", "reason": "why" })
    );
}

#[test]
fn antigravity_names_its_session_and_its_tool_in_its_own_fields() {
    let said = heard(json!({
        "conversationId": "conv-1",
        "toolCall": { "name": "write_to_file", "args": { "TargetFile": "a.md" } },
    }));

    assert_eq!(said.session(), "conv-1");
    assert_eq!(said.tool(), Some("write_to_file"));
    assert!(said.edits());
}

#[test]
fn the_tool_a_client_names_at_the_top_wins_over_the_one_inside_its_call() {
    let said = heard(json!({
        "tool_name": "Read",
        "toolCall": { "name": "write_to_file" },
    }));

    assert_eq!(said.tool(), Some("Read"));
    assert!(!said.edits());
}

#[test]
fn a_tool_call_without_a_readable_name_names_no_tool() {
    for value in [
        json!({ "toolCall": {} }),
        json!({ "toolCall": { "name": 4 } }),
        json!({ "toolCall": "write_to_file" }),
    ] {
        assert_eq!(heard(value.clone()).tool(), None, "{value}");
    }
}

#[test]
fn the_edits_of_antigravity_count_and_its_other_tools_do_not() {
    for named in [
        "write_to_file",
        "replace_file_content",
        "multi_replace_file_content",
    ] {
        let said = heard(json!({ "toolCall": { "name": named } }));
        assert!(said.edits(), "{named}");
    }
    for named in ["run_command", "view_file", "grep_search", "mcp_tisty_tick"] {
        let said = heard(json!({ "toolCall": { "name": named } }));
        assert!(!said.edits(), "{named}");
    }
}

#[test]
fn a_tool_of_tisty_is_known_as_antigravity_spells_it() {
    let said = heard(json!({ "toolCall": { "name": "mcp_tisty_say_done" } }));

    assert_eq!(said.tisty_tool(), Some("say_done"));
}

#[test]
fn the_hand_of_codex_and_antigravity_is_their_own_id() {
    assert_eq!(Client::Codex.hand(&Heard::default()), "codex");
    assert_eq!(Client::Antigravity.hand(&Heard::default()), "antigravity");
}
