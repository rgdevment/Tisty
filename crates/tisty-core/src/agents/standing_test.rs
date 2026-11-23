use super::*;
use crate::agents::{Admitted, BUILTIN, GLOBAL, KEPT_AT_MOST, doc_of, kept as backups, read_kept};
use linked_test::linked_folder;

#[path = "linked_test.rs"]
mod linked_test;

struct Room {
    data: tempfile::TempDir,
    home: tempfile::TempDir,
    ledger: Ledger,
}

fn room() -> Room {
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    ledger.take_tisty_skills(true);
    Room {
        data: tempfile::tempdir().unwrap(),
        home: tempfile::tempdir().unwrap(),
        ledger,
    }
}

fn piece(body: &str) -> Admitted {
    Admitted::trusted(
        &format!("doc-{}", crate::attach::printed(body.as_bytes())),
        body,
    )
    .unwrap()
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
            pieces: pieces.iter().map(|body| piece(body)).collect(),
            held: false,
            unproven: false,
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
        pieces: vec![piece(&rule("claude-code", "CLAUDE.md", "\n\n"))],
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
        pieces: vec![piece(&rule("codex", "AGENTS.md", "Inside.\n"))],
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
        pieces: vec![piece(&body)],
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

#[test]
fn the_persons_original_outlives_every_later_write() {
    let mut room = room();
    let at = room.home.path().join(".codex/AGENTS.md");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, "The person's own rules.\n").unwrap();
    let first = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Version 0.\n")],
    );
    overwrite(room.data.path(), &mut room.ledger, &first).unwrap();
    for n in 1..=25 {
        let next = room.target(
            ".codex/AGENTS.md",
            vec![rule("codex", "AGENTS.md", &format!("Version {n}.\n"))],
        );
        assert_eq!(room.round(std::slice::from_ref(&next)), [Standing::Wrote]);
    }
    let all = backups(room.data.path(), &at);
    assert_eq!(all.len(), crate::agents::KEPT_AT_MOST + 1);
    let pinned = all.last().unwrap();
    assert!(pinned.pinned);
    assert_eq!(
        read_kept(room.data.path(), &at, pinned.n).unwrap(),
        b"The person's own rules.\n"
    );
}

#[test]
fn writing_bytes_the_file_already_holds_leaves_no_backup() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Same.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Same.\n").unwrap();
    overwrite(room.data.path(), &mut room.ledger, &target).unwrap();
    overwrite(room.data.path(), &mut room.ledger, &target).unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::UpToDate]
    );
    assert!(backups(room.data.path(), &target.at).is_empty());
}

#[test]
fn putting_back_a_backup_the_file_already_holds_leaves_no_new_one() {
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "New.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Old.\n").unwrap();
    overwrite(room.data.path(), &mut room.ledger, &target).unwrap();
    restore(room.data.path(), &room.ledger, &target, 1).unwrap();
    assert_eq!(backups(room.data.path(), &target.at).len(), 2);
    restore(room.data.path(), &room.ledger, &target, 1).unwrap();
    assert_eq!(backups(room.data.path(), &target.at).len(), 2);
}

#[test]
fn the_persons_first_file_outlives_every_backup_cap_through_real_rounds() {
    let mut room = room();
    let at = ".claude/CLAUDE.md";
    let original = "# Mine\r\n\r\nWritten by hand.";
    let first = room.target(at, vec![rule("claude", "CLAUDE.md", original)]);
    std::fs::create_dir_all(first.at.parent().unwrap()).unwrap();
    std::fs::write(&first.at, original).unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&first)),
        [Standing::UpToDate]
    );

    let edits = KEPT_AT_MOST + 5;
    for n in 0..edits {
        let target = room.target(
            at,
            vec![
                rule(GLOBAL, "common.md", &format!("Edit {n}.\n")),
                rule("claude", "CLAUDE.md", original),
            ],
        );
        assert_eq!(room.round(&[target]), [Standing::Wrote], "edit {n}");
    }

    let kept = backups(room.data.path(), &first.at);
    assert_eq!(kept.len(), KEPT_AT_MOST + 1);
    let pinned = kept.last().unwrap();
    assert!(pinned.pinned);
    assert_eq!(
        read_kept(room.data.path(), &first.at, pinned.n).unwrap(),
        original.as_bytes()
    );
}

