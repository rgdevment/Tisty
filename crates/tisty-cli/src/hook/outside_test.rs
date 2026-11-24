use super::*;

fn nowhere() -> PathBuf {
    match cfg!(windows) {
        true => PathBuf::from(r"X:\Nowhere"),
        false => PathBuf::from("/nowhere"),
    }
}

fn home() -> PathBuf {
    nowhere().join("Person")
}

fn docs() -> PathBuf {
    home().join("Documents")
}

fn here() -> Here {
    Here {
        home: Some(home()),
        temp: nowhere().join("scratch"),
    }
}

fn said(at: &Path) -> String {
    at.display().to_string()
}

fn asks(name: &str) -> Option<PathBuf> {
    here().outside(&said(&docs().join(name)), "")
}

fn markers() -> Vec<&'static str> {
    HELD_BY.to_vec()
}

#[test]
fn a_new_text_file_in_a_plain_folder_asks_for_that_folder() {
    for name in [
        "a.md",
        "a.markdown",
        "a.txt",
        "a.rst",
        "a.adoc",
        "a.org",
        "A.MD",
        "Notes.TXT",
        "plan.v2.md",
        "handoff",
        "notes",
        "todo",
        "plan",
        "prompt",
        "brief",
        "Handoff",
    ] {
        assert_eq!(asks(name), Some(docs()), "{name}");
    }
}

#[test]
fn a_file_that_is_not_prose_never_asks() {
    for name in [
        "a.rs",
        "a.json",
        "a.html",
        "a.png",
        "a.toml",
        "a.md.bak",
        "Makefile",
        "README",
        "other",
        "notes.rs",
        "todo.json",
        "",
    ] {
        assert_eq!(asks(name), None, "{name:?}");
    }
}

#[test]
fn a_text_file_a_program_reads_never_asks() {
    for name in [
        "README.md",
        "readme.txt",
        "README",
        "LICENSE.txt",
        "LICENSE.md",
        "CHANGELOG.md",
        "requirements-dev.txt",
        "requirements.txt",
        "CLAUDE.md",
        "AGENTS.md",
        "GEMINI.md",
        "SKILL.md",
        "MEMORY.md",
        "robots.txt",
        "CMakeLists.txt",
        "llms.txt",
        "constraints.txt",
        "claude.md",
        "readme-first.md",
    ] {
        assert_eq!(asks(name), None, "{name}");
    }
}

#[test]
fn a_folder_set_apart_never_asks() {
    let here = Here {
        home: Some(home()),
        temp: docs().join("scratch"),
    };
    for folder in [
        docs().join(".claude"),
        docs().join(".hidden").join("deep"),
        docs().join("node_modules"),
        docs().join("target"),
        docs().join("vendor"),
        docs().join("site-packages"),
        docs().join("AppData"),
        docs().join("appdata").join("Local"),
        docs().join("ProgramData"),
        docs().join("Program Files"),
        docs().join("Program Files (x86)"),
        docs().join("tmp"),
        docs().join("Temp"),
        docs().join("scratch"),
        docs().join("scratch").join("deeper"),
    ] {
        let at = said(&folder.join("a.md"));
        assert_eq!(here.outside(&at, ""), None, "{folder:?}");
    }
}

#[test]
fn a_folder_another_session_leaves_notes_in_asks_even_under_a_folder_set_apart() {
    let side = docs()
        .join("AppData")
        .join("Local")
        .join(LEFT_BETWEEN_SESSIONS);
    let at = said(&side.join("a.md"));

    assert_eq!(here().outside(&at, ""), Some(side.clone()));
    let below = said(&side.join("local_1").join("a.md"));
    assert_eq!(here().outside(&below, ""), Some(side.join("local_1")));
}

#[test]
fn a_system_folder_never_asks_off_windows() {
    if cfg!(windows) {
        return;
    }
    for folder in [
        "/etc",
        "/usr/share",
        "/var/log",
        "/opt/x",
        "/private/x",
        "/Library/x",
    ] {
        assert_eq!(
            here().outside(&format!("{folder}/a.md"), ""),
            None,
            "{folder}"
        );
    }
}

