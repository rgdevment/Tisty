use std::sync::Mutex;

use tisty_core::witness::channel;

use super::papers::{book_of, named_anew, unwritten, writable};
use crate::{Answer, Session, blamed, caught, held, signing};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Woven {
    body: String,
    print: String,
    title: String,
}

/// `None` when the two touched the same place, which only the person can choose between.
#[tauri::command(async)]
pub fn doc_merge(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    base: String,
    mine: String,
    turn: Option<super::turns::Turn>,
) -> Answer<Option<Woven>> {
    let file = id.clone();
    caught("doc_merge", || {
        super::turns::in_turn(
            &mut held(&session),
            &file,
            turn,
            |session| merged(session, id, &base, &mine),
            Option::is_some,
        )
    })?
}

pub(crate) fn merged(
    session: &mut Session,
    id: String,
    base: &str,
    mine: &str,
) -> Answer<Option<Woven>> {
    writable(&session.state, &id)?;
    let root = session.paths.docs();
    let theirs = tisty_core::docs::read(&root, &id)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be read", e))?;
    let Some(whole) = tisty_core::merge::merged(base, mine, &theirs) else {
        return Ok(None);
    };
    // The merge joins lines with `\n`; a body kept with `\r\n` stays so, or every line would read as changed.
    let whole = match theirs.contains("\r\n") && !theirs.replace("\r\n", "").contains('\n') {
        true => whole.replace("\r\n", "\n").replace('\n', "\r\n"),
        false => whole,
    };
    let seen = tisty_core::docs::print_of_text(&theirs);
    if !tisty_core::docs::write_over(&root, &session.writer(), &id, &whole, &seen)
        .map_err(unwritten)?
    {
        return Ok(None);
    }
    let body = tisty_core::docs::settled(&whole);
    session.mind_body(&id, &body);
    session.corpus.forget(&id);
    let title = tisty_core::docs::titled(&body);
    let book = book_of(session, &id, &title);
    let hand = signing(&session.state);
    session.retell_over(&id, &body, hand, Some(&seen));
    named_anew(session, book, &id, &title);
    let print = tisty_core::docs::print_of_text(&body);
    Ok(Some(Woven { body, print, title }))
}