#[test]
fn a_file_brought_after_a_global_rule_takes_the_rule_with_a_backup_of_itself() {
    let mut room = room();
    let theirs = "# Codex\n\nTheirs.\n";
    let mut target = room.target(
        ".codex/AGENTS.override.md",
        vec![rule(GLOBAL, "common.md", "Common.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, theirs).unwrap();
    assert!(matches!(
        room.round(std::slice::from_ref(&target))[0],
        Standing::Both { .. }
    ));

    let brought = piece(&rule("codex", "AGENTS.override.md", theirs));
    target.put(&brought);
    assert!(adopted(&mut room.ledger, &target, brought.file()));
    assert_eq!(standing(&room.ledger, &target), Standing::Write);
    assert_eq!(room.round(std::slice::from_ref(&target)), [Standing::Wrote]);
    assert_eq!(read(&target), "Common.\n\n# Codex\n\nTheirs.\n");
    let kept = backups(room.data.path(), &target.at);
    assert_eq!(
        read_kept(room.data.path(), &target.at, kept[0].n).unwrap(),
        theirs.as_bytes()
    );
}

#[test]
fn only_a_file_still_as_it_was_brought_is_adopted() {
    let mut room = room();
    let brought = piece(&rule("codex", "AGENTS.md", "Brought.\n"));
    let mut target = room.target(".codex/AGENTS.md", vec![]);
    target.put(&brought);
    assert!(!adopted(&mut room.ledger, &target, brought.file()));

    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Changed since.\n").unwrap();
    assert!(!adopted(&mut room.ledger, &target, brought.file()));
    assert!(!adopted(&mut room.ledger, &target, "another-doc"));

    std::fs::write(&target.at, "Brought.\n").unwrap();
    room.ledger.settle(&target.at, b"Earlier.\n");
    assert!(!adopted(&mut room.ledger, &target, brought.file()));

    room.ledger = Ledger::fresh("machine-a");
    assert!(!adopted(&mut room.ledger, &target, brought.file()));
    assert!(room.ledger.written(&target.at).is_none());
}

#[test]
fn a_file_no_document_places_goes_with_a_backup_and_is_forgotten() {
    let mut room = room();
    let placed = room.target(
        ".claude/rules/style.md",
        vec![rule("claude", "rules/style.md", "Style.\n")],
    );
    room.round(std::slice::from_ref(&placed));
    let loose = room.target(".claude/rules/style.md", vec![]);

    removed(room.data.path(), &mut room.ledger, &loose).unwrap();
    assert!(!loose.at.exists());
    assert!(room.ledger.written(&loose.at).is_none());
    let kept = backups(room.data.path(), &loose.at);
    assert_eq!(
        read_kept(room.data.path(), &loose.at, kept[0].n).unwrap(),
        b"Style.\n"
    );
    removed(room.data.path(), &mut room.ledger, &loose).unwrap();
}

#[test]
fn a_file_is_never_taken_away_while_a_document_places_it_or_without_leave() {
    let mut room = room();
    let placed = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&placed));
    assert_eq!(
        removed(room.data.path(), &mut room.ledger, &placed),
        Err(Refused::Held)
    );
    let mut held = room.target(".codex/AGENTS.md", vec![]);
    held.held = true;
    assert_eq!(
        removed(room.data.path(), &mut room.ledger, &held),
        Err(Refused::Held)
    );
    let loose = room.target(".codex/AGENTS.md", vec![]);
    room.ledger.turn("codex", false);
    assert_eq!(
        removed(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::Off)
    );
    room.ledger = Ledger::fresh("machine-a");
    assert_eq!(
        removed(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::NoConsent)
    );
    assert_eq!(read(&placed), "Rules.\n");
}

fn loose_under(room: &Room, root: &str, at: &str) -> Target {
    let mut loose = room.target(at, vec![]);
    loose.root = room.home.path().join(root);
    std::fs::create_dir_all(loose.at.parent().unwrap()).unwrap();
    std::fs::write(&loose.at, "Tisty's.\n").unwrap();
    loose
}

#[test]
fn an_emptied_skill_folder_goes_and_every_folder_the_assistant_or_person_keeps_stays() {
    let mut room = room();
    let home = room.home.path().to_path_buf();
    let skill = loose_under(&room, ".claude", ".claude/skills/tidy/SKILL.md");
    let rule = loose_under(&room, ".claude", ".claude/rules/last.md");
    let mixed = loose_under(&room, ".claude", ".claude/skills/mixed/SKILL.md");
    std::fs::write(home.join(".claude/skills/mixed/notes.txt"), "Mine.\n").unwrap();
    let shared = loose_under(&room, ".agents", ".agents/skills/k/SKILL.md");
    let alone = loose_under(&room, ".codex", ".codex/AGENTS.md");
    for loose in [&skill, &rule, &mixed, &shared, &alone] {
        removed(room.data.path(), &mut room.ledger, loose).unwrap();
        assert!(!loose.at.exists(), "{}", loose.at.display());
    }
    assert!(!home.join(".claude/skills/tidy").exists());
    assert!(!home.join(".agents/skills/k").exists());
    for stays in [
        ".claude",
        ".claude/skills",
        ".claude/rules",
        ".claude/skills/mixed",
        ".agents",
        ".agents/skills",
        ".codex",
    ] {
        assert!(home.join(stays).is_dir(), "{stays} stays");
    }
    assert_eq!(
        std::fs::read_to_string(home.join(".claude/skills/mixed/notes.txt")).unwrap(),
        "Mine.\n"
    );
}

#[test]
fn a_relative_folder_is_never_written_nor_emptied() {
    let mut room = room();
    let mut target = room.target(
        ".claude/CLAUDE.md",
        vec![rule("claude", "CLAUDE.md", "X.\n")],
    );
    target.root = PathBuf::from(".claude");
    target.at = PathBuf::from(".claude").join("CLAUDE.md");
    assert_eq!(
        room.round(std::slice::from_ref(&target)),
        [Standing::Linked]
    );
    let mut loose = target.clone();
    loose.pieces.clear();
    assert_eq!(
        removed(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::Linked)
    );
    let mut rooted = loose_under(&room, ".claude", ".claude/skills/tidy/SKILL.md");
    std::fs::remove_file(&rooted.at).unwrap();
    rooted.root = PathBuf::from(".claude");
    emptied_up(&rooted);
    assert!(room.home.path().join(".claude/skills/tidy").is_dir());
}

#[test]
fn a_linked_skill_folder_is_never_removed_even_when_what_it_points_at_is_empty() {
    let room = room();
    let home = room.home.path();
    let elsewhere = home.join("dotfiles").join("demo");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::write(elsewhere.join("a.md"), "A.\n").unwrap();
    std::fs::write(elsewhere.join("b.md"), "B.\n").unwrap();
    std::fs::create_dir_all(home.join(".claude/skills")).unwrap();
    let demo = home.join(".claude").join("skills").join("demo");
    linked_folder(&demo, &elsewhere);

    let mut loose = room.target(".claude/skills/demo/a.md", vec![]);
    loose.root = home.join(".claude");
    std::fs::remove_file(&loose.at).unwrap();
    emptied_up(&loose);
    assert!(std::fs::symlink_metadata(&demo).is_ok(), "the link stays");
    assert!(elsewhere.join("b.md").exists());

    std::fs::remove_file(elsewhere.join("b.md")).unwrap();
    emptied_up(&loose);
    assert!(
        std::fs::symlink_metadata(&demo).is_ok(),
        "an empty target keeps its link"
    );
    assert!(elsewhere.is_dir());
}

#[test]
fn a_loose_file_behind_a_link_is_never_taken_away() {
    let mut room = room();
    let mut loose = room.target(".codex/AGENTS.md", vec![]);
    loose.at = room.home.path().join("..").join("AGENTS.md");
    assert_eq!(
        removed(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::Linked)
    );
}

#[test]
fn only_tistys_import_line_leaves_the_agents_own_file() {
    let room = room();
    let target = room.target(".gemini/GEMINI.md", vec![]);
    let line = "@tisty-rules.md";
    assert!(lacks(&target, line));
    assert_eq!(
        lead_out(room.data.path(), &room.ledger, &target, line),
        Ok(false)
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "- likes tea\r\n@tisty-rules.md\r\n- and rain").unwrap();
    assert!(!lacks(&target, line));

    assert_eq!(
        lead_out(room.data.path(), &room.ledger, &target, line),
        Ok(true)
    );
    assert_eq!(read(&target), "- likes tea\r\n- and rain");
    assert!(lacks(&target, line));
    assert_eq!(
        lead_out(room.data.path(), &room.ledger, &target, line),
        Ok(false)
    );
    assert_eq!(backups(room.data.path(), &target.at).len(), 1);

    std::fs::write(&target.at, [0xff, 0xfe]).unwrap();
    assert!(
        !lacks(&target, line),
        "a file Tisty cannot read is not taken for one without it"
    );
    assert_eq!(
        lead_out(room.data.path(), &room.ledger, &target, line),
        Err(Refused::NotText)
    );
    let stranger = Ledger::fresh("machine-a");
    assert_eq!(
        lead_out(room.data.path(), &stranger, &target, line),
        Err(Refused::NoConsent)
    );
}

fn gemini(room: &Room) -> Target {
    let mut target = room.target(".gemini/GEMINI.md", Vec::new());
    target.assistant = "gemini".into();
    target
}

#[test]
fn geminis_file_changed_between_reading_and_adding_the_line_is_left_as_gemini_wrote_it() {
    let room = room();
    let target = gemini(&room);
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "- remembered just now\n").unwrap();

    let read_before = Some(b"- older memory\n".to_vec());
    assert_eq!(
        led_over(room.data.path(), &target, "@tisty-rules.md", read_before),
        Err(Refused::Changed)
    );
    assert_eq!(read(&target), "- remembered just now\n");
    assert!(backups(room.data.path(), &target.at).is_empty());

    assert_eq!(
        led_over(room.data.path(), &target, "@tisty-rules.md", None),
        Err(Refused::Changed),
        "a file that appeared since it was read missing is not written over either"
    );
}

