use super::*;
use crate::agents::{GLOBAL, doc_of, kept as backups, read_kept};

struct Room {
    data: tempfile::TempDir,
    home: tempfile::TempDir,
    ledger: Ledger,
}

fn room() -> Room {
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    Room {
        data: tempfile::tempdir().unwrap(),
        home: tempfile::tempdir().unwrap(),
        ledger,
    }
}

fn rule(level: &str, name: &str, said: &str) -> String {
    doc_of(level, Kind::Rule, name, said.as_bytes()).unwrap()
}

impl Room {
    fn target(&self, file: &str, pieces: Vec<String>) -> Target {
        Target {
            root: self.home.path().to_path_buf(),
            at: self.home.path().join(file),
            assistant: "codex".into(),
            pieces,
        }
    }

    fn round(&mut self, targets: &[Target]) -> Vec<Standing> {
        round(self.data.path(), &mut self.ledger, targets)
    }
}

fn read(target: &Target) -> String {
    std::fs::read_to_string(&target.at).unwrap()
}

#[test]
fn a_first_round_writes_what_the_documents_say() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "# Rules\n\nTests first.\n")],
    );
    assert_eq!(room.round(std::slice::from_ref(&target)), [Standing::Wrote]);
    assert_eq!(read(&target), "# Rules\n\nTests first.\n");
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::UpToDate]
    );
}

#[test]
fn global_and_the_assistants_own_are_composed_where_one_file_is_all_it_reads() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![
            rule(GLOBAL, "style.md", "Write in Spanish."),
            rule("codex", "AGENTS.md", "# Codex\n"),
        ],
    );
    room.round(std::slice::from_ref(&target));
    assert_eq!(read(&target), "Write in Spanish.\n\n# Codex\n");
}

#[test]
fn removing_one_rule_rewrites_the_file_but_emptying_the_folder_touches_nothing() {
    let mut room = room();
    let both = room.target(
        ".codex/AGENTS.md",
        vec![
            rule(GLOBAL, "style.md", "Write in Spanish.\n"),
            rule("codex", "AGENTS.md", "Tests first.\n"),
        ],
    );
    room.round(std::slice::from_ref(&both));

    let one = Target {
        pieces: vec![both.pieces[1].clone()],
        ..both.clone()
    };
    assert_eq!(room.round(std::slice::from_ref(&one)), [Standing::Wrote]);
    assert_eq!(read(&one), "Tests first.\n");

    let none = Target {
        pieces: vec![],
        ..both.clone()
    };
    assert_eq!(room.round(std::slice::from_ref(&none)), [Standing::Empty]);
    assert_eq!(read(&none), "Tests first.\n");
    assert_eq!(
        overwrite(room.data.path(), &mut room.ledger, &none),
        Err(Refused::Empty)
    );
    assert!(none.at.exists());
}

#[test]
fn a_rule_left_blank_never_empties_the_agents_file() {
    let mut room = room();
    let target = room.target(
        ".claude/CLAUDE.md",
        vec![rule("claude-code", "CLAUDE.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&target));
    let blank = Target {
        pieces: vec![rule("claude-code", "CLAUDE.md", "\n\n")],
        ..target.clone()
    };
    assert_eq!(room.round(std::slice::from_ref(&blank)), [Standing::Empty]);
    assert_eq!(read(&blank), "Rules.\n");
}

#[test]
fn a_change_made_outside_is_marked_and_never_written_over_alone() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Tests first.\n")],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::write(&target.at, "Tests first.\nAnd small commits.\n").unwrap();

    let told = room.round(std::slice::from_ref(&target));
    assert_eq!(
        told,
        [Standing::Outside {
            now: Some(b"Tests first.\nAnd small commits.\n".to_vec()),
            piece: Some(0)
        }]
    );
    assert_eq!(read(&target), "Tests first.\nAnd small commits.\n");
}

#[test]
fn a_change_on_both_sides_is_put_to_the_person() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "One.\n")],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::write(&target.at, "Outside.\n").unwrap();
    let edited = Target {
        pieces: vec![rule("codex", "AGENTS.md", "Inside.\n")],
        ..target.clone()
    };
    assert_eq!(
        room.round(std::slice::from_ref(&edited)),
        [Standing::Both {
            now: b"Outside.\n".to_vec()
        }]
    );
    assert_eq!(read(&edited), "Outside.\n");
}

