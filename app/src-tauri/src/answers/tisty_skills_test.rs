use std::path::Path;

use super::*;
use crate::answers::agents_sync::{
    brought_home, consented_here, found_here, gated, loose_in, read_chosen, rounded,
};
use tisty_core::Paths;
use tisty_core::agents::{GLOBAL, Kind, doc_of};

const HERE: &str = "this-computer";
const PLUGIN: &str = r#"{"plugins":{"tisty@rgdevment":[{"scope":"user"}]}}"#;

struct Room {
    _tmp: tempfile::TempDir,
    home: PathBuf,
    homes: Homes,
    session: Session,
}

fn put(at: &Path, said: &str) {
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, said).unwrap();
}

fn shipped() -> Vec<Bundled> {
    [
        ("consolidate", "---\nname: consolidate\n---\n\nTidy up.\n"),
        ("tisty", "---\nname: tisty\n---\n\nWork with Tisty.\n"),
    ]
    .into_iter()
    .map(|(skill, said)| {
        let name = format!("{skill}/SKILL.md");
        Bundled {
            skill: skill.into(),
            body: doc_of(GLOBAL, Kind::Skill, &name, said.as_bytes()).unwrap(),
            name,
        }
    })
    .collect()
}

fn room() -> Room {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    put(&home.join(".claude/CLAUDE.md"), "# Claude\n");
    put(&home.join(".codex/AGENTS.md"), "Codex rules.\n");
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    Room {
        homes: Homes::with(&home, &[]),
        session: Session::at(paths).unwrap(),
        home,
        _tmp: tmp,
    }
}

impl Room {
    fn yes(&mut self) {
        let (ledger, _, _) = consented_here(&self.session, &self.homes, HERE, Some(true)).unwrap();
        brought_home(&mut self.session, &self.homes, ledger, &[], Vec::new()).unwrap();
        assert!(agents::root_of(&self.session.state).is_some());
    }

    fn ledger(&self) -> Ledger {
        Ledger::open(self.session.paths.data(), HERE)
    }

    fn written_with_the_switch_on(&self) -> Ledger {
        let data = self.session.paths.data();
        let mut ledger = self.ledger();
        ledger.take_tisty_skills(true);
        ledger.keep(data).unwrap();
        let gate = gated_with(&self.session, &ledger, shipped);
        let placed = places::targets(&self.homes, &gate, &ledger);
        rounded(data, &mut ledger, &placed);
        ledger
    }

    fn at(&self, at: &str) -> PathBuf {
        at.split('/')
            .fold(self.home.clone(), |at, step| at.join(step))
    }

    fn summed(&self) -> TistySkills {
        summed_from(&self.session, &self.homes, &self.ledger(), &shipped())
    }
}