#[test]
fn geminis_file_changed_between_reading_and_taking_the_line_out_is_left_as_gemini_wrote_it() {
    let room = room();
    let target = gemini(&room);
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "@tisty-rules.md\n- remembered just now\n").unwrap();

    let read_before = b"@tisty-rules.md\n".to_vec();
    assert_eq!(
        unled_over(room.data.path(), &target, "@tisty-rules.md", read_before),
        Err(Refused::Changed)
    );
    assert_eq!(read(&target), "@tisty-rules.md\n- remembered just now\n");

    let now = std::fs::read(&target.at).unwrap();
    assert_eq!(
        unled_over(room.data.path(), &target, "@tisty-rules.md", now),
        Ok(true)
    );
    assert_eq!(read(&target), "- remembered just now\n");
    assert_eq!(backups(room.data.path(), &target.at).len(), 1);
}

#[cfg(windows)]
#[test]
fn a_locked_file_tried_again_and_again_keeps_one_backup_of_what_it_held() {
    use std::os::windows::fs::OpenOptionsExt;
    let mut room = room();
    let target = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "New.\n")],
    );
    std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
    std::fs::write(&target.at, "Written by hand.\n").unwrap();
    let locked = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&target.at)
        .unwrap();

    for _ in 0..3 {
        assert!(overwrite(room.data.path(), &mut room.ledger, &target).is_err());
    }
    assert_eq!(backups(room.data.path(), &target.at).len(), 1);

    drop(locked);
    overwrite(room.data.path(), &mut room.ledger, &target).unwrap();
    assert_eq!(read(&target), "New.\n");
    assert_eq!(backups(room.data.path(), &target.at).len(), 1);
}

