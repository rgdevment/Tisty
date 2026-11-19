use super::*;
use crate::agents::{doc_of, rendered};

fn doc(address: &str, words: &str) -> String {
    format!("---\nagent: {address}\ntitle: {address}\n---\n{words}")
}

fn label(address: &str, words: &str) -> String {
    shown(&doc(address, words))
        .expect("an agent document")
        .label
}

#[test]
fn a_rule_is_called_by_its_first_heading() {
    let body = doc(
        "claude/rule/rules/horario.md",
        "Intro.\n\n## Horario laboral en repos personales\n\n# Otro\n",
    );
    assert_eq!(
        shown(&body),
        Some(Shown {
            kind: "rule",
            level: "claude".into(),
            name: "rules/horario.md".into(),
            label: "Horario laboral en repos personales".into(),
            skill: None,
        })
    );
}

#[test]
fn a_rule_without_a_heading_is_called_by_its_file_name() {
    assert_eq!(label("global/rule/AGENTS.md", "Just words.\n"), "AGENTS.md");
    assert_eq!(label("claude/rule/rules/horario.md", ""), "horario.md");
}

#[test]
fn a_heading_inside_a_fence_is_code_not_a_name() {
    let words = "```sh\n# not a name\n```\n\n# Real name\n";
    assert_eq!(label("claude/rule/CLAUDE.md", words), "Real name");
}

#[test]
fn a_code_file_is_called_by_its_file_name() {
    let words = "```sh\n# looks like a heading\necho hi\n```\n";
    assert_eq!(label("claude/skill/pr/run.sh", words), "run.sh");
}

#[test]
fn a_skill_is_called_by_the_name_in_its_own_properties() {
    let words = "---\nname: pr\ndescription: Open a pull request\n---\n# Pull requests\n";
    let one = shown(&doc("claude/skill/pr/SKILL.md", words)).expect("a skill");
    assert_eq!(one.label, "pr");
    assert_eq!(one.skill.as_deref(), Some("pr"));
    assert_eq!(one.kind, "skill");
}

#[test]
fn a_skill_without_a_name_is_called_by_its_folder() {
    assert_eq!(
        label("claude/skill/review/SKILL.md", "# Review\n"),
        "review"
    );
    assert_eq!(
        label("claude/skill/review/skill.md", "---\nname: \"\"\n---\n"),
        "review"
    );
}

#[test]
fn the_other_files_of_a_skill_keep_their_folder_and_heading() {
    let one = shown(&doc(
        "claude/skill/pr/references/body.md",
        "---\nname: not this\n---\n# How the body reads\n",
    ))
    .expect("a skill file");
    assert_eq!(one.skill.as_deref(), Some("pr"));
    assert_eq!(one.label, "How the body reads");
}

#[test]
fn a_skill_file_loose_in_the_skills_folder_stands_alone() {
    let one = shown(&doc("claude/skill/notes.md", "# Notes\n")).expect("a skill file");
    assert_eq!(one.skill, None);
    assert_eq!(one.label, "Notes");
}

#[test]
fn memory_and_global_rules_say_their_kind_and_level() {
    let one = shown(&doc("claude/memory/MEMORY.md", "# Memory index\n")).expect("memory");
    assert_eq!((one.kind, one.level.as_str()), ("memory", "claude"));
    let one = shown(&doc("global/rule/comunes.md", "")).expect("global");
    assert_eq!((one.kind, one.level.as_str()), ("rule", "global"));
}

#[test]
fn crlf_and_bom_files_are_named_the_same() {
    let file = "\u{feff}# Con BOM\r\n\r\nTexto\r\n".as_bytes();
    let body = doc_of("claude", Kind::Rule, "CLAUDE.md", file).expect("a document");
    assert_eq!(shown(&body).expect("named").label, "Con BOM");
}

#[test]
fn naming_never_changes_the_bytes_the_agent_reads() {
    let file = "---\nname: pr\n---\n# PR\r\nTexto sin final".as_bytes();
    let body = doc_of("claude", Kind::Skill, "pr/SKILL.md", file).expect("a document");
    let before = rendered(&body).expect("rendered");
    let _ = shown(&body);
    assert_eq!(before, file);
    assert_eq!(rendered(&body).expect("rendered"), file);
}

#[test]
fn a_document_without_a_card_has_no_name_here() {
    assert_eq!(shown("# Just a page\n"), None);
    assert_eq!(shown("---\ntitle: x\n---\n# x\n"), None);
}
