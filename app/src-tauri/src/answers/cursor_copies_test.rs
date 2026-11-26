use std::path::Path;

use super::*;
use crate::answers::agents_sync::round_here;
use crate::answers::agents_sync::tests::{HERE, Room, doc_here, loose, put, room};
use crate::answers::tisty_skills::{shipped_round, summed_from, taken_from};
use tisty_core::agents::{Bundled, doc_of};

const PDF: &str = "pdf/SKILL.md";

fn cursor_room() -> Room {
    let mut room = room();
    std::fs::create_dir_all(room.home.join(".cursor")).unwrap();
    room.bring(&[".claude/CLAUDE.md"]);
    room
}

fn under(base: &Path, rel: &str) -> PathBuf {
    let base: PathBuf = base.components().collect();
    rel.split('/').fold(base, |at, step| at.join(step))
}

fn pdf(room: &mut Room, name: &str) -> String {
    doc_here(room, GLOBAL, Kind::Skill, name, "pdf\n")
}

fn ledger(room: &Room) -> Ledger {
    Ledger::open(room.session.paths.data(), HERE)
}

fn round(room: &mut Room) {
    round_here(&mut room.session, &room.homes, HERE).unwrap();
}

// What an earlier build did: wrote its copy in Cursor's own folder and kept it in the ledger.
fn old_copy(room: &Room, rel: &str, bytes: &str, file: &str) -> PathBuf {
    let at = under(&room.home.join(".cursor/skills"), rel);
    put(&at, bytes);
    let mut ledger = ledger(room);
    ledger.settle(&at, bytes.as_bytes());
    ledger.applied_to(&at, [file.to_string()]);
    ledger.keep(room.session.paths.data()).unwrap();
    at
}

fn backups(room: &Room, at: &Path) -> usize {
    agents::kept(room.session.paths.data(), at).len()
}

fn remembered(room: &Room, at: &Path) -> bool {
    ledger(room).written(at).is_some()
}

#[test]
fn an_intact_old_copy_of_a_global_skill_goes_with_a_backup_once_the_shared_one_stands() {
    let mut room = cursor_room();
    let file = pdf(&mut room, PDF);
    round(&mut room);
    let shared = under(&room.home.join(".agents/skills"), PDF);
    assert!(shared.is_file(), "the one copy is in the shared folder");
    let bytes = std::fs::read_to_string(&shared).unwrap();
    let at = old_copy(&room, PDF, &bytes, &file);

    round(&mut room);

    assert!(!at.exists());
    assert!(
        !room.home.join(".cursor/skills/pdf").exists(),
        "no empty folder is left"
    );
    assert_eq!(backups(&room, &at), 1);
    assert!(!remembered(&room, &at), "the ledger forgets it");
    assert!(shared.is_file());
    assert!(!loose(&room).contains(&at.to_string_lossy().into_owned()));
}

#[test]
fn a_changed_old_copy_stays_and_shows_as_loose() {
    let mut room = cursor_room();
    let file = pdf(&mut room, PDF);
    round(&mut room);
    let shared = under(&room.home.join(".agents/skills"), PDF);
    let bytes = std::fs::read_to_string(&shared).unwrap();
    let at = old_copy(&room, PDF, &bytes, &file);
    std::fs::write(&at, "Mine now.\n").unwrap();

    round(&mut room);

    assert_eq!(std::fs::read_to_string(&at).unwrap(), "Mine now.\n");
    assert_eq!(backups(&room, &at), 0);
    assert!(loose(&room).contains(&at.to_string_lossy().into_owned()));
}

