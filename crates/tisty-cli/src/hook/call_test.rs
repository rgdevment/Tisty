use serde_json::{Value, json};

use super::*;

fn leaving(heard: Value, tool: Option<&str>) -> Leaving {
    Call::read(heard.to_string().as_bytes()).leaving(tool)
}

fn file(path: &str) -> Leaving {
    Leaving::File(path.to_string())
}

fn command(said: &str) -> Leaving {
    Leaving::Command(said.to_string())
}

#[test]
fn a_file_is_read_from_the_field_each_client_names_it_in() {
    for tool in ["Write", "write_file", "create"] {
        for (field, expected) in [
            ("file_path", "a/one.md"),
            ("filePath", "a/two.md"),
            ("path", "a/three.md"),
            ("target_file", "a/four.md"),
        ] {
            let heard = json!({ "tool_input": { field: expected, "content": "x" } });
            assert_eq!(leaving(heard, Some(tool)), file(expected), "{tool} {field}");
        }
    }
}

#[test]
fn a_file_with_no_readable_name_leaves_nothing() {
    for input in [
        json!({ "tool_input": { "content": "x" } }),
        json!({ "tool_input": { "file_path": 4 } }),
        json!({ "tool_input": { "file_path": null } }),
        json!({}),
    ] {
        assert_eq!(leaving(input.clone(), Some("Write")), Leaving::Nothing);
    }
}

#[test]
fn a_shell_tool_leaves_the_command_it_was_given() {
    for tool in ["Bash", "PowerShell", "Shell"] {
        let heard = json!({ "tool_input": { "command": "echo x > a.md" } });
        assert_eq!(
            leaving(heard, Some(tool)),
            command("echo x > a.md"),
            "{tool}"
        );
    }
}

#[test]
fn a_command_with_no_tool_named_is_the_one_cursor_hands_over() {
    let heard = json!({ "command": "echo x > a.md", "cwd": "/w" });
    assert_eq!(leaving(heard, None), command("echo x > a.md"));
    assert_eq!(leaving(json!({}), None), Leaving::Nothing);
}

#[test]
fn a_command_at_the_top_is_ignored_when_a_tool_is_named() {
    let heard = json!({ "command": "echo x > a.md" });
    assert_eq!(leaving(heard.clone(), Some("Bash")), Leaving::Nothing);
    assert_eq!(leaving(heard, Some("Write")), Leaving::Nothing);
}

#[test]
fn a_card_asks_only_when_the_agent_started_it_on_its_own() {
    let start = |initiation: Value| json!({ "tool_input": { "initiation": initiation } });
    let tool = Some("mcp__ccd_session__start_session");
    assert_eq!(leaving(start(json!("own_initiative")), tool), Leaving::Card);
    for initiation in [json!("user_asked"), json!("offer"), json!(3), Value::Null] {
        assert_eq!(leaving(start(initiation), tool), Leaving::Nothing);
    }
    assert_eq!(leaving(json!({ "tool_input": {} }), tool), Leaving::Nothing);
    assert_eq!(leaving(json!({}), tool), Leaving::Nothing);
}

#[test]
fn any_other_tool_leaves_nothing() {
    let heard = json!({ "tool_input": {
        "file_path": "a.md", "command": "echo x > a.md", "initiation": "own_initiative"
    } });
    for tool in [
        "Edit",
        "TaskCreate",
        "CronCreate",
        "Task",
        "Read",
        "MultiEdit",
    ] {
        assert_eq!(
            leaving(heard.clone(), Some(tool)),
            Leaving::Nothing,
            "{tool}"
        );
    }
}

#[test]
fn copilot_hands_its_arguments_as_text_or_as_an_object() {
    let as_text = json!({ "toolArgs": json!({ "path": "a/c.md", "command": "ls" }).to_string() });
    let as_object = json!({ "toolArgs": { "path": "a/c.md", "command": "ls" } });
    for heard in [as_text, as_object] {
        assert_eq!(leaving(heard.clone(), Some("create")), file("a/c.md"));
        assert_eq!(leaving(heard, Some("Bash")), command("ls"));
    }
}

#[test]
fn arguments_of_a_wrong_shape_leave_nothing_and_never_panic() {
    for heard in [
        json!({ "tool_input": 4 }),
        json!({ "tool_input": null }),
        json!({ "tool_input": "not json" }),
        json!({ "tool_input": [1, 2] }),
        json!({ "toolArgs": "not json" }),
        json!({ "toolArgs": 7 }),
        json!({ "toolArgs": "[1]" }),
    ] {
        for tool in ["Write", "Bash", "create", "mcp__ccd_session__start_session"] {
            assert_eq!(leaving(heard.clone(), Some(tool)), Leaving::Nothing);
        }
        assert_eq!(leaving(heard.clone(), None), Leaving::Nothing);
        assert_eq!(Call::read(heard.to_string().as_bytes()).cwd(), "");
    }
}

#[test]
fn an_unreadable_payload_leaves_nothing_and_never_panics() {
    for raw in [
        &b""[..],
        b"not json",
        b"[1, 2]",
        b"null",
        b"{",
        b"\xff\xfe\x00",
    ] {
        let call = Call::read(raw);
        assert_eq!(call.leaving(Some("Write")), Leaving::Nothing);
        assert_eq!(call.leaving(Some("Bash")), Leaving::Nothing);
        assert_eq!(call.leaving(None), Leaving::Nothing);
        assert_eq!(call.cwd(), "");
    }
}

#[test]
fn the_folder_it_runs_in_is_the_tools_then_the_clients_then_none() {
    for (heard, expected) in [
        (
            json!({ "tool_input": { "working_directory": "/tool" }, "cwd": "/client" }),
            "/tool",
        ),
        (json!({ "tool_input": {}, "cwd": "/client" }), "/client"),
        (json!({ "cwd": "/client" }), "/client"),
        (
            json!({ "toolArgs": { "working_directory": "/tool" } }),
            "/tool",
        ),
        (json!({ "tool_input": {}, "cwd": 4 }), ""),
        (json!({}), ""),
    ] {
        assert_eq!(
            Call::read(heard.to_string().as_bytes()).cwd(),
            expected,
            "{heard}"
        );
    }
}
