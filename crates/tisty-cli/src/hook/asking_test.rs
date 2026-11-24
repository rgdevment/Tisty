use std::path::PathBuf;

use serde_json::{Value, json};

use super::*;
use crate::mcp::{CARD_LEFT_OUTSIDE, COMMAND_LEAVES_OUTSIDE, FILE_LEFT_OUTSIDE};

fn nowhere() -> PathBuf {
    match cfg!(windows) {
        true => PathBuf::from(r"X:\Nowhere"),
        false => PathBuf::from("/nowhere"),
    }
}

fn here() -> Here {
    Here {
        home: Some(nowhere().join("Person")),
        temp: nowhere().join("scratch"),
    }
}

fn folder(name: &str) -> PathBuf {
    nowhere().join("Person").join(name)
}

fn writing(path: &Path) -> Call {
    let heard = json!({ "tool_input": { "file_path": path.display().to_string() } });
    Call::read(heard.to_string().as_bytes())
}

fn running(command: &str, cwd: &str) -> Call {
    let heard = json!({ "tool_input": { "command": command }, "cwd": cwd });
    Call::read(heard.to_string().as_bytes())
}

fn card(initiation: &str) -> Call {
    let heard = json!({ "tool_input": { "initiation": initiation } });
    Call::read(heard.to_string().as_bytes())
}

struct Cache {
    tmp: tempfile::TempDir,
}

fn cache() -> Cache {
    Cache {
        tmp: tempfile::tempdir().unwrap(),
    }
}

impl Cache {
    fn session(&self, name: &str) -> Session {
        Session::of(self.tmp.path(), name)
    }
}

fn ask_file(session: &Session, name: &str, at: i128) -> Option<&'static str> {
    asked(
        session,
        &here(),
        &writing(&folder(name).join("a.md")),
        Some("Write"),
        at,
    )
}

#[test]
fn the_first_file_in_a_folder_asks_and_leaves_its_mark_and_the_same_call_again_does_not() {
    let cache = cache();
    let session = cache.session("s1");
    let call = writing(&folder("Documents").join("a.md"));

    assert_eq!(
        asked(&session, &here(), &call, Some("Write"), 7),
        Some(FILE_LEFT_OUTSIDE)
    );
    assert_eq!(session.marks().asked, Some(7));
    assert_eq!(asked(&session, &here(), &call, Some("Write"), 8), None);
    assert_eq!(session.marks().asked, Some(7));
}

#[test]
fn a_folder_below_one_already_asked_about_does_not_ask_and_another_folder_does() {
    let cache = cache();
    let session = cache.session("s1");

    assert!(ask_file(&session, "Documents", 1).is_some());
    assert_eq!(ask_file(&session, "Documents/sub", 2), None);
    assert_eq!(ask_file(&session, "Documents/sub/deeper", 3), None);
    assert_eq!(ask_file(&session, "Other", 4), Some(FILE_LEFT_OUTSIDE));
    assert_eq!(session.marks().asked, Some(4));
}

#[test]
fn a_folder_above_one_already_asked_about_asks_again() {
    let cache = cache();
    let session = cache.session("s1");

    assert!(ask_file(&session, "Documents/sub", 1).is_some());
    assert!(ask_file(&session, "Documents", 2).is_some());
}

#[test]
fn a_session_asks_about_three_folders_at_most() {
    let cache = cache();
    let session = cache.session("s1");

    let all: Vec<bool> = ["One", "Two", "Three", "Four", "Five"]
        .iter()
        .zip(1..)
        .map(|(name, at)| ask_file(&session, name, at).is_some())
        .collect();

    assert_eq!(all, [true, true, true, false, false]);
    assert_eq!(session.marks().asked, Some(3));
}

#[test]
fn another_session_asks_about_the_same_folder_again() {
    let cache = cache();

    assert!(ask_file(&cache.session("s1"), "Documents", 1).is_some());
    assert_eq!(ask_file(&cache.session("s1"), "Documents", 2), None);
    assert!(ask_file(&cache.session("s2"), "Documents", 3).is_some());
}

#[test]
fn a_file_that_would_not_ask_leaves_no_mark() {
    let cache = cache();
    let session = cache.session("s1");
    let call = writing(&folder("Documents").join("a.rs"));

    assert_eq!(asked(&session, &here(), &call, Some("Write"), 1), None);
    assert_eq!(session.marks().asked, None);
    assert!(!cache.tmp.path().join("turns").exists());
}

#[test]
fn a_card_the_agent_started_on_its_own_asks_once_a_session() {
    let cache = cache();
    let session = cache.session("s1");
    let tool = Some("mcp__ccd_session__start_session");

    let first = asked(&session, &here(), &card("own_initiative"), tool, 1);
    let again = asked(&session, &here(), &card("own_initiative"), tool, 2);
    let other = asked(
        &cache.session("s2"),
        &here(),
        &card("own_initiative"),
        tool,
        3,
    );

    assert_eq!(first, Some(CARD_LEFT_OUTSIDE));
    assert_eq!(session.marks().asked, Some(1));
    assert_eq!(again, None);
    assert_eq!(session.marks().asked, Some(1));
    assert_eq!(other, Some(CARD_LEFT_OUTSIDE));
}

#[test]
fn a_card_the_person_asked_for_never_asks() {
    let cache = cache();
    let session = cache.session("s1");
    let tool = Some("mcp__ccd_session__start_session");

    for initiation in ["user_asked", "offer"] {
        assert_eq!(asked(&session, &here(), &card(initiation), tool, 1), None);
    }
}

