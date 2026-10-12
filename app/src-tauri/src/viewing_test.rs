use super::asked;

fn written(room: &tempfile::TempDir, name: &str) -> std::path::PathBuf {
    let at = room.path().join(name);
    std::fs::write(&at, "# hi").unwrap();
    at
}

fn linked(at: &std::path::Path) -> String {
    let mut link = url::Url::parse("tisty://open").unwrap();
    link.query_pairs_mut()
        .append_pair("path", &at.display().to_string());
    link.to_string()
}

#[test]
fn a_whole_path_to_a_markdown_file_is_opened_however_it_comes() {
    let room = tempfile::tempdir().unwrap();
    let at = written(&room, "notes.MD");

    assert_eq!(asked(&at.display().to_string(), None), Some(at.clone()));
    assert_eq!(asked(&linked(&at), None), Some(at.clone()));
    let file = url::Url::from_file_path(&at).unwrap();
    assert_eq!(asked(file.as_str(), None), Some(at));
}

#[test]
fn a_name_given_in_a_terminal_is_found_where_that_terminal_stands() {
    let room = tempfile::tempdir().unwrap();
    let at = written(&room, "readme.md");

    assert_eq!(asked("readme.md", Some(room.path())), Some(at));
    assert_eq!(asked("readme.md", None), None);
    assert_eq!(
        asked("tisty://open?path=readme.md", Some(room.path())),
        None,
        "a link names a whole path or nothing"
    );
}

#[test]
fn nothing_but_a_readable_local_text_file_passes() {
    let room = tempfile::tempdir().unwrap();
    let script = written(&room, "run.exe");
    let folder = room.path().join("a.md");
    std::fs::create_dir(&folder).unwrap();
    let real = written(&room, "real.md");
    let climbing = room.path().join("x").join("..").join("real.md");

    for said in [
        script.display().to_string(),
        folder.display().to_string(),
        climbing.display().to_string(),
        room.path().join("missing.md").display().to_string(),
        "--hushed".to_string(),
        "https://example.com/a.md".to_string(),
        r"\\evil.example\share\a.md".to_string(),
        "tisty://open?path=%5C%5Cevil.example%5Cshare%5Ca.md".to_string(),
        "file://evil.example/share/a.md".to_string(),
        format!("tisty://import?path={}", real.display()),
        format!("{}&path={}", linked(&real), real.display()),
        "tisty://open".to_string(),
    ] {
        assert_eq!(asked(&said, Some(room.path())), None, "{said}");
    }
}