#[test]
fn a_file_already_there_the_first_time_is_asked_about_unless_it_agrees() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Mine.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Theirs.\n").unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::Both {
            now: b"Theirs.\n".to_vec()
        }]
    );

    std::fs::write(&target.at, "Mine.\n").unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::UpToDate]
    );
    assert!(room.ledger.written(&target.at).is_some());
}

#[test]
fn a_file_removed_outside_is_marked_not_written_again() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::remove_file(&target.at).unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::Outside {
            now: None,
            piece: None
        }]
    );
    assert!(!target.at.exists());
}

#[test]
fn overwriting_keeps_what_the_file_held_first() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Tisty's.\n")],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::write(&target.at, "Outside.\r\n").unwrap();

    overwrite(room.data.path(), &mut room.ledger, &target).unwrap();
    assert_eq!(read(&target), "Tisty's.\n");
    let kept = backups(room.data.path(), &target.at);
    assert_eq!(kept.len(), 1);
    assert_eq!(
        read_kept(room.data.path(), &target.at, kept[0].n).unwrap(),
        b"Outside.\r\n"
    );
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::UpToDate]
    );
}

#[test]
fn a_backup_put_back_is_marked_as_a_change_outside_not_undone() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "New.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Old.\n").unwrap();
    overwrite(room.data.path(), &mut room.ledger, &target).unwrap();

    let n = backups(room.data.path(), &target.at)[0].n;
    restore(room.data.path(), &room.ledger, &target, n).unwrap();
    assert_eq!(read(&target), "Old.\n");
    assert!(matches!(
        room.round(std::slice::from_ref(&target))[0],
        Standing::Outside { .. }
    ));
    assert_eq!(backups(room.data.path(), &target.at).len(), 2);
}

#[test]
fn bringing_a_change_in_gives_the_document_and_leaves_the_file_alone() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Tests first.\n")],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::write(&target.at, "Tests first, always.\n").unwrap();

    let body = bring(&room.ledger, &target, 0).unwrap();
    assert!(body.ends_with("Tests first, always.\n"));
    let after = Target {
        pieces: vec![body],
        ..target.clone()
    };
    assert_eq!(
        room.round(std::slice::from_ref(&after)),
        [Standing::UpToDate]
    );
    assert_eq!(read(&after), "Tests first, always.\n");
}

#[test]
fn a_change_in_a_composed_file_is_traced_to_the_one_document_it_touched() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![
            rule(GLOBAL, "style.md", "Write in Spanish.\n"),
            rule("codex", "AGENTS.md", "Tests first.\n"),
        ],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::write(&target.at, "Write in Spanish.\n\nTests first, always.\n").unwrap();

    assert!(matches!(
        room.round(std::slice::from_ref(&target))[0],
        Standing::Outside { piece: Some(1), .. }
    ));
    let body = bring(&room.ledger, &target, 1).unwrap();
    assert!(body.ends_with("\n---\nTests first, always.\n"));
    assert_eq!(bring(&room.ledger, &target, 0), Err(Refused::Unattributed));
}

#[test]
fn a_change_across_two_composed_documents_is_left_for_the_person_to_place() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![
            rule(GLOBAL, "style.md", "A.\n"),
            rule("codex", "AGENTS.md", "B.\n"),
        ],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::write(&target.at, "A, changed.\n\nB, changed.\n").unwrap();
    assert!(matches!(
        room.round(std::slice::from_ref(&target))[0],
        Standing::Outside { piece: None, .. }
    ));
    assert_eq!(bring(&room.ledger, &target, 0), Err(Refused::Unattributed));
}

