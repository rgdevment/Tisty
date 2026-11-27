use std::path::PathBuf;

use super::{InDistro, counted, round_in};
use crate::answers::agents_sync::Rounded;
use crate::answers::agents_sync::gated;
use crate::answers::agents_sync::tests::{HERE, Room, put, room};
use crate::wiring::agents::{self as places, Homes};
use tisty_core::agents::{self, Bundled, Ledger};

const DISTRO_ID: &str = "{ABC-12}";

struct Distro {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    homes: Homes,
}

fn distro() -> Distro {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("distro");
    let home = root.join("home").join("mario");
    std::fs::create_dir_all(home.join(".claude")).unwrap();
    let homes = Homes::in_distro(&home, &root).unwrap();
    Distro {
        _tmp: tmp,
        root,
        homes,
    }
}

impl Distro {
    fn at(&self, name: &str) -> PathBuf {
        name.split('/')
            .fold(self.root.clone(), |at, step| at.join(step))
    }

    fn files(&self) -> Vec<String> {
        let mut found = Vec::new();
        let mut folders = vec![self.root.clone()];
        while let Some(folder) = folders.pop() {
            for one in std::fs::read_dir(&folder).unwrap().flatten() {
                match one.path().is_dir() {
                    true => folders.push(one.path()),
                    false => found.push(
                        one.path()
                            .strip_prefix(&self.root)
                            .unwrap()
                            .to_string_lossy()
                            .replace(std::path::MAIN_SEPARATOR, "/"),
                    ),
                }
            }
        }
        found.sort();
        found
    }

    fn read(&self, name: &str) -> Vec<u8> {
        std::fs::read(self.root.join("home").join("mario").join(name)).unwrap()
    }
}

// This computer has approved a rule, a second one and an agent's memory, as bringing them in does.
fn approved_here() -> Room {
    let room = room();
    put(
        &room.home.join(".claude/projects/repo/memory/MEMORY.md"),
        "- a fact\n",
    );
    let mut room = room;
    let brought = room.bring(&[
        ".claude/CLAUDE.md",
        ".claude/rules/tone.md",
        ".claude/projects/repo/memory/MEMORY.md",
    ]);
    assert_eq!(brought.made, 3, "{:?}", brought.refused);
    room
}

fn windows_ledger_bytes(room: &Room) -> Vec<u8> {
    let at = agents::folder(room.session.paths.data()).join("ledger.json");
    std::fs::read(at).unwrap()
}

fn distro_ledger(room: &Room) -> Ledger {
    Ledger::of_distro(room.session.paths.data(), HERE, DISTRO_ID).unwrap()
}

fn round(room: &Room, distro: &Distro, ledger: &mut Ledger) -> Rounded {
    let mine = Ledger::open(room.session.paths.data(), HERE);
    round_in(&room.session, &distro.homes, &mine, ledger, &[])
}

const SKILLS: [(&str, &str); 2] = [
    ("consolidate", "---\nname: consolidate\n---\n\nTidy up.\n"),
    ("tisty", "---\nname: tisty\n---\n\nWork with Tisty.\n"),
];

fn shipped() -> Vec<Bundled> {
    SKILLS
        .iter()
        .map(|(skill, said)| {
            let name = format!("{skill}/SKILL.md");
            Bundled {
                skill: (*skill).into(),
                body: agents::doc_of(agents::GLOBAL, agents::Kind::Skill, &name, said.as_bytes())
                    .unwrap(),
                name,
            }
        })
        .collect()
}

fn standings(round: &Rounded) -> Vec<&str> {
    round.told.iter().map(|one| one.standing).collect()
}

#[test]
fn what_this_computer_took_is_a_real_memory_here_or_the_test_proves_nothing() {
    let room = approved_here();
    let mine = Ledger::open(room.session.paths.data(), HERE);
    let landed = places::targets(&room.homes, &gated(&room.session, &mine), &mine);
    assert!(
        landed
            .iter()
            .any(|one| one.target.at.ends_with("memory/MEMORY.md")),
        "{landed:?}"
    );
}

