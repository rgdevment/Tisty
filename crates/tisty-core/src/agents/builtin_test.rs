use std::path::{Path, PathBuf};

use super::*;
use crate::agents::rendered;

const FETCHED: &str = "TISTY_SKILLS_FETCHED";

fn app() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/src-tauri")
}

fn the_lock() -> Vec<Locked> {
    locked(&std::fs::read_to_string(app().join("skills.lock")).expect("skills.lock reads"))
}

fn lock_of(files: &[(&str, &[u8])]) -> Vec<Locked> {
    files
        .iter()
        .map(|(name, bytes)| Locked {
            name: (*name).to_string(),
            print: crate::attach::printed(bytes),
        })
        .collect()
}

fn laid_out(files: &[(&str, &[u8])]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for (name, bytes) in files {
        let at = dir.path().join(name);
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        std::fs::write(at, bytes).unwrap();
    }
    dir
}

const TWO_SKILLS: [(&str, &[u8]); 3] = [
    ("ask/SKILL.md", b"---\nname: ask\n---\n\nAsk first.\n"),
    ("write/LICENSE", b"MIT License\n\nCopyright (c) someone\n"),
    ("write/SKILL.md", b"---\nname: write\n---\n\nWrite short.\n"),
];

#[test]
fn the_lock_names_the_four_files_of_the_three_skills() {
    let lock = the_lock();
    let names: Vec<&str> = lock.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "consolidate/SKILL.md",
            "tisty/SKILL.md",
            "writing-for-the-next-agent/LICENSE",
            "writing-for-the-next-agent/SKILL.md",
        ]
    );
    assert!(
        lock.iter()
            .all(|one| one.print.len() == 64 && one.print.chars().all(|c| c.is_ascii_hexdigit()))
    );
}

// CI fetches them with scripts/skills.sh and says so; a plain local run without them has nothing to compare.
#[test]
fn every_shipped_file_comes_back_byte_for_byte() {
    let dir = app().join("resources/skills");
    if !dir.is_dir() && std::env::var_os(FETCHED).is_none() {
        return;
    }
    let lock = the_lock();
    let found = bundled(&dir, &lock);
    assert_eq!(found.len(), lock.len(), "every file the lock names ships");
    for (one, locked) in found.iter().zip(&lock) {
        let disk = std::fs::read(dir.join(&one.name)).expect("the shipped file reads");
        assert_eq!(crate::attach::printed(&disk), locked.print, "{}", one.name);
        assert_eq!(
            rendered(&one.body).expect("it renders"),
            disk,
            "{}",
            one.name
        );
        assert!(builtin(&one.file()));
        assert_eq!(one.skill, skill_of(&one.name));
    }
}

#[test]
fn what_the_lock_names_is_read_and_nothing_else() {
    let mut files = TWO_SKILLS.to_vec();
    files.push(("ask/notes.md", b"Not shipped.\n"));
    let dir = laid_out(&files);
    let found = bundled(dir.path(), &lock_of(&TWO_SKILLS));
    let names: Vec<&str> = found.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["ask/SKILL.md", "write/LICENSE", "write/SKILL.md"]);
    for one in &found {
        let disk = std::fs::read(dir.path().join(&one.name)).unwrap();
        assert_eq!(rendered(&one.body).unwrap(), disk);
    }
}

#[test]
fn a_skill_with_one_file_unlike_the_lock_is_left_out_whole() {
    let dir = laid_out(&TWO_SKILLS);
    std::fs::write(dir.path().join("write/LICENSE"), b"Changed by hand.\n").unwrap();
    let found = bundled(dir.path(), &lock_of(&TWO_SKILLS));
    let names: Vec<&str> = found.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["ask/SKILL.md"]);
}

#[test]
fn a_skill_whose_licence_is_missing_never_goes_out_without_it() {
    let dir = laid_out(&TWO_SKILLS);
    std::fs::remove_file(dir.path().join("write/LICENSE")).unwrap();
    let found = bundled(dir.path(), &lock_of(&TWO_SKILLS));
    assert!(found.iter().all(|one| one.skill == "ask"));
    assert_eq!(found.len(), 1);
}

#[test]
fn a_lock_name_that_climbs_out_or_names_no_skill_is_never_read() {
    let outside = laid_out(&[("secret.md", b"Kept elsewhere.\n")]);
    let dir = outside.path().join("skills");
    std::fs::create_dir_all(dir.join("ask")).unwrap();
    std::fs::write(dir.join("ask/SKILL.md"), b"Ask.\n").unwrap();
    let climbing = [
        ("ask/../secret.md", b"Kept elsewhere.\n".as_slice()),
        ("../secret.md", b"Kept elsewhere.\n".as_slice()),
        ("SKILL.md", b"Ask.\n".as_slice()),
    ];
    assert!(bundled(&dir, &lock_of(&climbing)).is_empty());
}

#[test]
fn no_folder_brings_nothing() {
    assert!(bundled(Path::new("no/such/folder"), &lock_of(&TWO_SKILLS)).is_empty());
}

#[test]
fn a_lock_line_of_another_shape_names_nothing() {
    let lock = locked(
        "repo https://example.com/skills\nfile ABC tisty/SKILL.md\nfile only-one\nfile a b c\n# file x y\n",
    );
    assert_eq!(
        lock,
        [Locked {
            name: "tisty/SKILL.md".into(),
            print: "abc".into(),
        }]
    );
}

#[test]
fn no_key_of_tistys_own_skills_can_name_a_document() {
    for one in the_lock() {
        let file = format!("{BUILTIN}{}", one.name);
        assert!(!crate::docs::well_formed(&file), "{file}");
        assert!(builtin(&file));
    }
    assert!(!crate::docs::well_formed("tisty:skill/x-1"));
    assert!(!builtin("dev_here-0001"));
}

#[test]
fn only_the_release_config_ships_the_skills_and_it_keeps_every_other_resource() {
    let read = |name: &str| -> Vec<String> {
        let said: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(app().join(name)).unwrap()).unwrap();
        said["bundle"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one.as_str().unwrap().to_string())
            .collect()
    };
    let every_build = read("tauri.conf.json");
    let mut release = every_build.clone();
    release.push("resources/skills/**/*".into());
    assert_eq!(read("tauri.skills.conf.json"), release);
    assert!(every_build.iter().all(|one| !one.contains("skills")));
}
