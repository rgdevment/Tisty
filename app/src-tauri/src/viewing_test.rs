use super::{Views, asked, asked_to_edit, wanted};

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
fn what_the_shell_hands_over_after_the_separator_still_opens() {
    let room = tempfile::tempdir().unwrap();
    let real = written(&room, "real.md");
    let argv = vec!["tisty-gui".to_string(), "--".to_string(), linked(&real)];

    assert_eq!(wanted(&argv, Some(room.path())), vec![real]);
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

#[test]
fn a_viewer_opens_wide_enough_for_its_index_and_a_fifth_shorter_than_tisty() {
    assert_eq!(super::scaled(820.0), (1400.0, 656.0));
    assert_eq!(super::scaled(700.0), (1400.0, 560.0));
    assert_eq!(super::scaled(750.0), (1400.0, 600.0));
}

fn in_the_contract(name: &str) -> String {
    let contract: serde_json::Value =
        serde_json::from_str(tisty_contracts::CONTRACT).expect("the contract is JSON");
    contract["contracts"]["link"]["values"]["edit_example"][name]
        .as_str()
        .expect("what the contract gives as an example")
        .to_string()
}

#[test]
fn the_edit_address_of_the_contract_hands_over_its_link_whole() {
    let found = asked_to_edit(&in_the_contract("address")).expect("the contract's own address");
    assert_eq!(found.as_str(), in_the_contract("link"));
    assert_eq!(asked(&in_the_contract("address"), None), None);
}

#[test]
fn an_edit_address_with_anything_more_or_from_elsewhere_is_refused() {
    for said in [
        "tisty://edit",
        "tisty://edit?link=https%3A%2F%2Ftisty.cl%2Fx&link=https%3A%2F%2Ftisty.cl%2Fy",
        "tisty://edit?link=https%3A%2F%2Ftisty.cl%2Fx&folder=y",
        "tisty://edit?folder=y&link=https%3A%2F%2Ftisty.cl%2Fx",
        "tisty://edit/more?link=https%3A%2F%2Ftisty.cl%2Fx",
        "tisty://edit?link=https://tisty.cl/x#the-key",
        "tisty://edit?link=https%3A%2F%2Fevil.example%2Fx",
        "tisty://edit?link=http%3A%2F%2Ftisty.cl%2Fx",
        "tisty://edit?link=file%3A%2F%2F%2FC%3A%2Fx.md",
        "tisty://folio?link=https%3A%2F%2Ftisty.cl%2Fx",
    ] {
        assert_eq!(asked_to_edit(said), None, "{said}");
    }
}

#[test]
fn a_link_owed_to_the_welcome_is_due_once_however_often_it_was_fired() {
    let views = super::Views::default();
    let link = url::Url::parse("https://tisty.cl/Ab3xK9pQ#key").unwrap();
    assert!(views.links_due().is_empty());
    views.owe_link(link.clone());
    views.owe_link(link.clone());
    assert_eq!(views.links_due(), [link]);
    assert!(views.links_due().is_empty());
}

#[test]
fn an_edit_owed_to_the_welcome_is_due_once_however_often_it_was_asked_for() {
    let views = super::Views::default();
    assert!(views.edits_due().is_empty());
    views.owe_edit("view-1");
    views.owe_edit("view-1");
    views.owe_edit("view-2");
    assert_eq!(views.edits_due(), ["view-1", "view-2"]);
    assert!(views.edits_due().is_empty());
}

#[test]
fn only_a_file_from_outside_gets_the_systems_title_bar() {
    let file = super::Shown::File(std::path::PathBuf::from("a.md"));
    assert_eq!(
        super::look_of(&file),
        super::Look {
            framed: true,
            clear: false
        }
    );
    let paper = super::Paper {
        id: "D1".into(),
        file: "f1".into(),
        title: "Notas".into(),
        place: String::new(),
    };
    for own in [
        super::Shown::Doc(paper.clone()),
        super::Shown::Widget {
            source: "<p>x</p>".into(),
            from: Some(paper),
        },
    ] {
        assert_eq!(
            super::look_of(&own),
            super::Look {
                framed: false,
                clear: true
            }
        );
    }
}

#[test]
fn a_link_waits_for_no_second_question_while_one_stands() {
    let views = Views::default();

    let asking = views.asking_of_a_link();
    assert!(asking.is_some());
    assert!(views.asking_of_a_link().is_none());

    drop(asking);
    assert!(views.asking_of_a_link().is_some());
}