const OURS: &str = "tisty:skill/tisty/SKILL.md";

fn ours(said: &str) -> Admitted {
    let body = doc_of(GLOBAL, Kind::Skill, "tisty/SKILL.md", said.as_bytes()).unwrap();
    Admitted::trusted(OURS, &body).unwrap()
}

impl Room {
    fn shipped(&self, said: &str) -> Target {
        Target {
            pieces: vec![ours(said)],
            ..self.target(".claude/skills/tisty/SKILL.md", Vec::new())
        }
    }

    fn put(&self, target: &Target, said: &str) {
        std::fs::create_dir_all(target.at.parent().unwrap()).unwrap();
        std::fs::write(&target.at, said).unwrap();
    }

    fn one_round(&mut self, target: &Target) -> Standing {
        self.round(std::slice::from_ref(target)).remove(0)
    }
}

#[test]
fn tistys_own_skill_is_written_where_nothing_is_and_kept_up_to_date() {
    let mut room = room();
    let target = room.shipped("Tisty 1.50.\n");
    assert!(target.shipped());
    assert_eq!(room.one_round(&target), Standing::Wrote);
    assert_eq!(read(&target), "Tisty 1.50.\n");
    assert_eq!(room.one_round(&target), Standing::UpToDate);
    assert!(
        !room.ledger.took(OURS),
        "no document is taken by Tisty's own skill"
    );
}

#[test]
fn a_newer_release_renews_a_copy_still_as_tisty_wrote_it() {
    let mut room = room();
    room.one_round(&room.shipped("Tisty 1.50.\n"));
    let newer = room.shipped("Tisty 1.51.\n");
    assert_eq!(room.one_round(&newer), Standing::Wrote);
    assert_eq!(read(&newer), "Tisty 1.51.\n");
}

#[test]
fn bytes_tisty_never_wrote_are_foreign_even_when_they_read_the_same() {
    for there in ["Someone else's skill.\n", "Tisty 1.50.\n"] {
        let mut room = room();
        let target = room.shipped("Tisty 1.50.\n");
        room.put(&target, there);
        assert_eq!(
            room.one_round(&target),
            Standing::Foreign { changed: false }
        );
        assert_eq!(read(&target), there);
        assert_eq!(
            room.ledger.written(&target.at),
            None,
            "never recorded as Tisty's"
        );
        assert!(backups(room.data.path(), &target.at).is_empty());
    }
}

#[test]
fn a_copy_changed_or_taken_away_by_the_person_is_left_and_never_put_to_them_as_both() {
    let mut room = room();
    let target = room.shipped("Tisty 1.50.\n");
    room.one_round(&target);
    room.put(&target, "Tisty 1.50, with my line.\n");
    let newer = room.shipped("Tisty 1.51.\n");
    assert_eq!(room.one_round(&newer), Standing::Foreign { changed: true });
    assert_eq!(read(&newer), "Tisty 1.50, with my line.\n");

    std::fs::remove_file(&target.at).unwrap();
    assert_eq!(room.one_round(&newer), Standing::Foreign { changed: true });
    assert!(!target.at.exists());
}

