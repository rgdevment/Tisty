use super::*;
use crate::answers::agents_homes::tests::mine_in;
use crate::answers::agents_sync::tests::{room as session_room, root_folder, sub_of};
use crate::wiring::agents::gated::{gated, id_of};
use tisty_core::agents::{Kind, doc_of, round};

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
    fn placed(&self, file: &str, docs: &[(&str, String)]) -> Placed {
        let named: Vec<(String, String)> = docs
            .iter()
            .map(|(name, body)| (name.to_string(), body.clone()))
            .collect();
        let gated = gated(&self.ledger, &named);
        let home = self.home.path();
        let mut target =
            gated
                .gate
                .target(&self.ledger, home.to_path_buf(), home.join(file), "codex");
        for (name, _) in docs {
            let id = gated.id(name);
            let piece = gated.gate.admitted().iter().find(|one| one.file() == id);
            target.put(piece.unwrap());
        }
        Placed {
            files: target.files().map(str::to_string).collect(),
            target,
            lead_in: None,
        }
    }

    fn written(&mut self, all: &[Placed]) {
        let targets: Vec<Target> = all.iter().map(|one| one.target.clone()).collect();
        round(self.data.path(), &mut self.ledger, &targets);
    }

    fn place(&self, placed: &Placed, piece: usize) -> Place {
        place(self.data.path(), &self.ledger, &placed.target, piece)
    }
}

#[test]
fn a_document_is_shown_in_every_file_it_is_written_into_with_its_place_there() {
    let room = room();
    let skill = (
        "skill.md",
        doc_of("global", Kind::Skill, "pr/SKILL.md", b"Be brief.\n").unwrap(),
    );
    let claude = room.placed("claude/skills/pr/SKILL.md", std::slice::from_ref(&skill));
    let codex = room.placed("codex/skills/pr/SKILL.md", std::slice::from_ref(&skill));
    let found = placed_for(&id_of("skill.md"), vec![claude, codex.clone()]);
    assert_eq!(found.len(), 2);
    assert_eq!(found[1], (codex.target, 0));
    assert!(placed_for(&id_of("elsewhere.md"), vec![codex_of(&room)]).is_empty());
}

fn codex_of(room: &Room) -> Placed {
    room.placed(
        "codex/AGENTS.md",
        &[("codex.md", rule("codex", "AGENTS.md", "Use Rust.\n"))],
    )
}

#[test]
fn the_card_says_in_sync_then_changed_outside_and_whose_change_it_is() {
    let mut room = room();
    let codex = codex_of(&room);
    room.written(std::slice::from_ref(&codex));
    assert_eq!(room.place(&codex, 0).said, Said::Synced);
    assert!(room.place(&codex, 0).changed.is_some());
    std::fs::write(&codex.target.at, "Use Go.\n").unwrap();
    assert_eq!(
        room.place(&codex, 0).said,
        Said::Outside {
            mine: true,
            gone: false
        }
    );
    let (changes, both) = crate::answers::agents_sync::weighed(&room.ledger, &[codex]);
    assert_eq!((changes, both.len()), (1, 0));
}

#[test]
fn overwriting_leaves_a_backup_the_card_counts() {
    let mut room = room();
    let codex = codex_of(&room);
    std::fs::create_dir_all(codex.target.at.parent().unwrap()).unwrap();
    std::fs::write(&codex.target.at, "Written by hand.\n").unwrap();
    let mut ledger = room.ledger.clone();
    agents::overwrite(room.data.path(), &mut ledger, &codex.target).unwrap();
    room.ledger = ledger;
    let shown = room.place(&codex, 0);
    assert_eq!(shown.said, Said::Synced);
    assert_eq!(shown.shared, 1);
    assert_eq!(
        shown.kept,
        vec![Backup {
            n: 1,
            bytes: "Written by hand.\n".len() as u64,
            pinned: true,
        }]
    );
    let read = agents::read_kept(room.data.path(), &codex.target.at, 1).unwrap();
    assert_eq!(read, b"Written by hand.\n");
}

#[test]
fn a_file_removed_paused_or_never_allowed_says_so() {
    let mut room = room();
    let codex = codex_of(&room);
    room.written(std::slice::from_ref(&codex));
    std::fs::remove_file(&codex.target.at).unwrap();
    assert_eq!(
        room.place(&codex, 0).said,
        Said::Outside {
            mine: false,
            gone: true
        }
    );
    room.ledger.turn("codex", false);
    assert_eq!(room.place(&codex, 0).said, Said::Off);
    room.ledger.consent(false);
    assert_eq!(room.place(&codex, 0).said, Said::NoConsent);
}

