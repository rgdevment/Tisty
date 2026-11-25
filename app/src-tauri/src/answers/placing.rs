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
        Some(book) => places.may_hang(state, page, book).map_err(refused),
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
