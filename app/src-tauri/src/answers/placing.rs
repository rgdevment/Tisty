use std::sync::Mutex;

use tisty_core::model::{DocId, FolderId};
use tisty_core::places::{Making, PLACES, Place, Places, Ruled, Rules};

use crate::{Answer, Refusal, Session, held};

fn refused(ruled: Ruled) -> Refusal {
    Refusal::of(ruled.0)
}

fn places(session: &Session) -> Places {
    Places::read(&session.paths.docs(), &session.state)
}

/// Hanging a page from a book, or letting one go when `book` is none: Ideas and the diary keep theirs.
pub(crate) fn may_hang(session: &Session, page: DocId, book: Option<DocId>) -> Answer<()> {
    let state = &session.state;
    let (Some(page), places) = (state.docs.get(&page), places(session)) else {
        return Err(Refusal::of("noSuchDoc"));
    };
    match book.and_then(|up| state.docs.get(&up)) {
        Some(book) => {
            places.may_hang(state, page, book).map_err(refused)?;
            lodged(session, &places, page.id, places.of_doc(state, book))
        }
        None => places.may_file(state, page).map_err(refused),
    }
}

pub(crate) fn may_file(session: &Session, doc: DocId) -> Answer<()> {
    let state = &session.state;
    let Some(kept) = state.docs.get(&doc) else {
        return Err(Refusal::of("noSuchDoc"));
    };
    places(session).may_file(state, kept).map_err(refused)
}

pub(crate) fn may_make(
    session: &Session,
    into: Option<FolderId>,
    book: Option<DocId>,
    what: Making,
) -> Answer<()> {
    let places = places(session);
    match book.and_then(|up| session.state.docs.get(&up)) {
        Some(book) => places
            .of_doc(&session.state, book)
            .takes_pages()
            .map_err(refused),
        None => places.may_make(into, what).map_err(refused),
    }
}

/// Only the sheet Ideas writes to stays: one archived by an older version, or a second one, may still go.
pub(crate) fn may_go(session: &Session, doc: DocId) -> Answer<()> {
    // Finding the sheet reads files, so a document that never said it was one, nor was last found, goes first.
    let known = session.state.docs.get(&doc).is_some_and(|kept| {
        kept.brainstorm || session.brainstorm.as_deref() == Some(kept.file.as_str())
    });
    if !known {
        return Ok(());
    }
    let sheet = tisty_core::brainstorm::find(
        &session.paths.docs(),
        &session.state,
        session.brainstorm.as_deref(),
    );
    match sheet.is_some_and(|(one, _)| one.id == doc) {
        true => Place::Ideas.goes().map_err(refused),
        false => Ok(()),
    }
}

pub(crate) fn may_copy(session: &Session, doc: DocId) -> Answer<()> {
    let state = &session.state;
    let Some(kept) = state.docs.get(&doc) else {
        return Err(Refusal::of("noSuchDoc"));
    };
    places(session).may_copy(state, kept).map_err(refused)
}

/// AGENTS makes no folder of its own, so only a folder that is already one of its own moves inside, and none leaves.
pub(crate) fn may_shelve(
    session: &Session,
    folder: FolderId,
    into: Option<FolderId>,
) -> Answer<()> {
    places(session)
        .may_shelve(&session.state, folder, into)
        .map_err(refused)
}

/// Filing `doc` into `into`: a plain document does not slip into AGENTS, where every file is an agent's.
pub(crate) fn may_lodge(session: &Session, doc: DocId, into: Option<FolderId>) -> Answer<()> {
    let places = places(session);
    lodged(session, &places, doc, places.of_folder(into))
}

fn lodged(session: &Session, places: &Places, doc: DocId, into: Place) -> Answer<()> {
    let state = &session.state;
    let Some(kept) = state.docs.get(&doc) else {
        return Err(Refusal::of("noSuchDoc"));
    };
    let agents_own = into == Place::Agents
        && tisty_core::docs::read(&session.paths.docs(), &kept.file)
            .is_ok_and(|body| tisty_core::agents::Card::of(&body).is_some());
    places
        .may_lodge(state, kept, into, agents_own)
        .map_err(refused)
}

/// A diagram is a document to the store, so the window says which it is making and AGENTS refuses it here.
#[tauri::command]
pub fn doc_new(
    session: tauri::State<'_, Mutex<Session>>,
    folder: Option<String>,
    page_of: Option<String>,
    existing: Option<bool>,
    making: Option<Making>,
) -> Answer<tisty_core::docs::Doc> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let page_of = page_of
        .map(|up| up.parse().map_err(|_| Refusal::of("noSuchDoc")))
        .transpose()?;
    let mut session = held(&session);
    if let Some(what) = making.filter(|what| *what != Making::Doc) {
        may_make(&session, folder, page_of, what)?;
    }
    super::papers::doc_made(&mut session, folder, page_of, existing.unwrap_or(false))
}

#[derive(serde::Serialize)]
pub struct Ruling {
    plain: Rules,
    ideas: Rules,
    diary: Rules,
    agents: Rules,
}

#[tauri::command]
pub fn places_rules() -> Ruling {
    let [plain, ideas, diary, agents] = PLACES.map(Place::rules);
    Ruling {
        plain,
        ideas,
        diary,
        agents,
    }
}

#[derive(serde::Serialize)]
pub struct Placed {
    place: Place,
    /// An agent's own file is read by that agent as it stands, so no Tisty card goes in it.
    cards: bool,
}

#[tauri::command(async)]
pub fn doc_place(session: tauri::State<'_, Mutex<Session>>, file: String) -> Answer<Placed> {
    let session = held(&session);
    let Some(kept) = session.state.docs.values().find(|one| one.file == file) else {
        return Err(Refusal::of("noSuchDoc"));
    };
    let place = places(&session).of_doc(&session.state, kept);
    let agents_own = place == Place::Agents
        && tisty_core::docs::read(&session.paths.docs(), &kept.file)
            .is_ok_and(|body| tisty_core::agents::Card::of(&body).is_some());
    Ok(Placed {
        place,
        cards: !agents_own,
    })
}

#[cfg(test)]
#[path = "placing_test.rs"]
mod tests;

#[cfg(test)]
#[path = "places_matrix_test.rs"]
mod matrix;