#[test]
fn a_session_whose_marks_cannot_be_kept_never_asks() {
    let tmp = tempfile::tempdir().unwrap();
    let blocked = tmp.path().join("a-file");
    std::fs::write(&blocked, "x").unwrap();
    let session = Session::of(&blocked, "s1");
    let tool = Some("mcp__ccd_session__start_session");

    assert_eq!(ask_file(&session, "Documents", 1), None);
    assert_eq!(
        asked(&session, &here(), &card("own_initiative"), tool, 2),
        None
    );
    assert_eq!(session.marks().asked, None);
}

#[test]
fn a_command_that_writes_a_text_file_outside_asks_with_its_own_words() {
    let cache = cache();
    let session = cache.session("s1");
    let target = folder("Documents").join("a.md").display().to_string();
    let call = running(&format!("echo x > {target}"), "");

    let said = asked(&session, &here(), &call, Some("Bash"), 1);

    assert_eq!(said, Some(COMMAND_LEAVES_OUTSIDE));
    assert_eq!(asked(&session, &here(), &call, Some("Bash"), 2), None);
}

#[test]
fn a_command_with_no_tool_named_asks_too() {
    let cache = cache();
    let session = cache.session("s1");
    let target = folder("Documents").join("a.md").display().to_string();
    let heard = json!({ "command": format!("echo x > {target}"), "cwd": "" });
    let call = Call::read(heard.to_string().as_bytes());

    assert_eq!(
        asked(&session, &here(), &call, None, 1),
        Some(COMMAND_LEAVES_OUTSIDE)
    );
}

#[test]
fn a_command_that_writes_nothing_of_its_own_or_not_prose_never_asks() {
    let cache = cache();
    let session = cache.session("s1");
    let target = folder("Documents").display().to_string();

    for command in [
        "git status".to_string(),
        format!("python x.py > {target}/a.md"),
        format!("echo x > {target}/a.rs"),
        format!("echo x 2> {target}/a.md"),
        String::new(),
    ] {
        let call = running(&command, "");
        assert_eq!(
            asked(&session, &here(), &call, Some("Bash"), 1),
            None,
            "{command}"
        );
    }
    assert_eq!(session.marks().asked, None);
}

#[test]
fn a_relative_path_in_a_command_is_placed_from_where_it_runs() {
    let cache = cache();
    let cwd = folder("Documents").display().to_string();

    let session = cache.session("s1");
    let plain = running("echo x > a.md", &cwd);
    assert_eq!(
        asked(&session, &here(), &plain, Some("Bash"), 1),
        Some(COMMAND_LEAVES_OUTSIDE)
    );

    let session = cache.session("s2");
    assert_eq!(
        asked(
            &session,
            &here(),
            &running("echo x > a.md", ""),
            Some("Bash"),
            1
        ),
        None
    );
}

#[test]
fn a_relative_path_after_a_move_cannot_be_placed_but_an_absolute_one_can() {
    let cache = cache();
    let cwd = folder("Documents").display().to_string();
    let absolute = folder("Other").join("a.md").display().to_string();

    let relative = running("cd /somewhere && echo y > a.md", &cwd);
    assert_eq!(
        asked(&cache.session("s1"), &here(), &relative, Some("Bash"), 1),
        None
    );

    let named = running(&format!("cd /somewhere && echo y > {absolute}"), &cwd);
    assert_eq!(
        asked(&cache.session("s1"), &here(), &named, Some("Bash"), 2),
        Some(COMMAND_LEAVES_OUTSIDE)
    );

    let home = running("cd /somewhere && echo y > ~/Notes/a.md", &cwd);
    assert_eq!(
        asked(&cache.session("s2"), &here(), &home, Some("Bash"), 3),
        Some(COMMAND_LEAVES_OUTSIDE)
    );
}

fn everything_under(at: &Path, found: &mut Vec<(String, String)>) {
    for one in std::fs::read_dir(at).unwrap().flatten() {
        let name = one.file_name().to_string_lossy().to_lowercase();
        match one.path().is_dir() {
            true => {
                found.push((name, String::new()));
                everything_under(&one.path(), found);
            }
            false => {
                let content = std::fs::read_to_string(one.path()).unwrap_or_default();
                found.push((name, content.to_lowercase()));
            }
        }
    }
}

#[test]
fn neither_a_file_name_nor_a_folder_is_kept_anywhere_in_the_session() {
    let cache = cache();
    let session = cache.session("s1");
    let secret: Value = json!("Confidential-Roadmap");
    let at = folder("Confidential-Folder").join(format!("{}.md", secret.as_str().unwrap()));
    let call = writing(&at);
    let typed = running(
        &format!(
            "echo x > {}",
            folder("Hidden-Place").join("Private-Plan.md").display()
        ),
        "",
    );

    assert!(asked(&session, &here(), &call, Some("Write"), 1).is_some());
    assert!(asked(&session, &here(), &typed, Some("Bash"), 2).is_some());

    let mut found = Vec::new();
    everything_under(cache.tmp.path(), &mut found);
    assert!(!found.is_empty());
    for (name, content) in found {
        for word in [
            "confidential",
            "roadmap",
            "hidden",
            "private",
            "nowhere",
            "person",
            ".md",
        ] {
            assert!(!name.contains(word), "{name} holds {word}");
            assert!(!content.contains(word), "{name} holds {word}");
        }
    }
}
