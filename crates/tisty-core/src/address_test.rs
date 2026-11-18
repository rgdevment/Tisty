use super::{doc_asked, of_doc};
use crate::State;
use crate::model::{Folder, Kept};
use ulid::Ulid;

const ID: &str = "4nnctnme-0039";

fn folder(state: &mut State, name: &str, parent: Option<Ulid>) -> Ulid {
    let id = Ulid::generate();
    let mut one = Folder::new(id, name, name);
    one.parent = parent;
    state.folders.insert(id, one);
    id
}

fn kept(
    state: &mut State,
    file: &str,
    title: Option<&str>,
    folder: Option<Ulid>,
    page_of: Option<Ulid>,
) -> Ulid {
    let id = Ulid::generate();
    let one: Kept = serde_json::from_value(serde_json::json!({
        "id": id.to_string(), "file": file, "order": file, "title": title,
        "folder": folder.map(|at| at.to_string()),
        "page_of": page_of.map(|of| of.to_string()),
    }))
    .unwrap();
    state.docs.insert(id, one);
    id
}

fn address(state: &State, id: Ulid) -> String {
    of_doc(state, &state.docs[&id])
}

#[test]
fn an_address_says_where_the_document_sits_and_ends_in_its_id() {
    let mut state = State::default();
    let personal = folder(&mut state, "Personal", None);
    let projects = folder(&mut state, "Proyectos", Some(personal));
    let doc = kept(
        &mut state,
        ID,
        Some("Validación de datos"),
        Some(projects),
        None,
    );

    let said = address(&state, doc);

    assert_eq!(
        said,
        "tisty://doc/personal/proyectos/validacion-de-datos?id=4nnctnme-0039"
    );
    assert_eq!(doc_asked(&said), Some(ID));
}

#[test]
fn a_page_is_named_after_its_book_and_the_folder_the_book_sits_in() {
    let mut state = State::default();
    let work = folder(&mut state, "Trabajo", None);
    let book = kept(&mut state, "4nnctnme-0001", Some("Actas"), Some(work), None);
    let page = kept(&mut state, ID, Some("Marzo"), None, Some(book));

    assert_eq!(
        address(&state, page),
        "tisty://doc/trabajo/actas/marzo?id=4nnctnme-0039"
    );
}

#[test]
fn a_name_keeps_only_what_any_keyboard_types() {
    let mut state = State::default();
    let odd = folder(&mut state, "¿Qué?  —  C++ & más…", None);
    let doc = kept(
        &mut state,
        ID,
        Some("  Año 2026: señales  "),
        Some(odd),
        None,
    );

    assert_eq!(
        address(&state, doc),
        "tisty://doc/que-c-mas/ano-2026-senales?id=4nnctnme-0039"
    );
}

#[test]
fn a_document_with_no_name_to_spell_is_its_id_alone() {
    let mut state = State::default();
    let untitled = kept(&mut state, ID, None, None, None);
    let unspelled = kept(&mut state, "4nnctnme-0040", Some("日本語"), None, None);

    assert_eq!(address(&state, untitled), "tisty://doc?id=4nnctnme-0039");
    assert_eq!(address(&state, unspelled), "tisty://doc?id=4nnctnme-0040");
}

#[test]
fn a_long_title_is_cut_short_and_never_ends_in_a_dash() {
    let mut state = State::default();
    let title = format!("{} y lo que sigue", "palabra ".repeat(6).trim_end());
    let doc = kept(&mut state, ID, Some(&title), None, None);

    let said = address(&state, doc);
    let name = said
        .strip_prefix("tisty://doc/")
        .and_then(|rest| rest.strip_suffix("?id=4nnctnme-0039"))
        .unwrap();

    assert_eq!(name, "palabra-palabra-palabra-palabra-palabra-palabra");
    assert_eq!(doc_asked(&said), Some(ID));
}

#[test]
fn the_id_opens_the_document_whatever_the_names_say() {
    for said in [
        "tisty://doc?id=4nnctnme-0039",
        "tisty://doc/?id=4nnctnme-0039",
        "tisty://doc/renamed/and/moved/since?id=4nnctnme-0039",
        "TISTY://DOC/Personal?id=4nnctnme-0039",
        "  tisty://doc/personal?id=4nnctnme-0039\n",
    ] {
        assert_eq!(doc_asked(said), Some(ID), "{said}");
    }
}

#[test]
fn an_address_carries_nothing_but_the_id_of_a_document() {
    for said in [
        "",
        ID,
        "tisty://doc",
        "tisty://doc/personal/proyectos",
        "tisty://doc?id=",
        "tisty://doc?id=validacion de datos",
        "tisty://doc?id=../../secrets",
        "tisty://doc?id=4nnctnme-0039&folder=personal",
        "tisty://doc?folder=personal&id=4nnctnme-0039",
        "tisty://doc/a?id=4nnctnme-0039#more",
        "tisty://doc/a#b?id=4nnctnme-0039",
        "tisty://doc/a?b?id=4nnctnme-0039",
        "tisty://doc?id=4nnctnme-0039%20",
        "tisty://docs?id=4nnctnme-0039",
        "tisty://task?id=4nnctnme-0039",
        "tisty://open?path=4nnctnme-0039",
        "https://doc?id=4nnctnme-0039",
        "日本語のアドレス?id=4nnctnme-0039",
    ] {
        assert_eq!(doc_asked(said), None, "{said}");
    }
}