#[test]
fn the_agents_own_memory_is_read_and_never_written() {
    let mut room = room();
    let memory = doc_of(
        "claude-code",
        Kind::Memory,
        "projects/tisty/MEMORY.md",
        b"- old note\n",
    )
    .unwrap();
    let target = room.target(".claude/projects/tisty/memory/MEMORY.md", vec![memory]);
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::UpToDate]
    );
    assert!(!target.at.exists());

    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "- new note\n").unwrap();
    assert!(matches!(
        room.round(std::slice::from_ref(&target))[0],
        Standing::Outside { piece: Some(0), .. }
    ));
    assert_eq!(
        overwrite(room.data.path(), &mut room.ledger, &target),
        Err(Refused::AgentsOwn)
    );
    assert_eq!(read(&target), "- new note\n");
    assert!(
        bring(&room.ledger, &target, 0)
            .unwrap()
            .ends_with("- new note\n")
    );
}

#[test]
fn without_consent_on_this_computer_nothing_is_read_or_written() {
    let mut room = room();
    room.ledger = Ledger::fresh("machine-a");
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::NoConsent]
    );
    assert!(!target.at.exists());
    assert_eq!(
        overwrite(room.data.path(), &mut room.ledger, &target),
        Err(Refused::NoConsent)
    );
    assert_eq!(bring(&room.ledger, &target, 0), Err(Refused::NoConsent));
}

#[test]
fn an_assistant_turned_off_is_left_as_it_is() {
    let mut room = room();
    room.ledger.turn("codex", false);
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    assert_eq!(room.round(std::slice::from_ref(&target)), [Standing::Off]);
    assert!(!target.at.exists());
}

#[cfg(unix)]
#[test]
fn a_file_that_is_a_link_is_never_replaced() {
    let mut room = room();
    let real = room.home.path().join("dotfiles.md");
    std::fs::write(&real, "Kept elsewhere.\n").unwrap();
    let target = room.target("AGENTS.md", vec![rule("codex", "AGENTS.md", "Rules.\n")]);
    std::os::unix::fs::symlink(&real, &target.at).unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::Linked]
    );
    assert_eq!(
        overwrite(room.data.path(), &mut room.ledger, &target),
        Err(Refused::Linked)
    );
    assert_eq!(std::fs::read_to_string(&real).unwrap(), "Kept elsewhere.\n");
}

#[test]
fn a_real_file_written_from_its_imported_document_is_the_same_file() {
    let mut room = room();
    let file = include_bytes!("fixtures/notify.cmd");
    let target = room.target(
        ".claude/hooks/notify.cmd",
        vec![doc_of("claude-code", Kind::Rule, "hooks/notify.cmd", file).unwrap()],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, file).unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::UpToDate]
    );
    assert_eq!(std::fs::read(&target.at).unwrap(), file);
    assert!(backups(room.data.path(), &target.at).is_empty());
}

#[test]
fn a_credential_added_outside_is_never_brought_into_a_document() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&target));
    std::fs::write(
        &target.at,
        "Rules.\nToken: ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789\n",
    )
    .unwrap();
    assert!(matches!(
        bring(&room.ledger, &target, 0),
        Err(Refused::Secret(_))
    ));
}

#[cfg(unix)]
#[test]
fn a_folder_on_the_way_that_is_a_link_is_never_written_through() {
    let mut room = room();
    let shared = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(room.home.path().join("skills")).unwrap();
    std::os::unix::fs::symlink(shared.path(), room.home.path().join("skills/pdf")).unwrap();
    let skill = doc_of("claude-code", Kind::Skill, "pdf/SKILL.md", b"Fill forms.\n").unwrap();
    let target = room.target("skills/pdf/SKILL.md", vec![skill]);
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::Linked]
    );
    assert!(std::fs::read_dir(shared.path()).unwrap().next().is_none());
}

#[test]
fn a_target_outside_the_assistants_folder_is_never_written() {
    let mut room = room();
    let elsewhere = tempfile::tempdir().unwrap();
    let target = Target {
        at: elsewhere.path().join("AGENTS.md"),
        ..room.target("AGENTS.md", vec![rule("codex", "AGENTS.md", "Rules.\n")])
    };
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::Linked]
    );
    assert!(!target.at.exists());
}

#[test]
fn a_file_changed_between_looking_and_writing_is_left_alone() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Tisty's.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Saved by the agent.\n").unwrap();
    let seen = Some(crate::attach::printed(b"What was looked at.\n"));
    assert_eq!(
        written(room.data.path(), &mut room.ledger, &target, Some(&seen)),
        Err(Refused::Changed)
    );
    assert_eq!(read(&target), "Saved by the agent.\n");
}