#[test]
fn with_both_yeses_off_nothing_is_written_in_the_distro() {
    let room = approved_here();
    let distro = distro();
    let mut ledger = distro_ledger(&room);
    assert!(!ledger.consented() && !ledger.tisty_skills());

    let told = round(&room, &distro, &mut ledger);

    assert_eq!(
        distro.files(),
        Vec::<String>::new(),
        "{:?}",
        standings(&told)
    );
    assert!(!standings(&told).contains(&"wrote"));
}

#[test]
fn with_the_distros_yes_the_admitted_documents_are_written_in_linux_bytes_and_no_memory() {
    let room = approved_here();
    let distro = distro();
    let mut ledger = distro_ledger(&room);
    ledger.consent(true);

    let told = round(&room, &distro, &mut ledger);

    assert_eq!(
        distro.files(),
        [
            "home/mario/.claude/CLAUDE.md",
            "home/mario/.claude/rules/tone.md"
        ],
        "{:?}",
        standings(&told)
    );
    assert_eq!(
        distro.read(".claude/CLAUDE.md"),
        room.read(".claude/CLAUDE.md").as_bytes()
    );
    assert!(!distro.read(".claude/rules/tone.md").contains(&b'\r'));
    assert_eq!(
        standings(&told)
            .iter()
            .filter(|one| **one == "wrote")
            .count(),
        2
    );

    let again = round(&room, &distro, &mut ledger);
    assert!(
        standings(&again).iter().all(|one| *one == "upToDate"),
        "{:?}",
        standings(&again)
    );
}

#[test]
fn a_round_in_a_distro_never_changes_the_ledger_of_this_computer() {
    let room = approved_here();
    let distro = distro();
    let before = windows_ledger_bytes(&room);
    let mut ledger = distro_ledger(&room);
    ledger.consent(true);
    ledger.take_tisty_skills(true);

    round(&room, &distro, &mut ledger);
    ledger.keep(room.session.paths.data()).unwrap();

    assert_eq!(windows_ledger_bytes(&room), before);
    let folder = agents::folder(room.session.paths.data());
    assert!(folder.join("ledger.wsl.abc-12.json").exists());
    let mine = Ledger::open(room.session.paths.data(), HERE);
    assert!(mine.consented() && !mine.tisty_skills());
}

#[test]
fn the_skills_yes_alone_writes_only_the_shipped_skills_with_their_exact_bytes() {
    let room = approved_here();
    let distro = distro();
    let mut ledger = distro_ledger(&room);
    ledger.take_tisty_skills(true);
    assert!(!ledger.consented());
    let mine = Ledger::open(room.session.paths.data(), HERE);

    round_in(&room.session, &distro.homes, &mine, &mut ledger, &shipped());

    assert_eq!(
        distro.files(),
        [
            "home/mario/.claude/skills/consolidate/SKILL.md",
            "home/mario/.claude/skills/tisty/SKILL.md"
        ]
    );
    for (skill, said) in SKILLS {
        assert_eq!(
            distro.read(&format!(".claude/skills/{skill}/SKILL.md")),
            said.as_bytes()
        );
    }
}

#[test]
fn the_shipped_skills_are_not_written_without_their_yes() {
    let room = approved_here();
    let distro = distro();
    let mut ledger = distro_ledger(&room);
    ledger.consent(true);
    let mine = Ledger::open(room.session.paths.data(), HERE);

    round_in(&room.session, &distro.homes, &mine, &mut ledger, &shipped());

    assert!(
        distro
            .files()
            .iter()
            .all(|one| !one.contains("/skills/tisty"))
    );
}

#[test]
fn a_change_made_inside_the_distro_is_left_as_it_is_and_never_brought_back() {
    let room = approved_here();
    let distro = distro();
    let mut ledger = distro_ledger(&room);
    ledger.consent(true);
    round(&room, &distro, &mut ledger);

    let at = distro.at("home/mario/.claude/CLAUDE.md");
    std::fs::write(&at, "Changed inside the distro.\n").unwrap();
    let told = round(&room, &distro, &mut ledger);

    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        "Changed inside the distro.\n"
    );
    assert!(
        standings(&told).contains(&"outside"),
        "{:?}",
        standings(&told)
    );
    assert!(
        standings(&told)
            .iter()
            .all(|one| *one != "wrote" && *one != "both")
    );
}

