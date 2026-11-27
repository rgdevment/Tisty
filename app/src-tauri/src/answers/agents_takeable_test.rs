use super::*;
use crate::answers::agents_sync::tests::{HERE, Room, put, room};

fn takeable_now(room: &Room) -> Vec<String> {
    takeable(&room.session, &room.homes, HERE).unwrap_or_default()
}

fn lists(room: &Room, name: &str) -> bool {
    takeable_now(room).contains(&room.at(name))
}

#[test]
fn without_the_yes_of_this_computer_nothing_is_listed_even_with_a_new_skill_there() {
    let room = room();
    put(&room.home.join(".claude/skills/pdf/SKILL.md"), "pdf\n");
    assert_eq!(takeable(&room.session, &room.homes, HERE), None);
}

#[test]
fn with_the_yes_a_new_skill_shows_by_its_file_and_so_does_a_rule_not_yet_brought() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    assert!(lists(&room, ".claude/rules/tone.md"));
    assert!(!lists(&room, ".claude/CLAUDE.md"));
    assert!(!lists(&room, ".claude/settings.json"));

    assert!(!lists(&room, ".claude/skills/pdf/SKILL.md"));
    put(
        &room.home.join(".claude/skills/pdf/SKILL.md"),
        "pdf
",
    );
    assert!(lists(&room, ".claude/skills/pdf/SKILL.md"));
}

#[test]
fn a_file_a_document_already_holds_does_not_show() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    assert!(lists(&room, ".claude/rules/tone.md"));
    room.bring(&[".claude/rules/tone.md"]);
    assert!(!lists(&room, ".claude/rules/tone.md"));
}

#[test]
fn an_agents_own_memory_never_shows() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let before = takeable_now(&room);
    put(
        &room.home.join(".claude/projects/repo/memory/MEMORY.md"),
        "- a fact
",
    );
    assert_eq!(takeable_now(&room), before);
}

#[test]
fn tistys_own_copies_never_show() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let at = room.home.join(".claude/skills/tisty/SKILL.md");
    put(
        &at,
        "Work with Tisty.
",
    );
    assert!(lists(&room, ".claude/skills/tisty/SKILL.md"));

    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    ledger.settle(
        &at,
        b"Work with Tisty.
",
    );
    ledger.applied_to(&at, ["tisty:skill/tisty/SKILL.md".to_string()]);
    ledger.keep(room.session.paths.data()).unwrap();

    assert!(!lists(&room, ".claude/skills/tisty/SKILL.md"));
}
