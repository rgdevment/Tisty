use super::*;

fn yes(agents: &[&str]) -> HookYes {
    HookYes {
        print: String::new(),
        name: "guard.sh".into(),
        script: String::new(),
        event: BEFORE_TOOL.into(),
        tools: "shell".into(),
        timeout: 5,
        agents: agents.iter().map(|one| (*one).to_string()).collect(),
    }
}

#[test]
fn only_the_flagged_name_and_agent_are_asked() {
    let said = |all: &[&str]| asked(&all.iter().map(|one| (*one).to_string()).collect::<Vec<_>>());
    assert_eq!(
        said(&["--name", "guard.sh", "--agent", "codex"]),
        Some(("guard.sh".into(), "codex".into()))
    );
    assert_eq!(said(&["guard.sh", "codex"]), None);
    assert_eq!(said(&["--agent", "codex", "--name", "guard.sh"]), None);
}

#[test]
fn a_hook_answers_only_for_the_agents_it_was_registered_with() {
    let heard = Heard::read(b"{}");
    assert_eq!(speaker("codex", &heard, &yes(&["claude-code"])), None);
    assert_eq!(
        speaker("codex", &heard, &yes(&["codex"])),
        Some(Client::Codex)
    );
}

#[test]
fn inside_cursor_claude_codes_entry_answers_as_cursor_and_never_twice() {
    let heard = Heard::read(br#"{"cursor_version":"1.0"}"#);
    assert_eq!(
        speaker(CLAUDE_CODE, &heard, &yes(&[CLAUDE_CODE])),
        Some(Client::Cursor)
    );
    assert_eq!(
        speaker(CLAUDE_CODE, &heard, &yes(&[CLAUDE_CODE, CURSOR])),
        None
    );
}

#[test]
fn a_silent_block_still_says_which_hook_stopped_it() {
    assert_eq!(
        reason("guard.sh", "  "),
        "The hook guard.sh stopped this call."
    );
    assert_eq!(reason("guard.sh", "no rm"), "no rm");
}
