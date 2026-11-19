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
    let places = |args: Value| places_named("folder", &args).collect::<Vec<_>>();
    let alone = |args: Value| named_alone("folder", &args);
    assert_eq!(
        alone(json!({ "name": "Claude" })).as_deref(),
        Some("Claude")
    );
    assert_eq!(
        alone(json!({ "name": "Claude", "inside": " " })).as_deref(),
        Some("Claude")
    );
    assert_eq!(alone(json!({ "name": "Claude", "inside": "Notes" })), None);
    assert_eq!(alone(json!({ "name": "Notes / Claude" })), None);
    assert_eq!(places(json!({ "name": "AGENTS / Codex" })), vec!["AGENTS"]);
    assert_eq!(
        places(json!({ "name": "Codex", "inside": " Notes" })),
        vec!["Notes"]
    );
    assert!(places(json!({ "name": "", "inside": "" })).is_empty());
    assert_eq!(
        places_named("file_doc", &json!({ "folder": "Notes", "name": "x" })).collect::<Vec<_>>(),
        vec!["Notes"]
    );
    assert_eq!(named_alone("file_doc", &json!({ "name": "AGENTS" })), None);
    assert!(makes_agents(
        "folder",
        &json!({ "name": "Work / AGEN\tTS" })
    ));
    assert!(!makes_agents("folder", &json!({ "name": "Agent" })));
    assert!(!makes_agents("write_doc", &json!({ "name": "AGENTS" })));
}