fn landed(said: &TistySkills) -> Vec<(String, &'static str)> {
    let mut all: Vec<(String, &'static str)> = said
        .landing
        .iter()
        .map(|one| (one.assistant.clone(), one.standing))
        .collect();
    all.sort();
    all
}

impl Room {
    fn skills_yes(&self) -> TistySkills {
        taken_from(&self.session, &self.homes, HERE, true, &shipped()).unwrap()
    }

    fn files_of(&self, folder: &str) -> Vec<String> {
        let at = self.at(folder);
        let mut all: Vec<String> = std::fs::read_dir(&at)
            .map(|all| {
                all.flatten()
                    .map(|one| one.file_name().to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        all.sort();
        all
    }
}

#[test]
fn before_either_yes_the_card_names_where_they_go_and_reads_nothing_of_the_agents() {
    let room = room();
    put(&room.at(".claude/plugins/installed_plugins.json"), PLUGIN);
    // Had it looked, the plugin would keep Claude out and this skill would read as not Tisty's.
    put(
        &room.at(".claude/skills/tisty/SKILL.md"),
        "Someone else's.\n",
    );
    let said = room.summed();
    assert!(!said.on);
    assert!(said.plugged.is_empty(), "the plugin's records are not read");
    assert_eq!(
        landed(&said),
        [
            ("claude".to_string(), "write"),
            ("claude".to_string(), "write"),
            ("codex".to_string(), "write"),
            ("codex".to_string(), "write"),
        ],
        "only the folders found, never what they hold"
    );
    let names: Vec<&str> = said.skills.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["consolidate", "tisty"]);
    assert!(said.leftover.is_empty());

    let mut ledger = room.ledger();
    ledger.take_tisty_skills(true);
    let looked = summed_from(&room.session, &room.homes, &ledger, &shipped());
    assert_eq!(
        looked.plugged,
        ["claude"],
        "with the yes it reads the plugin"
    );
    assert!(looked.landing.iter().all(|one| one.assistant != "claude"));
}

#[test]
fn their_own_yes_writes_them_with_agents_off_and_reads_the_plugin_then() {
    let room = room();
    let said = room.skills_yes();
    assert!(said.on);
    assert!(
        agents::root_of(&room.session.state).is_none(),
        "Agents stays off"
    );
    assert!(!room.ledger().consented());
    for folder in [".claude/skills/tisty", ".agents/skills/consolidate"] {
        assert_eq!(room.files_of(folder), ["SKILL.md"], "{folder}");
    }
    assert!(
        landed(&said)
            .iter()
            .all(|(_, standing)| *standing == "upToDate")
    );

    put(&room.at(".claude/plugins/installed_plugins.json"), PLUGIN);
    let said = room.summed();
    assert_eq!(said.plugged, ["claude"]);
    assert!(said.landing.iter().all(|one| one.assistant != "claude"));
    let mut over: Vec<&str> = said.leftover.iter().map(|one| one.at.as_str()).collect();
    over.sort();
    let claude = |skill: &str| {
        room.at(&format!(".claude/skills/{skill}/SKILL.md"))
            .to_string_lossy()
            .into_owned()
    };
    assert_eq!(over, [claude("consolidate"), claude("tisty")]);
    assert!(said.leftover.iter().all(|one| !one.changed));
    assert!(
        room.at(".claude/skills/tisty/SKILL.md").is_file(),
        "never taken away alone"
    );
}

#[test]
fn a_copy_left_over_goes_only_by_its_own_button_with_a_backup_and_only_one_the_ledger_names() {
    let room = room();
    room.skills_yes();
    put(&room.at(".claude/plugins/installed_plugins.json"), PLUGIN);
    let tisty = room.at(".claude/skills/tisty/SKILL.md");
    let at = tisty.to_string_lossy().into_owned();

    let stranger = room.at(".claude/CLAUDE.md").to_string_lossy().into_owned();
    assert!(left_out(&room.session, &room.homes, HERE, &stranger, &shipped()).is_err());
    assert!(room.at(".claude/CLAUDE.md").is_file());

    let said = left_out(&room.session, &room.homes, HERE, &at, &shipped()).unwrap();
    assert!(!tisty.exists());
    assert_eq!(said.leftover.len(), 1);
    assert_eq!(
        agents::read_kept(room.session.paths.data(), &tisty, 1).unwrap(),
        std::fs::read(room.at(".agents/skills/tisty/SKILL.md")).unwrap()
    );
    assert!(left_out(&room.session, &room.homes, HERE, &at, &shipped()).is_err());
}

#[test]
fn turning_them_off_takes_the_intact_copies_away_and_keeps_the_changed_one_in_the_card() {
    let mut room = room();
    room.yes();
    let ledger = room.written_with_the_switch_on();
    let mine = room.at(".claude/skills/tisty/SKILL.md");
    let intact = room.at(".agents/skills/tisty/SKILL.md");
    assert!(intact.is_file());
    assert_eq!(
        landed(&summed_from(
            &room.session,
            &room.homes,
            &ledger,
            &shipped()
        ))[0]
            .1,
        "upToDate"
    );
    put(&mine, "My own lines now.\n");

    let said = taken_from(&room.session, &room.homes, HERE, false, &shipped()).unwrap();
    assert!(!said.on);
    assert!(!intact.exists());
    assert!(!room.at(".claude/skills/consolidate/SKILL.md").exists());
    assert_eq!(
        std::fs::read_to_string(&mine).unwrap(),
        "My own lines now.\n"
    );
    let over: Vec<(String, bool)> = said
        .leftover
        .iter()
        .map(|one| (one.at.clone(), one.changed))
        .collect();
    assert_eq!(over, [(mine.to_string_lossy().into_owned(), true)]);

    let ledger = room.ledger();
    let gate = gated(&room.session, &ledger);
    assert!(
        loose_in(&room.homes, &gate, &ledger, &[], true).is_empty(),
        "Agents never asks about Tisty's own copies"
    );
}

#[test]
fn agents_turned_on_after_them_brings_none_of_tistys_copies_in_and_withdrawing_it_keeps_them() {
    let mut room = room();
    room.skills_yes();
    let (ledger, known, writes) =
        consented_here(&room.session, &room.homes, HERE, Some(true)).unwrap();
    let found = found_here(&room.homes);
    let chosen: std::collections::BTreeSet<String> = found
        .iter()
        .map(|one| one.at.to_string_lossy().into_owned())
        .collect();
    let (read, refused) = read_chosen(&ledger, &found, &chosen, &known, &writes, &shipped());
    let tistys = |at: &str| at.contains("SKILL.md");
    assert!(
        read.iter()
            .all(|(one, _)| !tistys(&one.at.to_string_lossy()))
    );
    assert!(
        refused
            .iter()
            .filter(|one| tistys(&one.at))
            .all(|one| one.reason == "skip_tisty")
    );
    brought_home(&mut room.session, &room.homes, ledger, &read, refused).unwrap();
    assert!(
        room.session.state.docs.values().all(|kept| {
            let body =
                tisty_core::docs::read(&room.session.paths.docs(), &kept.file).unwrap_or_default();
            !body.contains("Work with Tisty.") && !body.contains("Tidy up.")
        }),
        "no document holds Tisty's words"
    );
    assert!(room.summed().on);

    // «Stop syncing and hide» pauses every agent; it is the way Agents goes quiet, and it never reaches the skills.
    let mut ledger = room.ledger();
    ledger.turn_all(false);
    ledger.keep(room.session.paths.data()).unwrap();
    let newer: Vec<Bundled> = shipped()
        .into_iter()
        .map(|one| Bundled {
            body: one
                .body
                .replace("Work with Tisty.", "Work with Tisty, 1.51."),
            ..one
        })
        .collect();
    let mut ledger = room.ledger();
    shipped_round(&room.session, &room.homes, &mut ledger, &newer);
    assert!(summed_from(&room.session, &room.homes, &ledger, &newer).on);
    assert!(
        std::fs::read_to_string(room.at(".claude/skills/tisty/SKILL.md"))
            .unwrap()
            .contains("1.51"),
        "Agents paused, the skills still update"
    );
    let said = taken_from(&room.session, &room.homes, HERE, false, &newer).unwrap();
    assert!(!room.at(".claude/skills/tisty/SKILL.md").exists());
    assert!(said.leftover.is_empty());
}

#[test]
fn a_ledger_copied_from_another_computer_turns_none_of_them_on() {
    let room = room();
    room.skills_yes();
    let other = Ledger::open(room.session.paths.data(), "another-computer");
    assert!(!other.tisty_skills());
    let said = summed_from(&room.session, &room.homes, &other, &shipped());
    assert!(!said.on && said.plugged.is_empty());
}

#[test]
fn a_computer_that_cannot_name_itself_cannot_turn_them_on() {
    let room = room();
    assert!(taken_from(&room.session, &room.homes, "", true, &shipped()).is_err());
    let mut ledger = Ledger::open(room.session.paths.data(), HERE);
    assert!(
        shipped_round(&room.session, &room.homes, &mut ledger, &shipped())
            .told
            .is_empty()
    );
}

#[test]
fn with_neither_yes_a_copy_left_over_is_still_listed_and_taken_as_changed() {
    let mut room = room();
    room.yes();
    let ledger = room.written_with_the_switch_on();
    let mine = room.at(".claude/skills/tisty/SKILL.md");
    put(&mine, "My own lines now.\n");
    taken_from(&room.session, &room.homes, HERE, false, &shipped()).unwrap();
    let mut ledger_now = room.ledger();
    ledger_now.consent(false);
    ledger_now.keep(room.session.paths.data()).unwrap();
    drop(ledger);
    let said = room.summed();
    let over: Vec<(String, bool)> = said
        .leftover
        .iter()
        .map(|one| (one.at.clone(), one.changed))
        .collect();
    assert_eq!(over, [(mine.to_string_lossy().into_owned(), true)]);
    std::fs::remove_file(&mine).unwrap();
    assert!(room.summed().leftover.is_empty());
}

#[test]
fn turning_them_off_takes_them_from_a_paused_agent_too_and_hides_nothing() {
    let mut room = room();
    room.yes();
    room.written_with_the_switch_on();
    let mut ledger = room.ledger();
    ledger.turn("codex", false);
    ledger.keep(room.session.paths.data()).unwrap();
    let said = taken_from(&room.session, &room.homes, HERE, false, &shipped()).unwrap();
    assert!(!room.at(".agents/skills/tisty/SKILL.md").exists());
    assert!(said.leftover.is_empty());
}

#[test]
fn the_first_look_says_tistys_copies_are_its_own_never_already_in_agents() {
    let room = room();
    room.skills_yes();
    let look = crate::answers::agents_sync::looked(
        &crate::answers::agents_sync::gathered(&room.session, HERE),
        &room.homes,
        &found_here(&room.homes),
    );
    let tistys: Vec<_> = look
        .assistants
        .iter()
        .flat_map(|one| &one.files)
        .filter(|one| one.at.ends_with("SKILL.md"))
        .collect();
    assert!(!tistys.is_empty());
    assert!(
        tistys
            .iter()
            .all(|one| one.skip == Some("tisty".to_string()) && !one.held),
        "{tistys:?}"
    );
}

#[test]
fn a_copy_whose_agent_left_this_computer_is_still_taken_away_and_listed() {
    let room = room();
    room.skills_yes();
    let shared = room.at(".agents/skills/tisty/SKILL.md");
    assert!(shared.is_file());
    std::fs::remove_dir_all(room.at(".codex")).unwrap();
    let said = room.summed();
    assert!(
        said.leftover
            .iter()
            .any(|one| one.at == shared.to_string_lossy()),
        "no agent reads it now, so it is left over"
    );
    let said = taken_from(&room.session, &room.homes, HERE, false, &shipped()).unwrap();
    assert!(!shared.exists(), "turning off takes it away all the same");
    assert!(said.leftover.is_empty());
}

#[test]
fn turning_off_keeps_a_whole_skill_when_one_of_its_files_changed() {
    let room = room();
    let two = |skill: &str, said: &str| {
        let name = skill.to_string();
        Bundled {
            skill: agents::skill_of(&name).to_string(),
            body: doc_of(GLOBAL, Kind::Skill, &name, said.as_bytes()).unwrap(),
            name,
        }
    };
    let next = [
        two("next/LICENSE", "MIT.\n"),
        two("next/SKILL.md", "---\nname: next\n---\n\nNext.\n"),
    ];
    taken_from(&room.session, &room.homes, HERE, true, &next).unwrap();
    let licence = room.at(".claude/skills/next/LICENSE");
    let skill = room.at(".claude/skills/next/SKILL.md");
    assert!(licence.is_file() && skill.is_file());
    put(&skill, "Mine now.\n");
    let said = taken_from(&room.session, &room.homes, HERE, false, &next).unwrap();
    assert!(
        licence.is_file(),
        "the licence stays with its changed skill"
    );
    assert_eq!(std::fs::read_to_string(&skill).unwrap(), "Mine now.\n");
    assert!(
        said.leftover
            .iter()
            .any(|one| one.at == licence.to_string_lossy())
    );
    assert!(
        !room.at(".agents/skills/next/LICENSE").exists(),
        "an untouched skill goes whole"
    );
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn a_skill_of_the_persons_named_with_other_capitals_is_never_offered_as_left_over() {
    let mut room = room();
    room.yes();
    room.written_with_the_switch_on();
    let (ledger, _, _) = consented_here(&room.session, &room.homes, HERE, Some(true)).unwrap();
    let at = room.at(".claude/skills/Tisty/SKILL.md");
    let found = agents::Found {
        assistant: "claude".into(),
        kind: Kind::Skill,
        at: at.clone(),
        name: "Tisty/SKILL.md".into(),
        skip: None,
    };
    let body = doc_of("claude", Kind::Skill, "Tisty/SKILL.md", b"Mine.\n").unwrap();
    brought_home(
        &mut room.session,
        &room.homes,
        ledger,
        &[(found, body)],
        Vec::new(),
    )
    .unwrap();
    let said = room.summed();
    assert!(
        said.skills
            .iter()
            .any(|one| one.name == "tisty" && one.yours),
        "{said:?}"
    );
    let tistys = room.at(".claude/skills/tisty/SKILL.md");
    assert!(
        said.leftover
            .iter()
            .all(|one| !one.at.eq_ignore_ascii_case(&tistys.to_string_lossy())),
        "the person's document places that file: {:?}",
        said.leftover
    );
    let refused = left_out(
        &room.session,
        &room.homes,
        HERE,
        &tistys.to_string_lossy(),
        &shipped(),
    );
    assert!(refused.is_err());
    assert!(at.is_file());
}

#[test]
fn with_the_ledger_gone_a_file_with_tistys_very_bytes_is_still_never_brought_in() {
    let mut room = room();
    let said = shipped()
        .into_iter()
        .find(|one| one.skill == "tisty")
        .unwrap();
    let raw = agents::rendered(&said.body).unwrap();
    let at = room.at(".claude/skills/tisty/SKILL.md");
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, &raw).unwrap();
    let theirs = room.at(".claude/skills/mine/SKILL.md");
    put(&theirs, "My own.\n");

    let look = crate::answers::agents_sync::looked_with(
        &crate::answers::agents_sync::gathered(&room.session, HERE),
        &room.homes,
        &found_here(&room.homes),
        &shipped(),
    );
    let files: Vec<_> = look.assistants.iter().flat_map(|one| &one.files).collect();
    let seen = |path: &std::path::Path| {
        files
            .iter()
            .find(|one| one.at == path.to_string_lossy())
            .unwrap()
    };
    assert_eq!(seen(&at).skip, Some("tisty".to_string()));
    assert_eq!(seen(&theirs).skip, None);

    let (ledger, known, writes) =
        consented_here(&room.session, &room.homes, HERE, Some(true)).unwrap();
    let found = found_here(&room.homes);
    let chosen: std::collections::BTreeSet<String> = [&at, &theirs]
        .iter()
        .map(|one| one.to_string_lossy().into_owned())
        .collect();
    let (read, refused) = read_chosen(&ledger, &found, &chosen, &known, &writes, &shipped());
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].0.at, theirs);
    assert_eq!(refused.len(), 1);
    assert_eq!(refused[0].reason, "skip_tisty");
    brought_home(&mut room.session, &room.homes, ledger, &read, refused).unwrap();
    assert!(!room.summed().skills.iter().any(|one| one.yours));
}
