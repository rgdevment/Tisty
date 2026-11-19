use super::*;
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
    let global = ("global.md", rule("global", "AGENTS.md", "Be brief.\n"));
    let own = ("codex.md", rule("codex", "AGENTS.md", "Use Rust.\n"));
    let claude = room.placed("claude/CLAUDE.md", std::slice::from_ref(&global));
    let codex = room.placed("codex/AGENTS.md", &[global.clone(), own]);
    let found = placed_for(&id_of("global.md"), vec![claude, codex.clone()]);
    assert_eq!(found.len(), 2);
    assert_eq!(found[1], (codex.target, 0));
    assert_eq!(
        placed_for(&id_of("codex.md"), vec![codex_of(&room)])[0].1,
        1
    );
    assert!(placed_for(&id_of("elsewhere.md"), vec![codex_of(&room)]).is_empty());
}

fn codex_of(room: &Room) -> Placed {
    room.placed(
        "codex/AGENTS.md",
        &[
            ("global.md", rule("global", "AGENTS.md", "Be brief.\n")),
            ("codex.md", rule("codex", "AGENTS.md", "Use Rust.\n")),
        ],
    )
}

#[test]
fn the_card_says_in_sync_then_changed_outside_and_whose_change_it_is() {
    let mut room = room();
    let codex = codex_of(&room);
    room.written(std::slice::from_ref(&codex));
    assert_eq!(room.place(&codex, 0).said, Said::Synced);
    assert!(room.place(&codex, 0).changed.is_some());
    std::fs::write(&codex.target.at, "Be brief.\n\nUse Go.\n").unwrap();
    assert_eq!(
        room.place(&codex, 1).said,
        Said::Outside {
            mine: true,
            gone: false
        }
    );
    assert_eq!(
        room.place(&codex, 0).said,
        Said::Outside {
            mine: false,
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
    assert_eq!(shown.shared, 2);
    assert_eq!(
        shown.kept,
        vec![Backup {
            n: 1,
            bytes: "Written by hand.\n".len() as u64
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