#[test]
fn a_file_that_already_exists_never_asks_and_a_new_one_beside_it_does() {
    let tmp = tempfile::tempdir().unwrap();
    let side = tmp.path().join(LEFT_BETWEEN_SESSIONS);
    std::fs::create_dir_all(&side).unwrap();
    std::fs::write(side.join("old.md"), "x").unwrap();
    let here = Here {
        home: Some(tmp.path().to_path_buf()),
        temp: nowhere().join("scratch"),
    };

    assert_eq!(here.outside(&said(&side.join("old.md")), ""), None);
    assert_eq!(here.outside(&said(&side.join("new.md")), ""), Some(side));
}

#[test]
fn a_relative_path_is_placed_from_the_folder_it_runs_in() {
    let here = here();
    let cwd = said(&docs());
    for (path, expected) in [
        ("a.md", Some(docs())),
        ("./a.md", Some(docs())),
        ("sub/a.md", Some(docs().join("sub"))),
        ("../a.md", Some(home())),
        ("sub/../a.md", Some(docs())),
        ("../Other/a.md", Some(home().join("Other"))),
    ] {
        assert_eq!(here.outside(path, &cwd), expected, "{path}");
    }
}

#[test]
fn a_relative_path_with_no_folder_to_run_from_is_never_placed() {
    let here = here();
    assert_eq!(here.outside("a.md", ""), None);
    assert_eq!(here.outside("a.md", "relative/dir"), None);
    assert_eq!(here.outside("", &said(&docs())), None);
}

#[test]
fn an_absolute_path_ignores_the_folder_it_runs_in() {
    let at = said(&docs().join("a.md"));
    assert_eq!(here().outside(&at, "relative/dir"), Some(docs()));
    assert_eq!(here().outside(&at, &said(&nowhere())), Some(docs()));
}

#[test]
fn a_folder_the_person_keeps_a_project_in_claims_what_is_under_it() {
    let tmp = tempfile::tempdir().unwrap();
    for marker in markers() {
        let project = tmp.path().join(format!("project-{marker}"));
        let deep = project.join("a").join("b");
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(project.join(marker), "x").unwrap();
        let here = Here {
            home: Some(tmp.path().to_path_buf()),
            temp: nowhere().join("scratch"),
        };

        assert!(here.held(&project), "{marker}");
        assert!(here.held(&project.join("a")), "{marker}");
        assert!(here.held(&deep), "{marker}");
    }
}

#[test]
fn a_folder_with_no_project_above_it_claims_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let deep = tmp.path().join("a").join("b");
    std::fs::create_dir_all(&deep).unwrap();
    let here = Here {
        home: Some(tmp.path().to_path_buf()),
        temp: nowhere().join("scratch"),
    };

    assert!(!here.held(&deep));
    assert!(!here.held(tmp.path()));
}

#[test]
fn the_persons_own_folder_claims_nothing_but_a_folder_in_it_does() {
    let tmp = tempfile::tempdir().unwrap();
    let sub = tmp.path().join("sub");
    let deep = sub.join("deep");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();
    let here = Here {
        home: Some(tmp.path().to_path_buf()),
        temp: nowhere().join("scratch"),
    };

    assert!(!here.held(tmp.path()));
    assert!(!here.held(&sub));
    assert!(!here.held(&deep));

    std::fs::create_dir_all(sub.join(".git")).unwrap();

    assert!(here.held(&sub));
    assert!(here.held(&deep));
    assert!(!here.held(tmp.path()));
}

#[test]
fn without_a_home_what_is_above_still_claims() {
    let tmp = tempfile::tempdir().unwrap();
    let deep = tmp.path().join("a");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(tmp.path().join("Cargo.toml"), "x").unwrap();
    let here = Here {
        home: None,
        temp: nowhere().join("scratch"),
    };

    assert!(here.held(&deep));
}

#[cfg(windows)]
#[test]
fn the_persons_folder_is_compared_ignoring_case_on_windows() {
    let tmp = tempfile::tempdir().unwrap();
    let sub = tmp.path().join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    std::fs::write(tmp.path().join("package.json"), "x").unwrap();
    let shouted = PathBuf::from(tmp.path().display().to_string().to_uppercase());
    let stated = |home: PathBuf| Here {
        home: Some(home),
        temp: nowhere().join("scratch"),
    };

    assert!(!stated(shouted).held(&sub));
    assert!(stated(nowhere()).held(&sub));
}

