use serde_json::{Value, json};
use tisty_core::State;
use tisty_core::model::{Folder, FolderId, Kept};
use ulid::Ulid;

use super::{
    folder_left_out, kept_in_agents, left_out, loosely, makes_agents, meant_document,
    meant_in_agents, named_alone, place_in_agents, places_named,
};

struct Shelf {
    state: State,
    root: FolderId,
    claude: FolderId,
    notes: FolderId,
}

fn folder(id: FolderId, name: &str, parent: Option<FolderId>, icon: Option<&str>) -> Folder {
    serde_json::from_value(json!({
        "id": id, "name": name, "order": "a0", "parent": parent, "icon": icon,
    }))
    .unwrap()
}

fn kept(file: &str, folder: Option<FolderId>, page_of: Option<Ulid>) -> Kept {
    serde_json::from_value(json!({
        "id": Ulid::generate(), "file": file, "order": "a0", "folder": folder, "page_of": page_of,
    }))
    .unwrap()
}

fn shelf() -> Shelf {
    let (root, claude, notes) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let mut state = State::default();
    for one in [
        folder(root, "AGENTS", None, Some("robot")),
        folder(claude, "Claude", Some(root), None),
        folder(notes, "Notes", None, None),
    ] {
        state.folders.insert(one.id, one);
    }
    let rule = kept("ab12cd34-0001", Some(claude), None);
    let under = kept("ab12cd34-0002", None, Some(rule.id));
    let mine = kept("ab12cd34-0003", Some(notes), None);
    for one in [rule, under, mine] {
        state.docs.insert(one.id, one);
    }
    Shelf {
        state,
        root,
        claude,
        notes,
    }
}

fn named(shelf: &Shelf, file: &str) -> Kept {
    shelf
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .unwrap()
        .clone()
}

#[test]
fn names_that_read_alike_once_blanks_and_hidden_marks_go_are_one() {
    for said in [
        "AGENTS",
        " agents ",
        "AGEN\tTS",
        "AGENTS\n",
        "AGENTS\u{2066}",
        "\u{202e}AGENTS",
        "AGE\u{200b}NTS",
        "\u{feff}AGENTS\u{00ad}",
        "ÁGENTS",
        "agents\u{fe0f}",
    ] {
        assert_eq!(loosely(said), "agents", "{said:?}");
    }
    assert_ne!(loosely("AGENTS 2"), "agents");
    assert_ne!(loosely("Agent"), "agents");
}

#[test]
fn a_document_is_meant_by_its_id_however_it_is_wrapped() {
    for said in [
        "4nnctnme-0040",
        " 4NNCTNME-0040\n",
        "\u{2066}4nnctnme-0040",
        "tisty://doc/claude-md?id=4nnctnme-0040",
        " tisty://doc/claude-md?id= 4nnctnme-0040",
        "tisty://doc/claude-md?id=\u{200b}4nnctnme-0040",
    ] {
        assert_eq!(meant_document(said), "4nnctnme-0040", "{said:?}");
    }
    assert_eq!(meant_document("Notes"), "notes");
}

#[test]
fn only_the_exact_name_of_a_document_in_agents_may_become_a_proposal() {
    let shelf = shelf();
    let found = |said: &str| {
        kept_in_agents(&shelf.state, said).map(|(one, exactly)| (one.file.clone(), exactly))
    };
    assert_eq!(found("ab12cd34-0001"), Some(("ab12cd34-0001".into(), true)));
    assert_eq!(
        found("AB12CD34-0001"),
        Some(("ab12cd34-0001".into(), false))
    );
    assert_eq!(
        found("ab12cd34-0001\u{2066}"),
        Some(("ab12cd34-0001".into(), false))
    );
    assert_eq!(found("ab12cd34-0002"), Some(("ab12cd34-0002".into(), true)));
    assert_eq!(found("ab12cd34-0003"), None);
    assert_eq!(found(" ab12cd34-0003"), None);
}

#[test]
fn a_tool_that_found_a_document_in_agents_still_writes_nothing_there() {
    let shelf = shelf();
    for file in ["ab12cd34-0001", "ab12cd34-0002"] {
        let refused = left_out(&shelf.state, &named(&shelf, file));
        assert!(matches!(refused, Err(super::Refused::Tool(_))), "{file}");
    }
    assert!(left_out(&shelf.state, &named(&shelf, "ab12cd34-0003")).is_ok());
}