#[test]
fn a_file_a_document_of_the_persons_wrote_is_never_taken_by_tistys_own_skill() {
    let mut room = room();
    let theirs = room.target(
        ".claude/skills/tisty/SKILL.md",
        vec![rule(GLOBAL, "tisty/SKILL.md", "Mine.\n")],
    );
    room.one_round(&theirs);
    let target = room.shipped("Tisty 1.50.\n");
    assert_eq!(
        room.one_round(&target),
        Standing::Foreign { changed: false }
    );
    assert_eq!(read(&target), "Mine.\n");
    assert_eq!(
        overwrite(room.data.path(), &mut room.ledger, &target),
        Err(Refused::Foreign)
    );
    assert_eq!(read(&target), "Mine.\n");
}

#[test]
fn a_file_an_earlier_build_wrote_without_a_record_is_foreign() {
    let mut room = room();
    let target = room.shipped("Tisty 1.50.\n");
    room.put(&target, "Tisty 1.50.\n");
    room.ledger.settle(&target.at, b"Tisty 1.50.\n");
    assert_eq!(
        room.one_round(&target),
        Standing::Foreign { changed: false }
    );
}

#[test]
fn a_pause_in_agents_never_reaches_tistys_own_skill_and_only_their_switch_stops_it() {
    let mut room = room();
    let target = room.shipped("Tisty 1.50.\n");
    room.ledger.turn("codex", false);
    room.ledger.turn_all(false);
    assert_eq!(room.one_round(&target), Standing::Wrote);
    let newer = room.shipped("Tisty 1.51.\n");
    assert_eq!(
        room.one_round(&newer),
        Standing::Wrote,
        "paused or not, it updates"
    );
    let gone = Target {
        pieces: Vec::new(),
        ..newer.clone()
    };
    room.ledger.take_tisty_skills(false);
    assert_eq!(room.one_round(&newer), Standing::NoConsent);
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &gone),
        Ok(true)
    );
    assert!(
        !newer.at.exists(),
        "turning them off takes it away, paused or not"
    );
}

#[test]
fn a_pause_in_agents_still_holds_the_persons_own_documents() {
    let mut room = room();
    room.ledger.turn("codex", false);
    let theirs = room.target(
        ".codex/AGENTS.md",
        vec![rule(GLOBAL, "AGENTS.md", "Mine.\n")],
    );
    assert_eq!(room.one_round(&theirs), Standing::Off);
    assert!(!theirs.at.exists());
}

#[test]
fn with_agents_off_their_own_yes_writes_them_and_a_pause_from_before_counts_no_more() {
    let mut room = room();
    room.ledger.turn("codex", false);
    room.ledger.consent(false);
    let target = room.shipped("Tisty 1.50.\n");
    assert_eq!(room.one_round(&target), Standing::Wrote);
    assert_eq!(read(&target), "Tisty 1.50.\n");
    let theirs = room.target(
        ".codex/AGENTS.md",
        vec![rule(GLOBAL, "AGENTS.md", "Mine.\n")],
    );
    assert_eq!(
        room.one_round(&theirs),
        Standing::NoConsent,
        "documents still need Agents' yes"
    );
}

#[test]
fn a_copy_is_taken_away_on_a_computer_that_names_itself_even_after_their_yes_is_gone() {
    let mut room = room();
    let skill = room.shipped("Tisty 1.50.\n");
    room.one_round(&skill);
    let gone = Target {
        pieces: Vec::new(),
        ..skill.clone()
    };
    room.ledger.take_tisty_skills(false);
    room.ledger.consent(false);
    room.ledger.turn("codex", false);
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &gone),
        Ok(true)
    );
    assert!(!skill.at.exists());
}

#[test]
fn turning_them_off_takes_away_only_a_copy_still_as_tisty_wrote_it_with_a_backup() {
    let mut room = room();
    let skill = room.shipped("Tisty 1.50.\n");
    room.one_round(&skill);
    let gone = Target {
        pieces: Vec::new(),
        ..skill.clone()
    };
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &gone),
        Ok(true)
    );
    assert!(!skill.at.exists());
    assert!(
        !skill.at.parent().unwrap().exists(),
        "its emptied folder goes too"
    );
    assert_eq!(room.ledger.written(&skill.at), None);
    assert_eq!(
        read_kept(room.data.path(), &skill.at, 1).unwrap(),
        b"Tisty 1.50.\n"
    );
}

#[test]
fn turning_them_off_leaves_a_changed_copy_loose_and_forgets_one_already_gone() {
    let mut room = room();
    let skill = room.shipped("Tisty 1.50.\n");
    room.one_round(&skill);
    room.put(&skill, "Mine now.\n");
    let gone = Target {
        pieces: Vec::new(),
        ..skill.clone()
    };
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &gone),
        Ok(false)
    );
    assert_eq!(read(&skill), "Mine now.\n");
    assert!(
        room.ledger.written(&skill.at).is_some(),
        "it stays loose, for the person"
    );

    std::fs::remove_file(&skill.at).unwrap();
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &gone),
        Ok(false)
    );
    assert_eq!(room.ledger.written(&skill.at), None);
}