#[test]
fn a_backup_is_not_put_back_while_the_assistant_is_turned_off() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "New.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Old.\n").unwrap();
    overwrite(room.data.path(), &mut room.ledger, &target).unwrap();
    room.ledger.turn("codex", false);
    assert_eq!(
        restore(room.data.path(), &room.ledger, &target, 1),
        Err(Refused::Off)
    );
    assert_eq!(read(&target), "New.\n");
}

#[test]
fn a_target_that_steps_out_of_the_assistants_folder_is_never_written() {
    let mut room = room();
    let target = room.target(
        ".codex/../escaped.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::Linked]
    );
    assert_eq!(bring(&room.ledger, &target, 0), Err(Refused::Linked));
    assert!(!room.home.path().join("escaped.md").exists());
}

#[test]
fn geminis_own_file_gets_one_import_line_and_keeps_everything_else() {
    let room = room();
    let target = room.target(".gemini/GEMINI.md", vec![]);
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "## Gemini Added Memories\r\n- likes tea").unwrap();

    let line = "@tisty-rules.md";
    assert_eq!(
        lead_in(room.data.path(), &room.ledger, &target, line),
        Ok(true)
    );
    assert_eq!(
        read(&target),
        "## Gemini Added Memories\r\n- likes tea\r\n@tisty-rules.md\r\n"
    );
    assert_eq!(
        lead_in(room.data.path(), &room.ledger, &target, line),
        Ok(false)
    );
    assert_eq!(
        read(&target),
        "## Gemini Added Memories\r\n- likes tea\r\n@tisty-rules.md\r\n"
    );
    let kept = backups(room.data.path(), &target.at);
    assert_eq!(kept.len(), 1);
    assert_eq!(
        read_kept(room.data.path(), &target.at, kept[0].n).unwrap(),
        b"## Gemini Added Memories\r\n- likes tea"
    );
}

#[test]
fn the_import_line_starts_the_file_when_there_is_none() {
    let room = room();
    let target = room.target(".gemini/GEMINI.md", vec![]);
    assert_eq!(
        lead_in(room.data.path(), &room.ledger, &target, "@tisty-rules.md"),
        Ok(true)
    );
    assert_eq!(read(&target), "@tisty-rules.md\n");
}

#[test]
fn the_import_line_needs_consent_and_must_be_one_import() {
    let mut room = room();
    let target = room.target(".gemini/GEMINI.md", vec![]);
    for line in ["no-at.md", "@a.md\n@b.md", " @a.md"] {
        assert_eq!(
            lead_in(room.data.path(), &room.ledger, &target, line),
            Err(Refused::Name),
            "{line}"
        );
    }
    room.ledger.turn("codex", false);
    assert_eq!(
        lead_in(room.data.path(), &room.ledger, &target, "@a.md"),
        Err(Refused::Off)
    );
    room.ledger = Ledger::fresh("machine-a");
    assert_eq!(
        lead_in(room.data.path(), &room.ledger, &target, "@a.md"),
        Err(Refused::NoConsent)
    );
    assert!(!target.at.exists());
}

#[test]
fn one_round_keeps_the_ledger_once_with_every_file_it_wrote() {
    let mut room = room();
    let all: Vec<Target> = (0..3)
        .map(|n| {
            room.target(
                &format!(".claude/rules/r{n}.md"),
                vec![rule("claude-code", &format!("rules/r{n}.md"), "Rule.\n")],
            )
        })
        .collect();
    assert_eq!(
        room.round(&all),
        [Standing::Wrote, Standing::Wrote, Standing::Wrote]
    );
    let kept = Ledger::open(room.data.path(), "machine-a");
    assert!(all.iter().all(|one| kept.written(&one.at).is_some()));
}

#[test]
fn files_written_before_the_ledger_was_kept_read_as_agreed_next_time() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&target));
    let mut lost = Ledger::fresh("machine-a");
    lost.consent(true);
    room.ledger = lost;
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::UpToDate]
    );
    assert!(backups(room.data.path(), &target.at).is_empty());
}