#[test]
fn a_folder_in_agents_or_named_like_it_is_never_touched_by_folder() {
    let shelf = shelf();
    let refused = |at, name: &str| folder_left_out(&shelf.state, at, name).is_err();
    assert!(refused(Some(shelf.root), "AGENTS"));
    assert!(refused(Some(shelf.claude), "Claude"));
    assert!(refused(None, "ÁGENTS"));
    assert!(refused(Some(shelf.notes), "agents"));
    assert!(!refused(Some(shelf.notes), "Claude"));
    assert!(!refused(None, "Notes"));
}

#[test]
fn a_folder_in_agents_is_meant_by_its_id_its_name_or_its_path_however_written() {
    let shelf = shelf();
    let claude = shelf.claude.to_string();
    for said in [
        " AGENTS".to_string(),
        "agents / CLAUDE".to_string(),
        "\tClaude\u{2066}".to_string(),
        claude.to_lowercase(),
        format!(" {claude}\n"),
    ] {
        assert!(meant_in_agents(&shelf.state, &said), "{said:?}");
    }
    assert!(!meant_in_agents(&shelf.state, "Notes"));
    assert!(!meant_in_agents(&shelf.state, "Notes / Claude"));
    assert!(!meant_in_agents(&State::default(), "AGENTS"));
}

#[test]
fn a_place_the_tools_resolve_outside_agents_stays_open_even_named_like_one_inside() {
    let mut shelf = shelf();
    let mine = folder(Ulid::generate(), "Claude", None, None);
    shelf.state.folders.insert(mine.id, mine);
    assert!(!place_in_agents(&shelf.state, "Claude"));
    assert!(!place_in_agents(&shelf.state, "Notes"));
    assert!(place_in_agents(&shelf.state, "AGENTS / Claude"));
    assert!(place_in_agents(&shelf.state, &shelf.claude.to_string()));
    assert!(place_in_agents(&shelf.state, "AGENTS\u{2066}"));
    assert!(place_in_agents(&shelf.state, "Claude\u{200b}"));
    assert!(!place_in_agents(&shelf.state, "Nowhere"));
}

#[test]
fn the_folder_tool_is_judged_by_the_place_it_reads_from_its_name() {
    let places = |args: Value| places_named("doc_organize", &args).collect::<Vec<_>>();
    let alone = |args: Value| named_alone("doc_organize", &args);
    assert_eq!(
        alone(json!({ "action": "create_folder", "name": "Claude" })).as_deref(),
        Some("Claude")
    );
    assert_eq!(
        alone(json!({ "action": "create_folder", "name": "Claude", "folder": " " })).as_deref(),
        Some("Claude")
    );
    assert_eq!(
        alone(json!({ "action": "create_folder", "name": "Claude", "folder": "Notes" })),
        None
    );
    assert_eq!(
        alone(json!({ "action": "create_folder", "name": "Notes / Claude" })),
        None
    );
    assert_eq!(
        places(json!({ "action": "create_folder", "name": "AGENTS / Codex" })),
        vec!["AGENTS"]
    );
    assert_eq!(
        places(json!({ "action": "create_folder", "name": "Codex", "folder": " Notes" })),
        vec!["Notes"]
    );
    assert!(places(json!({ "action": "create_folder", "name": "", "folder": "" })).is_empty());
    assert_eq!(
        places_named(
            "doc_organize",
            &json!({ "action": "file", "folder": "Notes", "name": "x" })
        )
        .collect::<Vec<_>>(),
        vec!["Notes"]
    );
    assert_eq!(
        named_alone(
            "doc_organize",
            &json!({ "action": "file", "name": "AGENTS" })
        ),
        None
    );
    assert!(makes_agents(
        "doc_organize",
        &json!({ "action": "create_folder", "name": "Work / AGEN\tTS" })
    ));
    assert!(!makes_agents(
        "doc_organize",
        &json!({ "action": "create_folder", "name": "Agent" })
    ));
    assert!(!makes_agents("doc_create", &json!({ "name": "AGENTS" })));
}

struct Home {
    _room: tempfile::TempDir,
    paths: tisty_core::Paths,
    rule: String,
    other: String,
    picture: String,
}