#[test]
fn turning_them_off_never_touches_a_file_a_document_placed_or_still_places() {
    let mut room = room();
    let theirs = room.target(
        ".claude/skills/tisty/SKILL.md",
        vec![rule(GLOBAL, "tisty/SKILL.md", "Mine.\n")],
    );
    room.one_round(&theirs);
    let loose = Target {
        pieces: Vec::new(),
        ..theirs.clone()
    };
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::Held)
    );
    let skill = room.shipped("Tisty 1.50.\n");
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &skill),
        Err(Refused::Held)
    );
    assert_eq!(read(&theirs), "Mine.\n");
    room.ledger.turn("codex", false);
    room.ledger.consent(false);
    assert_eq!(
        unshipped(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::Held),
        "whatever the yes, a document's file is never Tisty's to take"
    );
    assert_eq!(read(&theirs), "Mine.\n");
}

const NEXT: &str = "writing-for-the-next-agent";

impl Room {
    fn shipped_as(&self, name: &str, said: &str) -> Target {
        let body = doc_of(GLOBAL, Kind::Skill, name, said.as_bytes()).unwrap();
        Target {
            pieces: vec![Admitted::trusted(&format!("{BUILTIN}{name}"), &body).unwrap()],
            ..self.target(&format!(".agents/skills/{name}"), Vec::new())
        }
    }

    fn next_agent(&self, skill: &str, licence: &str) -> [Target; 2] {
        [
            self.shipped_as(&format!("{NEXT}/LICENSE"), licence),
            self.shipped_as(&format!("{NEXT}/SKILL.md"), skill),
        ]
    }
}

#[test]
fn a_file_of_the_persons_in_a_skills_folder_keeps_all_of_tistys_skill_out() {
    let mut room = room();
    let [licence, skill] = room.next_agent("Tisty's.\n", "MIT.\n");
    room.put(&skill, "My own skill.\n");
    assert_eq!(
        room.round(&[licence.clone(), skill.clone()]),
        vec![Standing::Foreign { changed: false }; 2]
    );
    assert!(
        !licence.at.exists(),
        "no licence lands beside a skill that is not Tisty's"
    );
    assert_eq!(read(&skill), "My own skill.\n");
}

#[test]
fn a_stray_file_beside_tistys_skill_holds_it_as_it_is() {
    let mut room = room();
    let both = room.next_agent("Tisty 1.50.\n", "MIT.\n");
    room.round(&both);
    let stray = both[0].at.with_file_name("notes.md");
    std::fs::write(&stray, "Mine.\n").unwrap();
    let newer = room.next_agent("Tisty 1.51.\n", "MIT.\n");
    assert_eq!(
        room.round(&newer),
        vec![Standing::Foreign { changed: false }; 2]
    );
    assert_eq!(read(&newer[1]), "Tisty 1.50.\n");
    assert_eq!(std::fs::read_to_string(stray).unwrap(), "Mine.\n");
}

#[test]
fn a_copy_the_person_changed_holds_the_rest_of_its_skill() {
    let mut room = room();
    room.round(&room.next_agent("Tisty 1.50.\n", "MIT.\n"));
    let newer = room.next_agent("Tisty 1.50.\n", "MIT, 2027.\n");
    room.put(&newer[1], "Tisty 1.50, with my line.\n");
    assert_eq!(
        room.round(&newer),
        vec![Standing::Foreign { changed: true }; 2]
    );
    assert_eq!(read(&newer[0]), "MIT.\n");
}

#[test]
fn a_file_an_earlier_release_wrote_and_this_one_dropped_still_lets_it_update() {
    let mut room = room();
    let old = room.shipped_as(&format!("{NEXT}/OLD.md"), "Gone in 1.51.\n");
    let [_, skill] = room.next_agent("Tisty 1.50.\n", "MIT.\n");
    room.round(&[old.clone(), skill]);
    let [licence, skill] = room.next_agent("Tisty 1.51.\n", "MIT.\n");
    assert_eq!(
        room.round(&[licence.clone(), skill.clone()]),
        vec![Standing::Wrote; 2]
    );
    assert_eq!(read(&skill), "Tisty 1.51.\n");
    assert_eq!(read(&licence), "MIT.\n");
    assert_eq!(read(&old), "Gone in 1.51.\n");
}