#[test]
fn every_standing_the_engine_gives_has_a_word_on_the_card() {
    assert_eq!(said(Standing::Write, 0), Said::Pending);
    assert_eq!(said(Standing::Agreed, 0), Said::Synced);
    assert_eq!(said(Standing::Both { now: Vec::new() }, 0), Said::Both);
    assert_eq!(said(Standing::Empty, 0), Said::Empty);
    assert_eq!(said(Standing::Linked, 0), Said::Linked);
    assert_eq!(
        said(Standing::Unreadable("denied".into()), 0),
        Said::Unreadable {
            why: "denied".into()
        }
    );
    assert!(matches!(
        said(
            Standing::Broken {
                piece: 0,
                why: Refused::BrokenFence
            },
            0
        ),
        Said::Broken { .. }
    ));
}

#[test]
fn a_backup_is_seen_then_put_back_and_the_file_reads_as_changed_outside() {
    let mut room = room();
    let codex = codex_of(&room);
    std::fs::create_dir_all(codex.target.at.parent().unwrap()).unwrap();
    std::fs::write(&codex.target.at, "Written by hand.\n").unwrap();
    agents::overwrite(room.data.path(), &mut room.ledger, &codex.target).unwrap();
    let data = room.data.path();
    let seen = done(
        data,
        &room.ledger,
        &codex.target,
        0,
        &Asked::Held { backup: Some(1) },
    )
    .unwrap();
    assert_eq!(seen.as_deref(), Some("Written by hand.\n"));
    let now = done(
        data,
        &room.ledger,
        &codex.target,
        0,
        &Asked::Held { backup: None },
    )
    .unwrap();
    assert!(now.unwrap().contains("Use Rust."));
    assert_eq!(
        done(
            data,
            &room.ledger,
            &codex.target,
            0,
            &Asked::Restore { backup: 1 }
        )
        .unwrap(),
        None
    );
    assert_eq!(
        std::fs::read(&codex.target.at).unwrap(),
        b"Written by hand.\n"
    );
    assert!(matches!(room.place(&codex, 0).said, Said::Outside { .. }));
    assert!(
        done(
            data,
            &room.ledger,
            &codex.target,
            0,
            &Asked::Restore { backup: 9 }
        )
        .is_err()
    );
}

#[test]
fn a_file_too_big_to_draw_is_refused_not_read() {
    let room = room();
    let codex = codex_of(&room);
    std::fs::create_dir_all(codex.target.at.parent().unwrap()).unwrap();
    let big = vec![b'a'; tisty_core::docs::READ_AT_MOST as usize + 1];
    std::fs::write(&codex.target.at, big).unwrap();
    let asked = Asked::Held { backup: None };
    assert!(done(room.data.path(), &room.ledger, &codex.target, 0, &asked).is_err());
}

#[test]
fn a_computer_that_has_not_said_yes_never_shows_the_file() {
    let room = room();
    let codex = codex_of(&room);
    std::fs::create_dir_all(codex.target.at.parent().unwrap()).unwrap();
    std::fs::write(&codex.target.at, "Be brief.\n").unwrap();
    let asked = Asked::Held { backup: None };
    let shown = done(room.data.path(), &room.ledger, &codex.target, 0, &asked);
    assert_eq!(shown.unwrap().as_deref(), Some("Be brief.\n"));
    let stranger = Ledger::fresh("machine-a");
    assert!(done(room.data.path(), &stranger, &codex.target, 0, &asked).is_err());
}

#[test]
fn the_card_weighs_the_whole_file_the_assistant_reads() {
    let mut room = room();
    let codex = codex_of(&room);
    assert_eq!(room.place(&codex, 0).bytes, None);
    room.written(std::slice::from_ref(&codex));
    let whole = std::fs::metadata(&codex.target.at).unwrap().len();
    assert_eq!(whole, "Use Rust.\n".len() as u64);
    assert_eq!(room.place(&codex, 0).bytes, Some(whole));
}

#[test]
fn a_new_rule_for_all_computers_starts_without_a_computer_mark_under_a_free_name() {
    let mut room = session_room();
    room.bring(&[".claude/CLAUDE.md"]);
    let first = rule_for_all(&room.session, "claude").unwrap();
    let card = Card::of(&first).unwrap();
    assert!(!first.contains("on-"), "{first}");
    assert_eq!(card.computer, None);
    assert_eq!(card.address(), format!("claude/rule/{}", card.name));

    let global = sub_of(&room, "Global");
    mine_in(&mut room, global, &first);
    let second = rule_for_all(&room.session, "claude").unwrap();
    let again = Card::of(&second).unwrap();
    assert_ne!(again.address(), card.address());
    assert_eq!(again.computer, None);
    assert!(!second.contains("on-"), "{second}");
}

