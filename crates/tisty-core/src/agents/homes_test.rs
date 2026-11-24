use super::*;

fn told(pairs: &[(&'static str, &Path)]) -> Told {
    pairs
        .iter()
        .map(|(named, at)| (*named, at.to_path_buf()))
        .collect()
}

#[test]
fn each_assistant_reads_from_its_usual_folder_or_the_one_its_variable_names() {
    let home = Path::new("/home/rina");
    let moved = Path::new("/home/rina/Documents/claude");
    let gemini = Path::new("/srv/gemini-home");
    let dsh = Path::new("/srv/dsh-home");
    let told = told(&[
        ("CLAUDE_CONFIG_DIR", moved),
        ("GEMINI_CLI_HOME", gemini),
        ("DSH_HOME", dsh),
    ]);

    assert_eq!(
        assistant_root(home, &told, "claude"),
        Some(moved.to_path_buf())
    );
    assert_eq!(
        assistant_root(home, &told, "codex"),
        Some(home.join(".codex"))
    );
    assert_eq!(
        assistant_root(home, &told, "gemini"),
        Some(gemini.join(".gemini"))
    );
    assert_eq!(
        assistant_root(home, &told, "cursor"),
        Some(home.join(".cursor"))
    );
    assert_eq!(assistant_root(home, &told, "dsh"), Some(dsh.to_path_buf()));
    assert_eq!(assistant_root(home, &told, "notepad"), None);
    assert!(assistant_folders(home, &told).contains(&home.join(".agents")));
    assert_eq!(assistant_folders(home, &told).len(), 7);
}

#[test]
fn a_place_inside_any_assistants_folder_is_told_apart_from_one_beside_it() {
    let tmp = tempfile::tempdir().unwrap();
    let claude = tmp.path().join("claude-config");
    std::fs::create_dir_all(claude.join("rules")).unwrap();
    let folders = [claude.clone(), tmp.path().join("never-made")];

    assert!(inside_any(&claude.canonicalize().unwrap(), &folders));
    assert!(inside_any(
        &claude.join("rules").canonicalize().unwrap(),
        &folders
    ));
    assert!(!inside_any(&tmp.path().canonicalize().unwrap(), &folders));
}

#[test]
fn without_a_home_the_folders_variables_name_are_still_guarded() {
    let tmp = tempfile::tempdir().unwrap();
    let claude = tmp.path().join("claude");
    let gemini = tmp.path().join("gemini-home");
    std::fs::create_dir_all(&claude).unwrap();
    std::fs::create_dir_all(gemini.join(".gemini")).unwrap();
    let alone = told_alone(&told(&[
        ("CLAUDE_CONFIG_DIR", &claude),
        ("GEMINI_CLI_HOME", &gemini),
    ]));

    assert!(inside_any(&claude.canonicalize().unwrap(), &alone));
    assert!(inside_any(
        &gemini.join(".gemini").canonicalize().unwrap(),
        &alone
    ));
    assert!(!inside_any(&gemini.canonicalize().unwrap(), &alone));
}
