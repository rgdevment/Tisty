use super::{client_id, client_named, client_said};

#[test]
fn a_client_is_named_for_people_by_what_it_called_itself() {
    assert_eq!(client_named("claude-code"), "Claude Code");
    assert_eq!(client_named("Claude Code"), "Claude Code");
    assert_eq!(client_named("codex-cli"), "Codex");
    assert_eq!(client_named("codex/1.2.3"), "Codex");
    assert_eq!(client_named("codex-mcp-client"), "Codex");
    assert_eq!(client_named("gemini-cli-mcp-client"), "Gemini CLI");
    assert_eq!(client_named("Visual Studio Code"), "Visual Studio Code");
    assert_eq!(client_named("antigravity"), "Antigravity");
    assert_eq!(client_named("some-new_tool"), "Some New Tool");
}

#[test]
fn a_known_client_is_the_one_wired_under_that_id_whatever_it_said() {
    assert_eq!(client_id("codex-mcp-client"), Some("codex"));
    assert_eq!(client_id("Codex-CLI/0.9"), Some("codex"));
    assert_eq!(client_id("visual studio code"), Some("vscode"));
    assert_eq!(client_id("gemini-cli-mcp-client"), Some("gemini-cli"));
    assert_eq!(client_id("claude-code"), Some("claude-code"));
    assert_eq!(client_id("some-new-tool"), None);
}

#[test]
fn what_a_client_said_is_kept_short_plain_and_composed() {
    assert_eq!(client_said("  claude-code \n"), Some("claude-code".into()));
    assert_eq!(client_said(""), None);
    assert_eq!(client_said("\u{7}\u{1b}"), None);
    assert_eq!(
        client_said("\u{202e}codex\u{200b}\u{feff} \u{2066}"),
        Some("codex".into()),
        "what turns text around or hides in it is not a name"
    );
    let long = client_said(&"x".repeat(100)).unwrap();
    assert_eq!(long.chars().count(), 40);
}