fn written(paths: &tisty_core::Paths, body: &str, folder: Option<FolderId>) -> String {
    let config = tisty_core::Config::load_or_init(paths).unwrap();
    let made = tisty_core::docs::create(&paths.docs(), &config.device_id, body).unwrap();
    persons(
        paths,
        vec![tisty_core::Op::DocAdd {
            id: Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: made.id.clone(),
                order: "a0".into(),
                said: Some(tisty_core::event::Said::of(body).agreed()),
                folder,
                ..Default::default()
            },
        }],
    );
    made.id
}

fn persons(paths: &tisty_core::Paths, ops: Vec<tisty_core::Op>) {
    let config = tisty_core::Config::load_or_init(paths).unwrap();
    tisty_core::Store::open(paths.store(), config.device_id.clone())
        .unwrap()
        .signing_with(tisty_core::signing::mine(paths, &config.device_id))
        .append_batch_tagged(ops, false, false)
        .unwrap();
}

fn home() -> Home {
    let room = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(room.path().join("data"), room.path().join("config"));
    persons(&paths, tisty_core::model::sown("en"));
    tisty_core::agent::register(&paths).unwrap();
    let (root, claude, notes) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let made = |id, name: &str, parent, icon: Option<&str>| tisty_core::Op::FolderAdd {
        id,
        d: tisty_core::event::FolderAdd {
            name: name.into(),
            order: "a0".into(),
            parent,
            icon: icon.map(str::to_string),
            color: None,
            computer: None,
        },
    };
    persons(
        &paths,
        vec![
            made(
                root,
                tisty_core::agents::FOLDER,
                None,
                Some(tisty_core::agents::ICON),
            ),
            made(claude, "Claude", Some(root), None),
            made(notes, "Notes", None, None),
        ],
    );
    let rule = written(
        &paths,
        "---\nagent: claude/rule/CLAUDE.md\ntitle: claude · CLAUDE.md\n---\n# Claude\nTests first.\n",
        Some(claude),
    );
    let other = written(&paths, "# Notes\n\nMine.\n", Some(notes));
    let picture = room.path().join("foto.txt");
    std::fs::write(&picture, "una foto").unwrap();
    Home {
        picture: picture.to_string_lossy().into_owned(),
        _room: room,
        paths,
        rule,
        other,
    }
}

#[test]
fn a_tool_past_the_door_still_leaves_an_agents_document_alone() {
    let home = home();
    let before = tisty_core::docs::read(&home.paths.docs(), &home.rule).unwrap();
    let ops = |paths: &tisty_core::Paths| tisty_core::store::read_all(paths.store()).unwrap().len();
    let at_first = ops(&home.paths);

    for (name, args) in [
        (
            "doc_annotate",
            json!({ "action": "archive", "docs": [home.rule] }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [home.rule], "folder": "Notes" }),
        ),
        (
            "doc_annotate",
            json!({ "action": "flag", "docs": [home.rule], "reason": "obsolete", "body": "viejo" }),
        ),
        (
            "doc_organize",
            json!({ "action": "add_page", "docs": [home.rule], "parent": home.other }),
        ),
        (
            "doc_organize",
            json!({ "action": "add_page", "docs": [home.other], "parent": home.rule }),
        ),
        (
            "doc_organize",
            json!({ "action": "link_translation", "docs": [home.rule], "parent": home.other }),
        ),
        (
            "doc_organize",
            json!({ "action": "link_translation", "docs": [home.other], "parent": home.rule }),
        ),
        (
            "file_transfer",
            json!({ "action": "attach", "doc": home.rule, "path": home.picture }),
        ),
    ] {
        let said = super::super::answered(&home.paths, name, args.clone(), None);
        match said {
            Err(super::super::Refused::Tool(why)) => {
                assert!(
                    why.contains("only the person changes"),
                    "{name} {args}: {why}"
                )
            }
            Err(_) => panic!("{name} {args}: refused for another reason"),
            Ok(said) => panic!("{name} {args} went through: {said}"),
        }
    }
    assert_eq!(
        tisty_core::docs::read(&home.paths.docs(), &home.rule).unwrap(),
        before
    );
    assert_eq!(ops(&home.paths), at_first, "a tool wrote past the door");
}
