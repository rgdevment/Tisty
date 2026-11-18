use super::{
    LinkQuestion, Paper, Shown, Views, asked, asked_for_task, asked_to_edit, docs_named,
    paper_asked, shows, task_link, wanted,
};

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
    let found =
        asked_to_edit(&in_the_contract("address"), "tisty.cl").expect("the contract's own address");
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
        assert_eq!(asked_to_edit(said, "tisty.dev"), None, "{said}");
    }
}

#[test]
fn an_edit_address_on_the_private_host_folio_signed_is_taken() {
    let said = "tisty://edit?link=https%3A%2F%2Ftisty.dev%2FAb3xK9pQ%23key";
    assert_eq!(
        asked_to_edit(said, "tisty.dev").map(|one| one.to_string()),
        Some("https://tisty.dev/Ab3xK9pQ#key".to_owned())
    );
    assert_eq!(asked_to_edit(said, "tisty.cl"), None);
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
fn only_what_the_light_viewer_shows_gets_the_systems_title_bar() {
    let paper = super::Paper {
        id: "D1".into(),
        file: "f1".into(),
        title: "Notas".into(),
        place: String::new(),
    };
    for light in [
        super::Shown::File(std::path::PathBuf::from("a.md")),
        super::Shown::Doc(paper.clone()),
    ] {
        assert_eq!(
            super::look_of(&light),
            super::Look {
                framed: true,
                clear: false
            }
        );
    }
    let widget = super::Shown::Piece {
        held: tisty_markdown::diagrams::Held::Widget,
        source: "<p>x</p>".into(),
        from: Some(paper),
        place: None,
        code: false,
    };
    assert_eq!(
        super::look_of(&widget),
        super::Look {
            framed: false,
            clear: true
        }
    );
}

fn link_numbered(at: usize) -> url::Url {
    url::Url::parse(&format!("https://tisty.cl/d{at}#k")).unwrap()
}

#[test]
fn links_wait_their_turn_and_only_the_first_goes_through_them() {
    let views = Views::default();

    assert!(views.link_arrived(link_numbered(1)));
    assert!(!views.link_arrived(link_numbered(2)));
    assert!(!views.link_arrived(link_numbered(1)));

    assert_eq!(views.next_link(), Some(link_numbered(1)));
    assert_eq!(views.next_link(), Some(link_numbered(2)));
    assert_eq!(views.next_link(), None);
    assert!(views.link_arrived(link_numbered(3)));
}

#[test]
fn a_link_fired_again_while_it_is_handled_is_not_asked_about_twice() {
    let views = Views::default();
    assert!(views.link_arrived(link_numbered(1)));
    assert_eq!(views.next_link(), Some(link_numbered(1)));

    assert!(!views.link_arrived(link_numbered(1)));
    assert_eq!(views.next_link(), None);

    assert!(views.link_arrived(link_numbered(1)));
    assert_eq!(views.next_link(), Some(link_numbered(1)));
}

#[test]
fn a_link_that_breaks_does_not_stop_the_ones_after_it() {
    let views = Views::default();
    views.link_arrived(link_numbered(1));
    views.link_arrived(link_numbered(2));
    let brought = std::cell::RefCell::new(Vec::new());

    views.go_through_links(|link| {
        assert_ne!(link, &link_numbered(1), "this one breaks");
        brought.borrow_mut().push(link.clone());
    });

    assert_eq!(brought.into_inner(), [link_numbered(2)]);
    assert!(views.link_arrived(link_numbered(3)));
}

#[test]
fn a_page_firing_links_cannot_line_up_more_than_a_few() {
    let views = Views::default();

    for at in 0..100 {
        views.link_arrived(link_numbered(at));
    }

    let mut waiting = 0;
    while views.next_link().is_some() {
        waiting += 1;
    }
    assert_eq!(waiting, super::LINKS_WAITING_AT_MOST);
}

#[test]
fn a_question_stands_until_it_is_answered_and_says_the_answer() {
    let views = Views::default();
    assert_eq!(views.link_standing(), None);

    let answer = views.ask_of_a_link(LinkQuestion::Look);
    let shown = views.link_standing().unwrap();
    assert_eq!(shown.question, LinkQuestion::Look);

    views.link_answered(shown.id, true);
    assert_eq!(answer.recv(), Ok(true));
    assert_eq!(views.link_standing(), None);
}

#[test]
fn an_answer_to_an_earlier_question_does_not_reach_the_one_standing() {
    let views = Views::default();
    let first = views.ask_of_a_link(LinkQuestion::Look);
    let earlier = views.link_standing().unwrap().id;
    views.link_answered(earlier, false);
    assert_eq!(first.recv(), Ok(false));
    let next = views.ask_of_a_link(LinkQuestion::Look);

    views.link_answered(earlier, true);

    assert!(views.link_standing().is_some());
    assert!(next.try_recv().is_err());
}

#[test]
fn the_languages_left_behind_reach_the_window_by_their_tags() {
    let said = serde_json::to_value(LinkQuestion::LanguagesLeft {
        languages: vec!["en".into(), "fr".into()],
    })
    .unwrap();

    assert_eq!(
        said,
        serde_json::json!({ "kind": "languagesLeft", "languages": ["en", "fr"] })
    );
}

#[test]
fn an_answer_with_no_question_standing_goes_nowhere() {
    let views = Views::default();

    views.link_answered(0, true);

    assert_eq!(views.link_standing(), None);
}

const TASK: &str = "01KA7B3VJ9RZ0Q8M5N2X4Y6W1T";

#[test]
fn a_reminder_s_address_opens_its_task() {
    assert_eq!(asked_for_task(&task_link(TASK)).as_deref(), Some(TASK));
}

#[test]
fn a_task_address_carries_nothing_but_a_task_id() {
    for said in [
        "tisty://task?id=not-a-task".to_string(),
        format!("tisty://task?id={TASK}&path=C:/x.md"),
        format!("tisty://task?path={TASK}"),
        format!("tisty://task/elsewhere?id={TASK}"),
        format!("tisty://task?id={TASK}#more"),
        format!("tisty://open?id={TASK}"),
        format!("https://task?id={TASK}"),
        "tisty://task".to_string(),
    ] {
        assert_eq!(asked_for_task(&said), None, "{said}");
    }
}

#[test]
fn a_task_address_is_no_file_to_open() {
    assert!(wanted(&["tisty".to_string(), task_link(TASK)], None).is_empty());
}

const DOC: &str = "4nnctnme-0039";
const DOC_ADDRESS: &str = "tisty://doc/personal/proyectos/validacion-de-datos?id=4nnctnme-0039";

fn said(argv: &[&str]) -> Vec<String> {
    argv.iter().map(|one| one.to_string()).collect()
}

fn folder(state: &mut tisty_core::State, name: &str, archived: bool) -> ulid::Ulid {
    let id = ulid::Ulid::generate();
    let mut one = tisty_core::model::Folder::new(id, name, name);
    one.archived = archived;
    state.folders.insert(id, one);
    id
}

fn kept(state: &mut tisty_core::State, file: &str, more: serde_json::Value) -> ulid::Ulid {
    let id = ulid::Ulid::generate();
    let mut said = serde_json::json!({ "id": id.to_string(), "file": file, "order": file });
    for (key, value) in more.as_object().unwrap() {
        said[key] = value.clone();
    }
    state.docs.insert(id, serde_json::from_value(said).unwrap());
    id
}

#[test]
fn a_document_address_among_the_arguments_names_its_document_once() {
    let argv = said(&[
        "tisty-gui",
        "--",
        DOC_ADDRESS,
        "tisty://doc?id=4nnctnme-0039",
        "tisty://doc/otro?id=4nnctnme-0040",
        "tisty://doc/sin-id",
        "C:/notes.md",
    ]);

    assert_eq!(docs_named(&argv), ["4nnctnme-0039", "4nnctnme-0040"]);
    assert!(docs_named(&said(&[DOC_ADDRESS])).is_empty());
}

#[test]
fn a_document_asked_for_before_the_store_opens_waits_once_and_is_handed_over_once() {
    let views = Views::default();

    views.owe_doc(DOC);
    views.owe_doc("4nnctnme-0040");
    views.owe_doc(DOC);

    assert_eq!(views.docs_due(), [DOC, "4nnctnme-0040"]);
    assert!(views.docs_due().is_empty());
}

#[test]
fn what_is_not_here_shares_one_window_and_a_document_has_its_own() {
    let here = |id: &str, file: &str| Paper {
        id: id.into(),
        file: file.into(),
        title: "Tisty".into(),
        place: String::new(),
    };
    let gone = Shown::Doc(here("", "zzzzzzzz-0001"));
    let open = Shown::Doc(here("01KA7B3VJ9RZ0Q8M5N2X4Y6W1T", DOC));

    assert!(shows(&gone, &here("", "zzzzzzzz-0002")));
    assert!(shows(&open, &here("01KA7B3VJ9RZ0Q8M5N2X4Y6W1T", DOC)));
    assert!(!shows(&open, &here("", "zzzzzzzz-0002")));
    assert!(!shows(
        &open,
        &here("01KA7B3VJ9RZ0Q8M5N2X4Y6W1U", "4nnctnme-0040")
    ));
    assert!(!shows(&Shown::File("C:/notes.md".into()), &here("", DOC)));
}

#[test]
fn a_document_address_is_no_file_to_open_no_task_and_nothing_to_edit() {
    assert!(wanted(&said(&["tisty", DOC_ADDRESS]), None).is_empty());
    assert_eq!(asked_for_task(DOC_ADDRESS), None);
    assert_eq!(asked_to_edit(DOC_ADDRESS, "tisty.cl"), None);
}

#[test]
fn a_document_at_hand_is_shown_with_its_title_and_where_it_sits() {
    let mut state = tisty_core::State::default();
    let personal = folder(&mut state, "Personal", false);
    let projects = folder(&mut state, "Proyectos", false);
    state.folders.get_mut(&projects).unwrap().parent = Some(personal);
    let id = kept(
        &mut state,
        DOC,
        serde_json::json!({ "title": " Validación de datos ", "folder": projects.to_string() }),
    );

    assert_eq!(
        paper_asked(&state, DOC),
        Some(Paper {
            id: id.to_string(),
            file: DOC.into(),
            title: "Validación de datos".into(),
            place: "Personal / Proyectos".into(),
        })
    );
}

#[test]
fn a_page_is_shown_as_sitting_where_its_book_does() {
    let mut state = tisty_core::State::default();
    let work = folder(&mut state, "Trabajo", false);
    let book = kept(
        &mut state,
        "4nnctnme-0001",
        serde_json::json!({ "title": "Actas", "folder": work.to_string() }),
    );
    kept(
        &mut state,
        DOC,
        serde_json::json!({ "title": "Marzo", "page_of": book.to_string() }),
    );

    let paper = paper_asked(&state, DOC).unwrap();

    assert_eq!(
        (paper.title.as_str(), paper.place.as_str()),
        ("Marzo", "Trabajo")
    );
}

#[test]
fn a_document_with_no_title_still_names_its_window() {
    let mut state = tisty_core::State::default();
    kept(&mut state, DOC, serde_json::json!({}));

    let paper = paper_asked(&state, DOC).unwrap();

    assert_eq!((paper.title.as_str(), paper.place.as_str()), ("Tisty", ""));
}

#[test]
fn a_document_put_away_is_still_shown_apart_to_be_read() {
    let mut state = tisty_core::State::default();
    let old = folder(&mut state, "Viejo", true);
    kept(
        &mut state,
        "4nnctnme-0001",
        serde_json::json!({ "title": "Archivado", "archived": true }),
    );
    kept(
        &mut state,
        "4nnctnme-0002",
        serde_json::json!({ "title": "En carpeta archivada", "folder": old.to_string() }),
    );

    let own = paper_asked(&state, "4nnctnme-0001").unwrap();
    let held = paper_asked(&state, "4nnctnme-0002").unwrap();

    assert_eq!((own.title.as_str(), own.place.as_str()), ("Archivado", ""));
    assert_eq!(
        (held.title.as_str(), held.place.as_str()),
        ("En carpeta archivada", "Viejo")
    );
}

#[test]
fn a_task_s_page_and_a_document_of_another_machine_are_no_document_here() {
    let mut state = tisty_core::State::default();
    kept(
        &mut state,
        "4nnctnme-0003",
        serde_json::json!({ "task_of": ulid::Ulid::generate().to_string() }),
    );

    for file in ["4nnctnme-0003", "4nnctnme-0004"] {
        assert_eq!(paper_asked(&state, file), None, "{file}");
    }
}
