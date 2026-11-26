use super::{Ledger, restored_here};
use crate::answers::agents_sync::round_now;
use crate::answers::agents_sync::tests::{HERE, Room, doc_here, put, room};
use tisty_core::agents::{GLOBAL, Kind};

const CLAUDE_SKILL: &str = ".claude/skills/voice/SKILL.md";
const SHARED_SKILL: &str = ".agents/skills/voice/SKILL.md";
const SAID: &str = "Speak plainly.\n";

fn written_everywhere() -> Room {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md", ".claude/rules/tone.md"]);
    doc_here(&mut room, GLOBAL, Kind::Skill, "voice/SKILL.md", SAID);
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    round_now(&room.session, &room.homes, &mut ledger);
    assert_eq!(room.read(CLAUDE_SKILL), SAID);
    assert_eq!(room.read(SHARED_SKILL), SAID);
    assert_eq!(room.look().vanished, 0);
    room
}

#[test]
fn the_look_counts_only_files_taken_away_whole() {
    let room = written_everywhere();
    std::fs::remove_file(room.at(CLAUDE_SKILL)).unwrap();
    assert_eq!(room.look().vanished, 1);

    put(
        std::path::Path::new(&room.at(SHARED_SKILL)),
        "My own voice.\n",
    );
    assert_eq!(
        room.look().vanished,
        1,
        "an edited file is not a vanished one"
    );
}

#[test]
fn restoring_brings_back_the_vanished_file_and_leaves_an_edited_one_alone() {
    let room = written_everywhere();
    std::fs::remove_file(room.at(CLAUDE_SKILL)).unwrap();
    put(
        std::path::Path::new(&room.at(SHARED_SKILL)),
        "My own voice.\n",
    );

    let out = restored_here(&room.session, &room.homes, HERE);
    assert_eq!((out.wrote, out.failed), (1, 0));
    assert_eq!(room.read(CLAUDE_SKILL), SAID);
    assert_eq!(room.read(SHARED_SKILL), "My own voice.\n");
    assert_eq!(room.look().vanished, 0);

    let again = restored_here(&room.session, &room.homes, HERE);
    assert_eq!((again.wrote, again.failed), (0, 0));
}