#[test]
fn a_name_taken_in_any_folder_for_all_is_not_offered_again() {
    let mut room = session_room();
    room.bring(&[".claude/CLAUDE.md"]);
    let first = rule_for_all(&room.session, "claude").unwrap();
    let claude = root_folder(&mut room, "Claude");
    mine_in(&mut room, claude, &first);
    let second = rule_for_all(&room.session, "claude").unwrap();
    assert_ne!(
        Card::of(&second).unwrap().name,
        Card::of(&first).unwrap().name
    );
}

#[test]
fn a_name_this_computer_uses_is_not_offered_for_all_unless_the_agent_reads_one_rules_file() {
    let mut room = session_room();
    room.bring(&[".claude/CLAUDE.md", ".codex/AGENTS.md"]);
    let claude = Card::of(&rule_for_all(&room.session, "claude").unwrap()).unwrap();
    let codex = Card::of(&rule_for_all(&room.session, "codex").unwrap()).unwrap();
    assert_ne!(
        claude.name, "CLAUDE.md",
        "a new rule for all would clash at once"
    );
    assert_eq!(
        codex.name, "AGENTS.md",
        "with one rules file the person chooses"
    );
}

#[test]
fn an_agent_without_rules_or_an_unknown_level_has_no_new_rule_for_all() {
    let mut room = session_room();
    room.bring(&[".claude/CLAUDE.md"]);
    assert_eq!(rule_for_all(&room.session, "cursor"), None);
    assert_eq!(rule_for_all(&room.session, "nobody"), None);
    assert_eq!(rule_for_all(&room.session, ""), None);
}

fn global_docs(room: &crate::answers::agents_sync::tests::Room) -> Vec<String> {
    let global = sub_of(room, "Global");
    room.session
        .state
        .docs
        .values()
        .filter(|one| one.folder == Some(global))
        .map(|one| one.file.clone())
        .collect()
}

#[test]
fn a_new_rule_made_in_global_is_for_all_approved_and_a_second_takes_another_name() {
    let mut room = session_room();
    room.bring(&[".claude/CLAUDE.md"]);

    let first = rule_made(&mut room.session, "claude").unwrap();
    let body = crate::answers::agents_homes::read(&room.session, &first.id).unwrap();
    assert!(!body.contains("on-"), "{body}");
    assert_eq!(Card::of(&body).unwrap().computer, None);
    let kept = crate::answers::agents_homes::kept_of(&room.session, &first.id).unwrap();
    assert_eq!(
        kept.agreed.as_deref(),
        Some(tisty_core::docs::print_of_text(&body).as_str())
    );
    assert!(crate::answers::agents_homes::approved(kept, &body));
    assert_eq!(kept.folder, Some(sub_of(&room, "Global")));
    assert!(global_docs(&room).contains(&first.id));

    let second = rule_made(&mut room.session, "claude").unwrap();
    assert_ne!(second.id, first.id);
    let other = crate::answers::agents_homes::read(&room.session, &second.id).unwrap();
    assert_ne!(
        Card::of(&other).unwrap().address(),
        Card::of(&body).unwrap().address()
    );
    assert_eq!(global_docs(&room).len(), 2);
}

#[test]
fn a_new_rule_for_an_agent_with_one_rules_file_opens_the_one_for_all_it_has() {
    let mut room = session_room();
    room.bring(&[".codex/AGENTS.md"]);
    let seed = rule_for_all(&room.session, "codex").unwrap();
    let global = sub_of(&room, "Global");
    let existing = mine_in(&mut room, global, &seed);

    let before = room.session.state.docs.len();
    let opened = rule_made(&mut room.session, "codex").unwrap();
    assert_eq!(opened.id, existing);
    assert_eq!(room.session.state.docs.len(), before);
}

#[test]
fn a_new_rule_for_cursor_or_an_unknown_agent_has_no_place() {
    let mut room = session_room();
    room.bring(&[".claude/CLAUDE.md"]);
    let before = room.session.state.docs.len();
    for level in ["cursor", "nobody", ""] {
        let refused = rule_made(&mut room.session, level).unwrap_err();
        assert_eq!(refused.code, "agentNoPlace", "{level}");
    }
    assert_eq!(room.session.state.docs.len(), before);
}