#[test]
fn folders_nested_too_deep_beside_tistys_skill_count_as_not_tistys() {
    let mut room = room();
    let [licence, skill] = room.next_agent("Tisty's.\n", "MIT.\n");
    let deep = skill.at.with_file_name("a/b/c/d/e");
    std::fs::create_dir_all(&deep).unwrap();
    assert_eq!(
        room.round(&[licence.clone(), skill]),
        vec![Standing::Foreign { changed: false }; 2]
    );
    assert!(!licence.at.exists());
    std::fs::remove_dir_all(licence.at.with_file_name("a/b/c/d/e")).unwrap();
    assert_eq!(
        room.round(std::slice::from_ref(&licence)),
        vec![Standing::Wrote]
    );
}

#[test]
fn a_document_over_a_path_tistys_skill_wrote_still_needs_agents_yes() {
    let mut room = room();
    room.ledger.consent(false);
    let skill = room.shipped("Tisty 1.50.\n");
    assert_eq!(room.one_round(&skill), Standing::Wrote);
    let theirs = room.target(
        ".claude/skills/tisty/SKILL.md",
        vec![rule(GLOBAL, "tisty/SKILL.md", "Mine.\n")],
    );
    assert_eq!(room.one_round(&theirs), Standing::NoConsent);
    assert_eq!(read(&skill), "Tisty 1.50.\n");
}

#[test]
fn with_agents_yes_a_document_over_a_path_tistys_skill_wrote_is_not_refused_for_their_yes() {
    let mut room = room();
    let skill = room.shipped("Tisty 1.50.\n");
    room.one_round(&skill);
    room.ledger.take_tisty_skills(false);
    let theirs = room.target(
        ".claude/skills/tisty/SKILL.md",
        vec![rule(GLOBAL, "tisty/SKILL.md", "Mine.\n")],
    );
    assert_ne!(room.one_round(&theirs), Standing::NoConsent);
}

#[test]
fn a_junction_on_the_way_to_tistys_skill_is_never_written_through() {
    let mut room = room();
    let elsewhere = tempfile::tempdir().unwrap();
    let skills = room.home.path().join(".claude").join("skills");
    std::fs::create_dir_all(&skills).unwrap();
    linked_folder(&skills.join("tisty"), elsewhere.path());
    let target = room.shipped("Tisty 1.50.");
    assert_eq!(room.one_round(&target), Standing::Linked);
    assert!(
        std::fs::read_dir(elsewhere.path())
            .unwrap()
            .next()
            .is_none()
    );
}

#[test]
fn a_copy_with_the_very_bytes_this_release_ships_is_claimed_back_when_the_ledger_forgot_it() {
    let mut room = room();
    let target = room.shipped("Tisty 1.50.\n");
    room.put(&target, "Tisty 1.50.\n");
    assert_eq!(
        room.one_round(&target),
        Standing::Foreign { changed: false }
    );
    assert!(reclaimed(&mut room.ledger, std::slice::from_ref(&target)));
    assert_eq!(room.one_round(&target), Standing::UpToDate);
    let newer = room.shipped("Tisty 1.51.\n");
    assert_eq!(
        room.one_round(&newer),
        Standing::Wrote,
        "Tisty's again, it updates"
    );
}

#[test]
fn only_the_very_bytes_are_claimed_back_and_only_with_their_yes() {
    let mut mine = room();
    let target = mine.shipped("Tisty 1.50.\n");
    mine.put(&target, "Tisty 1.50, and mine.\n");
    assert!(!reclaimed(&mut mine.ledger, std::slice::from_ref(&target)));
    assert_eq!(
        mine.one_round(&target),
        Standing::Foreign { changed: false }
    );

    let mut off = room();
    let target = off.shipped("Tisty 1.50.\n");
    off.put(&target, "Tisty 1.50.\n");
    off.ledger.take_tisty_skills(false);
    assert!(!reclaimed(&mut off.ledger, std::slice::from_ref(&target)));
    assert_eq!(off.ledger.written(&target.at), None);
}

#[test]
fn a_path_a_document_wrote_is_never_claimed_back() {
    let mut room = room();
    let theirs = room.target(
        ".claude/skills/tisty/SKILL.md",
        vec![rule(GLOBAL, "tisty/SKILL.md", "Tisty 1.50.\n")],
    );
    room.one_round(&theirs);
    let target = room.shipped("Tisty 1.50.\n");
    assert!(!reclaimed(&mut room.ledger, std::slice::from_ref(&target)));
    assert!(
        !room
            .ledger
            .applied_at(&target.at)
            .any(crate::agents::builtin)
    );
}