#[cfg(windows)]
#[test]
fn a_path_of_another_shell_is_read_as_a_disk_on_windows() {
    for said in [
        "/c/Person/a.md",
        "/C/Person/a.md",
        "/mnt/c/Person/a.md",
        "/c:/Person/a.md",
        "C:/Person/a.md",
        "C:\\Person\\a.md",
    ] {
        assert_eq!(
            here().outside(said, ""),
            Some(PathBuf::from(r"C:\Person")),
            "{said}"
        );
    }
}

#[cfg(windows)]
#[test]
fn a_path_that_cannot_be_placed_is_never_asked_about_on_windows() {
    for said in [
        "/home/x/a.md",
        "/a.md",
        "/mnt/a.md",
        "\\\\server\\share\\a.md",
        "//server/share/a.md",
    ] {
        assert_eq!(here().outside(said, ""), None, "{said}");
    }
}

#[test]
fn a_shell_word_for_the_persons_folder_is_filled_in_before_it_is_placed() {
    for said in [
        "~/Documents/a.md",
        "$HOME/Documents/a.md",
        "${HOME}/Documents/a.md",
        "$home/Documents/a.md",
        "$env:USERPROFILE/Documents/a.md",
        "${env:USERPROFILE}/Documents/a.md",
        "%USERPROFILE%/Documents/a.md",
    ] {
        assert_eq!(here().outside_in_shell(said, ""), Some(docs()), "{said}");
    }
    assert_eq!(here().outside_in_shell("~/a.md", ""), Some(home()));
}

#[cfg(windows)]
#[test]
fn a_shell_word_for_the_persons_folder_is_filled_in_with_backslashes_on_windows() {
    for said in [
        "~\\Documents\\a.md",
        "$env:USERPROFILE\\Documents\\a.md",
        "%USERPROFILE%\\Documents\\a.md",
        "%userprofile%\\Documents\\a.md",
    ] {
        assert_eq!(here().outside_in_shell(said, ""), Some(docs()), "{said}");
    }
}

#[test]
fn a_shell_word_that_cannot_be_filled_in_is_never_asked_about() {
    for said in [
        "$OTHER/a.md",
        "*.md",
        "?.md",
        "`x`.md",
        "~other/a.md",
        "$HOME2/a.md",
        "%OTHER%/a.md",
        "$HOME/*.md",
        "~/$X/a.md",
    ] {
        assert_eq!(here().outside_in_shell(said, ""), None, "{said}");
    }
}

#[test]
fn a_word_for_the_persons_folder_with_no_home_is_never_asked_about() {
    let homeless = Here {
        home: None,
        temp: nowhere().join("scratch"),
    };
    assert_eq!(homeless.outside_in_shell("~/Documents/a.md", ""), None);
    assert_eq!(homeless.outside_in_shell("$HOME/a.md", ""), None);
    let plain = said(&docs().join("a.md"));
    assert_eq!(homeless.outside_in_shell(&plain, ""), Some(docs()));
}

#[test]
fn a_path_with_no_shell_word_is_placed_as_it_is_in_a_shell() {
    let at = said(&docs().join("a.md"));
    assert_eq!(here().outside_in_shell(&at, ""), Some(docs()));
    assert_eq!(
        here().outside_in_shell("a.md", &said(&docs())),
        Some(docs())
    );
}

#[test]
fn only_a_relative_path_depends_on_where_the_command_runs() {
    for (said, depends) in [
        ("a.md", true),
        ("sub/a.md", true),
        ("./a.md", true),
        ("../a.md", true),
        ("~/a.md", false),
        ("~", false),
        ("$HOME/a.md", false),
        ("${HOME}/a.md", false),
        ("$env:USERPROFILE/a.md", false),
        ("%USERPROFILE%/a.md", false),
        ("$home/a.md", false),
    ] {
        assert_eq!(depends_on_where_it_runs(said), depends, "{said}");
    }
    assert!(!depends_on_where_it_runs(&said(&docs().join("a.md"))));
}