#[test]
fn the_whole_skill_folder_stays_when_one_of_its_files_changed() {
    let mut room = cursor_room();
    let skill = pdf(&mut room, PDF);
    let notes = pdf(&mut room, "pdf/notes.md");
    round(&mut room);
    let shared = room.home.join(".agents/skills/pdf");
    let main = old_copy(
        &room,
        PDF,
        &std::fs::read_to_string(shared.join("SKILL.md")).unwrap(),
        &skill,
    );
    let side = old_copy(
        &room,
        "pdf/notes.md",
        &std::fs::read_to_string(shared.join("notes.md")).unwrap(),
        &notes,
    );
    std::fs::write(&side, "Changed by me.\n").unwrap();

    round(&mut room);

    assert!(main.is_file(), "the intact file waits with its folder");
    assert!(side.is_file());
    assert!(remembered(&room, &main));
}

#[test]
fn nothing_is_taken_away_while_cursor_is_paused() {
    let mut room = cursor_room();
    let file = pdf(&mut room, PDF);
    round(&mut room);
    let bytes = std::fs::read_to_string(under(&room.home.join(".agents/skills"), PDF)).unwrap();
    let at = old_copy(&room, PDF, &bytes, &file);
    let mut paused = ledger(&room);
    paused.turn("cursor", false);
    paused.keep(room.session.paths.data()).unwrap();
    assert!(!ledger(&room).syncs("cursor"));

    round(&mut room);

    assert!(at.is_file());
    assert!(remembered(&room, &at));
}

#[test]
fn nothing_is_taken_away_when_the_shared_copy_is_not_one_tisty_wrote() {
    let mut room = cursor_room();
    let file = pdf(&mut room, PDF);
    let shared = under(&room.home.join(".agents/skills"), PDF);
    put(&shared, "A person's own skill.\n");
    let at = old_copy(&room, PDF, "old words\n", &file);

    round(&mut room);

    assert_eq!(
        std::fs::read_to_string(&shared).unwrap(),
        "A person's own skill.\n"
    );
    assert!(
        at.is_file(),
        "the old copy is all that Cursor would still have"
    );
    assert!(remembered(&room, &at));
}

#[test]
fn a_cursor_level_document_keeps_its_file_in_cursors_own_folder() {
    let mut room = cursor_room();
    doc_here(&mut room, "cursor", Kind::Skill, PDF, "for Cursor only\n");
    round(&mut room);
    let at = under(&room.home.join(".cursor/skills"), PDF);
    assert!(at.is_file());

    round(&mut room);

    assert!(remembered(&room, &at));
    assert!(remembered(&room, &at));
    assert!(!under(&room.home.join(".agents/skills"), PDF).exists());
}

fn tisty_skill() -> Vec<Bundled> {
    let name = "tisty/SKILL.md".to_string();
    vec![Bundled {
        skill: "tisty".into(),
        body: doc_of(
            GLOBAL,
            Kind::Skill,
            &name,
            b"---\nname: tisty\n---\n\nWork with Tisty.\n",
        )
        .unwrap(),
        name,
    }]
}

#[test]
fn an_old_cursor_copy_of_a_bundled_skill_goes_when_the_app_opens_and_is_no_longer_left_over() {
    let room = {
        let room = room();
        std::fs::create_dir_all(room.home.join(".cursor")).unwrap();
        room
    };
    let bundled = tisty_skill();
    taken_from(&room.session, &room.homes, HERE, true, &bundled).unwrap();
    let shared = room.home.join(".agents/skills/tisty/SKILL.md");
    assert!(shared.is_file());
    let bytes = std::fs::read_to_string(&shared).unwrap();
    let at = old_copy(&room, "tisty/SKILL.md", &bytes, &bundled[0].file());
    assert!(
        summed_from(&room.session, &room.homes, &ledger(&room), &bundled)
            .leftover
            .iter()
            .any(|one| Path::new(&one.at) == at),
        "before the round it is still told as left over"
    );

    let mut open = ledger(&room);
    shipped_round(&room.session, &room.homes, &mut open, &bundled);

    assert!(!at.exists());
    assert_eq!(backups(&room, &at), 1);
    assert!(!remembered(&room, &at));
    assert!(
        summed_from(&room.session, &room.homes, &ledger(&room), &bundled)
            .leftover
            .iter()
            .all(|one| Path::new(&one.at) != at)
    );
    assert!(shared.is_file());
}