#[test]
fn a_whole_skill_is_claimed_back_or_none_of_it() {
    let mut room = room();
    let [licence, skill] = room.next_agent("Tisty's.\n", "MIT.\n");
    room.put(&skill, "Tisty's.\n");
    room.put(&licence, "MIT, my way.\n");
    assert!(!reclaimed(
        &mut room.ledger,
        &[licence.clone(), skill.clone()]
    ));
    assert_eq!(
        room.ledger.written(&skill.at),
        None,
        "its SKILL.md alone is never claimed"
    );
    assert_eq!(
        room.round(&[licence.clone(), skill.clone()]),
        vec![Standing::Foreign { changed: false }; 2]
    );

    std::fs::remove_file(&licence.at).unwrap();
    assert!(
        !reclaimed(&mut room.ledger, &[licence.clone(), skill.clone()]),
        "nor with its licence gone"
    );

    room.put(&licence, "MIT.\n");
    let stray = skill.at.with_file_name("notes.md");
    std::fs::write(&stray, "Mine.\n").unwrap();
    assert!(
        !reclaimed(&mut room.ledger, &[licence.clone(), skill.clone()]),
        "nor beside a file of the person's"
    );

    std::fs::remove_file(&stray).unwrap();
    assert!(reclaimed(
        &mut room.ledger,
        &[licence.clone(), skill.clone()]
    ));
    assert!(room.ledger.written(&licence.at).is_some() && room.ledger.written(&skill.at).is_some());
    assert_eq!(room.round(&[licence, skill]), vec![Standing::UpToDate; 2]);
}

#[test]
fn taken_away_unseen_only_a_copy_still_as_tisty_wrote_it_goes_with_a_backup() {
    let mut room = room();
    let placed = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&placed));
    let loose = room.target(".codex/AGENTS.md", vec![]);
    std::fs::write(&loose.at, "Changed by hand.\n").unwrap();
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &loose),
        Ok(false)
    );
    assert_eq!(read(&loose), "Changed by hand.\n");
    assert!(backups(room.data.path(), &loose.at).is_empty());
    assert!(
        room.ledger.written(&loose.at).is_some(),
        "it stays loose for the person to see"
    );

    std::fs::write(&loose.at, "Rules.\n").unwrap();
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &loose),
        Ok(true)
    );
    assert!(!loose.at.exists());
    assert_eq!(backups(room.data.path(), &loose.at).len(), 1);
    assert_eq!(room.ledger.written(&loose.at), None);
}

#[test]
fn taken_away_unseen_never_reads_nor_deletes_through_a_real_link() {
    let mut room = room();
    let home = room.home.path().to_path_buf();
    let elsewhere = home.join("dotfiles").join("voice");
    std::fs::create_dir_all(&elsewhere).unwrap();
    std::fs::write(elsewhere.join("SKILL.md"), "Speak plainly.\n").unwrap();
    let skills = home.join(".claude").join("skills");
    std::fs::create_dir_all(&skills).unwrap();
    linked_folder(&skills.join("voice"), &elsewhere);
    let loose = Target {
        root: home.join(".claude"),
        at: skills.join("voice").join("SKILL.md"),
        ..room.target("unused", vec![])
    };
    room.ledger.settle(&loose.at, b"Speak plainly.\n");
    room.ledger.applied_to(&loose.at, ["doc-voice".to_string()]);
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::Linked)
    );
    assert_eq!(
        std::fs::read_to_string(elsewhere.join("SKILL.md")).unwrap(),
        "Speak plainly.\n"
    );
    assert!(backups(room.data.path(), &loose.at).is_empty());
    assert!(room.ledger.written(&loose.at).is_some());
}

#[test]
fn a_file_left_with_its_agent_is_asked_about_before_a_document_writes_over_it() {
    let mut room = room();
    let first = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&first));
    room.ledger.stay(&first.at);
    let again = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Other rules.\n")],
    );
    assert!(matches!(
        room.round(std::slice::from_ref(&again))[0],
        Standing::Both { .. }
    ));
    assert_eq!(
        read(&again),
        "Rules.\n",
        "nothing is written over it unasked"
    );
    assert!(room.ledger.stays(&again.at));

    let same = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&same));
    assert!(
        !room.ledger.stays(&same.at),
        "the same words placed again make it Tisty's once more"
    );
}

#[test]
fn taken_away_unseen_keeps_every_refusal_of_taking_away() {
    let mut room = room();
    let placed = room.target(
        ".codex/AGENTS.md",
        vec![rule("codex", "AGENTS.md", "Rules.\n")],
    );
    room.round(std::slice::from_ref(&placed));
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &placed),
        Err(Refused::Held)
    );
    let mut held = room.target(".codex/AGENTS.md", vec![]);
    held.held = true;
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &held),
        Err(Refused::Held)
    );
    let mut linked = room.target(".codex/AGENTS.md", vec![]);
    linked.at = room.home.path().join("..").join("AGENTS.md");
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &linked),
        Err(Refused::Linked)
    );
    let loose = room.target(".codex/AGENTS.md", vec![]);
    room.ledger.turn("codex", false);
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::Off)
    );
    room.ledger = Ledger::fresh("machine-a");
    assert_eq!(
        removed_as_written(room.data.path(), &mut room.ledger, &loose),
        Err(Refused::NoConsent)
    );
    assert_eq!(read(&placed), "Rules.\n");
    assert!(backups(room.data.path(), &placed.at).is_empty());
}
