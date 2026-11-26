use std::sync::Mutex;

use tisty_core::witness::channel;

use super::papers::{book_of, named_anew, unwritten, writable};
use crate::{Answer, Refusal, Session, blamed, caught, held, signing};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Woven {
    body: String,
    print: String,
    title: String,
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum Merge {
    Woven(Woven),
    /// How many places both touched, or `None` when standing by mine would put the whole page back.
    Torn {
        torn: Option<usize>,
        /// Both changed the properties, and standing by mine keeps mine whole.
        front: bool,
    },
}

const WEAVES_AT_MOST: usize = 3;

/// Standing by mine writes over the body just read, so a writer in between is woven, never trodden on.
#[tauri::command(async)]
pub fn doc_merge(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    base: Option<String>,
    mine: String,
    prefer: Option<String>,
    turn: Option<super::turns::Turn>,
) -> Answer<Merge> {
    let file = id.clone();
    let mine_stands = prefer.as_deref() == Some("mine");
    caught("doc_merge", || {
        super::turns::in_turn(
            &mut held(&session),
            &file,
            turn,
            |session| merged(session, id, base.as_deref(), &mine, mine_stands),
            |done| matches!(done, Merge::Woven(_)),
        )
    })?
}

pub(crate) fn merged(
    session: &mut Session,
    id: String,
    base: Option<&str>,
    mine: &str,
    mine_stands: bool,
) -> Answer<Merge> {
    writable(&session.state, &id)?;
    super::agents_homes::read_here(session, &id)?;
    let root = session.paths.docs();
    for _ in 0..WEAVES_AT_MOST {
        let theirs = tisty_core::docs::read(&root, &id).map_err(|e| match e.held_open() {
            true => Refusal::of("documentHeldOpen"),
            false => blamed(channel::WINDOW, "a document could not be read", e),
        })?;
        let preferred =
            || base.and_then(|base| tisty_core::merge::merged_preferring(base, mine, &theirs));
        let whole = match base.and_then(|base| tisty_core::merge::merged(base, mine, &theirs)) {
            Some(whole) => whole,
            None if !mine_stands => {
                let preferred = preferred();
                return Ok(Merge::Torn {
                    torn: preferred.as_ref().map(|one| one.torn),
                    front: preferred.is_some_and(|one| one.front_mine),
                });
            }
            None => preferred().map_or_else(|| mine.to_string(), |one| one.whole),
        };
        let whole = ended_as(&theirs, whole);
        let seen = tisty_core::docs::print_of_text(&theirs);
        let landed = match mine_stands {
            true => {
                tisty_core::docs::rewrite(&root, &session.writer(), &id, &whole, &seen)
                    .map_err(unwritten)?
                    != tisty_core::docs::Rewrite::Moved
            }
            false => tisty_core::docs::write_over(&root, &session.writer(), &id, &whole, &seen)
                .map_err(unwritten)?,
        };
        if landed {
            return Ok(Merge::Woven(told_woven(session, &id, &whole, &seen)));
        }
    }
    Err(Refusal::about("documentMoved", id))
}

// The merge joins lines with `\n`; a body kept with `\r\n` stays so, or every line would read as changed.
pub(crate) fn ended_as(theirs: &str, whole: String) -> String {
    match theirs.contains("\r\n") && !theirs.replace("\r\n", "").contains('\n') {
        true => whole.replace("\r\n", "\n").replace('\n', "\r\n"),
        false => whole,
    }
}

fn told_woven(session: &mut Session, id: &str, whole: &str, seen: &str) -> Woven {
    let body = tisty_core::docs::settled(whole);
    session.mind_body(id, &body);
    session.corpus.forget(id);
    let title = tisty_core::docs::titled(&body);
    let book = book_of(session, id, &title);
    let hand = signing(&session.state);
    session.retell_over(id, &body, hand, Some(seen));
    named_anew(session, book, id, &title);
    let print = tisty_core::docs::print_of_text(&body);
    Woven { body, print, title }
}

#[cfg(test)]
#[path = "merge_mine_test.rs"]
mod merge_mine_tests;
