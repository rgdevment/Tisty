use super::*;

const RULE: &str = "---\nagent: claude/rule/CLAUDE.md\n---\n# Rules\n\nBe kind.\n";

#[test]
fn a_change_under_the_same_agent_line_is_proposable() {
    let proposed = RULE.replace("Be kind.", "Be kind. #always");
    assert_eq!(proposable(RULE, &proposed), Ok(()));
}

#[test]
fn a_proposal_never_moves_the_agent_line() {
    let elsewhere = RULE.replace("claude/rule/CLAUDE.md", "codex/rule/AGENTS.md");
    let gone = RULE.replace("agent: claude/rule/CLAUDE.md\n", "");
    let flagged = RULE.replace("CLAUDE.md\n", "CLAUDE.md, crlf\n");
    assert_eq!(proposable(RULE, &elsewhere), Err(Unfit::Retargeted));
    assert_eq!(proposable(RULE, &gone), Err(Unfit::Retargeted));
    assert_eq!(proposable(RULE, &flagged), Err(Unfit::Retargeted));
}

#[test]
fn a_document_without_a_card_or_a_memory_takes_no_proposal() {
    let memory = RULE.replace("claude/rule/CLAUDE.md", "claude/memory/notes/MEMORY.md");
    assert_eq!(
        proposable("# Plain\n", "# Plain\n\nMore.\n"),
        Err(Unfit::NoCard)
    );
    assert_eq!(proposable(&memory, &memory), Err(Unfit::Memory));
}

#[test]
fn a_proposal_the_card_cannot_show_whole_is_refused() {
    let long = format!("{RULE}{}", "x".repeat(PROPOSAL_AT_MOST));
    let many = format!("{RULE}{}", "x\n".repeat(PROPOSAL_LINES_AT_MOST));
    assert_eq!(proposable(RULE, &long), Err(Unfit::TooLong));
    assert_eq!(proposable(RULE, &many), Err(Unfit::TooManyLines));
    assert_eq!(proposable(&many, RULE), Err(Unfit::TooManyLines));
}

#[test]
fn a_script_whose_block_breaks_is_refused() {
    let script = crate::agents::doc_of(
        "claude",
        Kind::Skill,
        "tidy/run.sh",
        b"echo hi
",
    )
    .unwrap();
    let broken = script.trim_end().trim_end_matches('`').to_string();
    assert_eq!(proposable(&script, &script), Ok(()));
    assert_eq!(proposable(&script, &broken), Err(Unfit::NotWritable));
}