#[test]
fn a_file_already_in_the_distro_is_never_adopted_nor_written_over() {
    let room = approved_here();
    let distro = distro();
    put(
        &distro.at("home/mario/.claude/CLAUDE.md"),
        "Mine, from before.\n",
    );
    let mut ledger = distro_ledger(&room);
    ledger.consent(true);

    round(&room, &distro, &mut ledger);

    assert_eq!(distro.read(".claude/CLAUDE.md"), b"Mine, from before.\n");
    assert!(
        ledger
            .written(&distro.at("home/mario/.claude/CLAUDE.md"))
            .is_none()
    );
}

fn put_away(room: &mut Room, said: &str) {
    let docs = room.session.paths.docs();
    let id = room
        .session
        .state
        .docs
        .values()
        .find(|one| tisty_core::docs::read(&docs, &one.file).is_ok_and(|body| body.contains(said)))
        .unwrap()
        .id;
    room.session
        .commit(tisty_core::event::Op::DocDelete { id })
        .unwrap();
}

fn written_in_a_distro() -> (Room, Distro, Ledger) {
    let room = approved_here();
    let distro = distro();
    let mut ledger = distro_ledger(&room);
    ledger.consent(true);
    round(&room, &distro, &mut ledger);
    assert_eq!(distro.files().len(), 2, "{:?}", distro.files());
    (room, distro, ledger)
}

#[test]
fn a_file_no_document_places_any_more_is_taken_away_with_a_backup_and_forgotten() {
    let (mut room, distro, mut ledger) = written_in_a_distro();
    let before = windows_ledger_bytes(&room);
    let at = distro.at("home/mario/.claude/rules/tone.md");
    put_away(&mut room, "Short answers.");

    let told = round(&room, &distro, &mut ledger);

    assert!(!at.exists(), "{:?}", standings(&told));
    assert!(told.loose.is_empty(), "{:?}", told.loose);
    assert!(ledger.written(&at).is_none());
    assert_eq!(agents::kept(room.session.paths.data(), &at).len(), 1);
    assert_eq!(distro.files(), ["home/mario/.claude/CLAUDE.md"]);
    assert_eq!(windows_ledger_bytes(&room), before);
}

#[test]
fn a_file_changed_inside_the_distro_stays_loose_and_counts_as_left() {
    let (mut room, distro, mut ledger) = written_in_a_distro();
    let before = windows_ledger_bytes(&room);
    let at = distro.at("home/mario/.claude/rules/tone.md");
    std::fs::write(
        &at,
        "Changed inside the distro.
",
    )
    .unwrap();
    put_away(&mut room, "Short answers.");

    let told = round(&room, &distro, &mut ledger);

    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        "Changed inside the distro.
"
    );
    assert_eq!(told.loose.len(), 1);
    assert!(told.loose[0].changed && told.loose[0].at.ends_with("tone.md"));
    assert!(ledger.written(&at).is_some());
    assert_eq!(counted(InDistro::default(), &told).left, 1);
    assert_eq!(windows_ledger_bytes(&room), before);
}

fn skills_written() -> (Room, Distro, Ledger, PathBuf) {
    let room = approved_here();
    let distro = distro();
    let mut ledger = distro_ledger(&room);
    ledger.take_tisty_skills(true);
    let mine = Ledger::open(room.session.paths.data(), HERE);
    round_in(&room.session, &distro.homes, &mine, &mut ledger, &shipped());
    let at = distro.at("home/mario/.claude/skills/tisty/SKILL.md");
    assert!(at.exists());
    (room, distro, ledger, at)
}

#[test]
fn with_the_skills_yes_gone_a_round_takes_away_an_unchanged_shipped_copy_with_a_backup() {
    let (room, distro, mut ledger, at) = skills_written();
    ledger.take_tisty_skills(false);

    round(&room, &distro, &mut ledger);

    assert_eq!(distro.files(), Vec::<String>::new());
    assert!(ledger.written(&at).is_none());
    assert_eq!(agents::kept(room.session.paths.data(), &at).len(), 1);
}

#[test]
fn with_the_skills_yes_gone_a_shipped_copy_edited_in_the_distro_stays() {
    let (room, distro, mut ledger, at) = skills_written();
    std::fs::write(
        &at,
        "Mine now.
",
    )
    .unwrap();
    ledger.take_tisty_skills(false);

    round(&room, &distro, &mut ledger);

    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        "Mine now.
"
    );
    assert!(ledger.written(&at).is_some());
}
