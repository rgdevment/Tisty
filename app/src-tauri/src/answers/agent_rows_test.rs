use super::*;
use tisty_core::Paths;
use tisty_core::agents::{Found, Kind, doc_of};

use crate::answers::agents_homes::tests::{
    body_for, laptop, laptops_folder, laptops_in, mine_in, my_folder,
};
use crate::answers::agents_sync::filed;
use crate::answers::agents_sync::tests::{doc_here, room, root_folder, sub_of};

fn session() -> (tempfile::TempDir, Session) {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.docs()).unwrap();
    let session = Session::at(paths).unwrap();
    (tmp, session)
}

fn found(assistant: &str, kind: Kind, name: &str, words: &str) -> (Found, String) {
    let one = Found {
        assistant: assistant.to_string(),
        kind,
        at: PathBuf::from(name),
        name: name.to_string(),
        skip: None,
    };
    let body = doc_of(assistant, kind, name, words.as_bytes()).unwrap();
    (one, body)
}

fn labels(session: &Session) -> Vec<(String, Option<String>)> {
    let (docs, files) = listed(session);
    let mut all: Vec<(String, Option<String>)> = rows(&docs, &files, &shared(session))
        .into_iter()
        .map(|one| (one.shown.label, one.shown.skill))
        .collect();
    all.sort();
    all
}

#[test]
fn without_agents_nothing_is_read() {
    let (_tmp, session) = session();
    let (_, files) = listed(&session);
    assert!(files.is_empty());
}

#[test]
fn each_agent_document_comes_with_its_name_kind_and_skill() {
    let (_tmp, mut session) = session();
    let bodies = [
        found(
            "claude",
            Kind::Rule,
            "rules/horario.md",
            "# Horario laboral\n",
        ),
        found(
            "claude",
            Kind::Skill,
            "pr/SKILL.md",
            "---\nname: pr\n---\n# PR\n",
        ),
        found("claude", Kind::Skill, "pr/references/body.md", "# Body\n"),
        found("claude", Kind::Memory, "MEMORY.md", "Index\n"),
    ];
    filed(&mut session, &bodies).unwrap();
    assert_eq!(
        labels(&session),
        [
            ("Body".to_string(), Some("pr".to_string())),
            ("Horario laboral".to_string(), None),
            ("MEMORY.md".to_string(), None),
            ("pr".to_string(), Some("pr".to_string())),
        ]
    );
}

#[test]
fn a_row_says_its_file_and_reads_as_the_window_expects() {
    let (_tmp, mut session) = session();
    filed(
        &mut session,
        &[found("codex", Kind::Rule, "AGENTS.md", "# Codex\n")],
    )
    .unwrap();
    let (docs, files) = listed(&session);
    let row = rows(&docs, &files, &shared(&session)).pop().unwrap();
    let said = serde_json::to_value(&row).unwrap();
    assert_eq!(said["file"], files[0].as_str());
    assert_eq!(said["kind"], "rule");
    assert_eq!(said["level"], "codex");
    assert_eq!(said["name"], "AGENTS.md");
    assert_eq!(said["label"], "Codex");
    assert!(said["skill"].is_null());
}

#[test]
fn a_document_that_lost_its_card_or_its_file_is_left_out() {
    let (_tmp, mut session) = session();
    filed(
        &mut session,
        &[
            found("claude", Kind::Rule, "a.md", "# A\n"),
            found("claude", Kind::Rule, "b.md", "# B\n"),
        ],
    )
    .unwrap();
    let (docs, files) = listed(&session);
    let at = tisty_core::docs::resolve(&docs, &files[0]).unwrap();
    std::fs::write(at, "# No card now\n").unwrap();
    let mut asked = files.clone();
    asked.push("missing.md".to_string());
    assert_eq!(rows(&docs, &asked, &shared(&session)).len(), 1);
}

#[test]
fn only_the_files_asked_for_are_read_and_only_under_agents() {
    let all = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(asked_for(all.clone(), None), all);
    assert_eq!(
        asked_for(all, Some(vec!["c".to_string(), "outside".to_string()])),
        ["c"]
    );
}

fn everyone_of(session: &Session, file: &str) -> bool {
    let (docs, files) = listed(session);
    rows(&docs, &files, &shared(session))
        .into_iter()
        .find(|one| one.file == file)
        .unwrap_or_else(|| panic!("no row for {file}"))
        .everyone
}

#[test]
fn a_document_in_global_or_in_a_root_agent_folder_is_for_everyone() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let body = |name: &str| {
        body_for(
            "claude", name, "Short.
", None,
        )
    };
    let global = sub_of(&room, "Global");
    let in_global = mine_in(&mut room, global, &body("rules/a.md"));
    let claude = root_folder(&mut room, "Claude");
    let in_claude = mine_in(&mut room, claude, &body("rules/b.md"));
    let at_root = doc_here(
        &mut room,
        "claude",
        tisty_core::agents::Kind::Rule,
        "rules/c.md",
        "Short.
",
    );
    for file in [in_global, in_claude, at_root] {
        assert!(everyone_of(&room.session, &file), "{file}");
    }
}

#[test]
fn a_document_in_this_computers_folder_or_below_it_is_not_for_everyone() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let body = body_for(
        "claude",
        "rules/a.md",
        "Short.
",
        None,
    );
    let own = my_folder(&room);
    let in_own = mine_in(&mut room, own, &body);
    let below = sub_of(&room, "Claude");
    let in_below = mine_in(&mut room, below, &body);
    for file in [in_own, in_below] {
        assert!(!everyone_of(&room.session, &file), "{file}");
    }
}

#[test]
fn a_document_in_another_computers_folder_is_not_for_everyone() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let other = laptop();
    let folder = laptops_folder(&mut room, &other);
    let body = body_for(
        "claude",
        "rules/a.md",
        "Short.
",
        Some(&other),
    );
    let file = laptops_in(&mut room, &other, folder, &body);
    assert!(!everyone_of(&room.session, &file));
}

#[test]
fn a_row_says_everyone_as_the_window_expects() {
    let mut room = room();
    room.bring(&[".claude/CLAUDE.md"]);
    let global = sub_of(&room, "Global");
    mine_in(
        &mut room,
        global,
        &body_for(
            "claude",
            "rules/a.md",
            "Short.
",
            None,
        ),
    );
    let (docs, files) = listed(&room.session);
    let all = rows(&docs, &files, &shared(&room.session));
    let said: Vec<serde_json::Value> = all
        .iter()
        .map(|one| serde_json::to_value(one).unwrap())
        .collect();
    assert!(said.iter().any(|one| one["everyone"] == true));
    assert!(said.iter().any(|one| one["everyone"] == false));
}

#[test]
fn without_agents_nothing_is_shared() {
    let (_tmp, session) = session();
    assert!(shared(&session).is_empty());
}
