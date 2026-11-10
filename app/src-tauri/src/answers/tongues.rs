use std::collections::BTreeMap;
use std::sync::Mutex;

use tisty_core::model::{DocId, FolderId, Kept};
use tisty_core::{Op, State};

use crate::{Answer, Refusal, Session, blamed, doc_out, held};
use tisty_core::witness::channel;

/// A copy of a document and its pages in another language, joined to it, ready to translate.
#[tauri::command]
pub fn doc_translate(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    lang: String,
    from: Option<String>,
) -> Answer<tisty_core::docs::Doc> {
    translated(&mut held(&session), &id, &lang, from)
}

pub fn translated(
    session: &mut Session,
    id: &str,
    lang: &str,
    from: Option<String>,
) -> Answer<tisty_core::docs::Doc> {
    let doc: DocId = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let lang = language(lang)?;
    let kept = joinable(&session.state, doc)?;
    let (said, unwritten) = said_of(&session.state, &kept, from)?;
    if said == lang || session.state.tongue_taken(doc, &lang) {
        return Err(Refusal::of("tongueTaken"));
    }
    if unwritten {
        write_lang(session, &kept, &said)?;
    }
    let anchor = session.state.anchor_of(&kept);
    // Whoever edits through a link may add a language, and it is a document of their own (150).
    let editing = session
        .state
        .docs
        .get(&anchor)
        .and_then(|one| {
            tisty_sharing::remembered(&session.state)
                .get(&one.file)
                .cloned()
        })
        .is_some_and(|one| one.edited_through_a_link());
    let made = session.copy_doc_as(id, editing, &|body| {
        let body = tisty_markdown::front::without_field(body, tisty_core::tongues::TRANSLATION_KEY);
        tisty_markdown::front::with_field(&body, "lang", &lang)
    })?;
    let twin = session
        .state
        .docs
        .values()
        .find(|one| one.file == made.id)
        .map(|one| one.id)
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;
    session.commit(Op::DocTongue {
        id: twin,
        d: tisty_core::event::Tongue { of: Some(anchor) },
    })?;
    Ok(made)
}

/// Joins a document to the languages of another, or with no `of` takes it out of its own.
#[tauri::command]
pub fn doc_tongue(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    of: Option<String>,
    lang: Option<String>,
    of_lang: Option<String>,
) -> Answer<()> {
    tongued(&mut held(&session), &id, of, lang, of_lang)
}

pub fn tongued(
    session: &mut Session,
    id: &str,
    of: Option<String>,
    lang: Option<String>,
    of_lang: Option<String>,
) -> Answer<()> {
    let doc: DocId = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let kept = joinable(&session.state, doc)?;
    let Some(of) = of else {
        let ops = session.state.parting(kept.id);
        session.commit_all(ops)?;
        return Ok(());
    };
    let of: DocId = of.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let up = joinable(&session.state, of)?;
    if session.state.anchor_of(&up) == session.state.anchor_of(&kept) {
        return Ok(());
    }
    // Everything is judged before a word is written, so a refusal leaves both as they were.
    let (theirs, theirs_unwritten) = said_of(&session.state, &up, of_lang)?;
    let (said, unwritten) = said_of(&session.state, &kept, lang)?;
    if said == theirs || session.state.tongue_taken(of, &said) {
        return Err(Refusal::of("tongueTaken"));
    }
    if theirs_unwritten {
        write_lang(session, &up, &theirs)?;
    }
    if unwritten {
        write_lang(session, &kept, &said)?;
    }
    let ops = session.state.joining(doc, of);
    session.commit_all(ops)?;
    Ok(())
}

/// Paper, sheets and their turn are how a document is laid out, so every language shares them.
#[tauri::command]
pub fn doc_dress(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    paper: Option<String>,
    sheets: bool,
    landscape: bool,
) -> Answer<()> {
    dressed(&mut held(&session), &id, paper, sheets, landscape)
}

pub fn dressed(
    session: &mut Session,
    id: &str,
    paper: Option<String>,
    sheets: bool,
    landscape: bool,
) -> Answer<()> {
    let doc: DocId = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let laid = [
        ("paper", paper.filter(|one| one != "a4")),
        ("sheets", sheets.then(|| "on".to_string())),
        (
            "orientation",
            (sheets && landscape).then(|| "landscape".to_string()),
        ),
    ];
    let others: Vec<(DocId, String)> = session
        .state
        .tongues_of(doc)
        .into_iter()
        .filter(|one| one.id != doc)
        .map(|one| (one.id, one.file.clone()))
        .collect();
    for (other, file) in others {
        if session.state.shut(other) || doc_out(&session.state, other).is_err() {
            continue;
        }
        let written =
            tisty_core::docs::amend(&session.paths.docs(), session.paths.data(), &file, |was| {
                let now = laid
                    .iter()
                    .fold(was.to_string(), |body, (key, value)| match value {
                        Some(value) => tisty_markdown::front::with_field(&body, key, value),
                        None => tisty_markdown::front::without_field(&body, key),
                    });
                (now != was).then_some(now)
            })
            .map_err(|e| {
                blamed(
                    channel::WINDOW,
                    "a language's paper could not be written",
                    e,
                )
            })?;
        if let Some(body) = written {
            session.retell(&file, &body, None);
        }
    }
    Ok(())
}

/// A document and the languages beside it, which the person sees as one row and acts on as one.
pub fn with_kin(state: &State, id: DocId) -> impl Iterator<Item = DocId> {
    std::iter::once(id).chain(state.kin(id))
}

/// Where each document stands among its languages: the anchor they gather around, and its place.
pub fn standing(state: &State) -> BTreeMap<DocId, (String, usize)> {
    state
        .families()
        .into_iter()
        .flat_map(|(anchor, tongues)| {
            tongues
                .into_iter()
                .enumerate()
                .map(move |(at, one)| (one.id, (anchor.to_string(), at)))
        })
        .collect()
}

/// Each group found once: the first of its languages the words turned up in stands for it.
pub fn once_each<T>(state: &State, found: Vec<T>, file: impl Fn(&T) -> &str) -> Vec<T> {
    let anchors: BTreeMap<&str, DocId> = state
        .families()
        .into_iter()
        .flat_map(|(anchor, tongues)| {
            tongues
                .into_iter()
                .map(move |one| (one.file.as_str(), anchor))
        })
        .collect();
    let mut seen = std::collections::BTreeSet::new();
    found
        .into_iter()
        .filter(|one| {
            anchors
                .get(file(one))
                .is_none_or(|anchor| seen.insert(*anchor))
        })
        .collect()
}

/// The languages beside a document that leaves its folder, which go along with it.
pub fn following(state: &State, id: DocId, folder: Option<FolderId>) -> Vec<Op> {
    let leaving = state.docs.get(&id).is_some_and(|one| one.folder != folder);
    if !leaving {
        return Vec::new();
    }
    state
        .kin(id)
        .into_iter()
        .map(|id| Op::DocMove {
            id,
            d: tisty_core::event::Filed {
                folder: Some(folder),
                page_of: None,
                order: None,
            },
        })
        .collect()
}

fn language(said: &str) -> Answer<String> {
    tisty_markdown::front::tagged(said).ok_or_else(|| Refusal::of("notALanguage"))
}

// A language is a document of its own: never a page, a task's words, or something put away.
fn joinable(state: &State, id: DocId) -> Answer<Kept> {
    let kept = state
        .docs
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;
    if kept.page_of.is_some() || kept.task_of.is_some() {
        return Err(Refusal::of("pageHasNoTongue"));
    }
    if state.held_away(&kept) {
        return Err(Refusal::of("awayStaysAway"));
    }
    doc_out(state, id)?;
    Ok(kept)
}

// The language a document says, or the one named for it and whether it still has to be written.
fn said_of(state: &State, kept: &Kept, named: Option<String>) -> Answer<(String, bool)> {
    let named = named.map(|one| language(&one)).transpose()?;
    match (named, kept.lang.clone()) {
        (None, Some(said)) => Ok((said, false)),
        (None, None) => Err(Refusal::of("tongueUnsaid")),
        (Some(named), said) if said.as_deref() == Some(named.as_str()) => Ok((named, false)),
        (Some(_), _) if state.shut(kept.id) => Err(Refusal::of("documentLocked")),
        (Some(named), _) => Ok((named, true)),
    }
}

fn write_lang(session: &mut Session, kept: &Kept, lang: &str) -> Answer<()> {
    let written = tisty_core::docs::amend(
        &session.paths.docs(),
        session.paths.data(),
        &kept.file,
        |was| Some(tisty_markdown::front::with_field(was, "lang", lang)),
    )
    .map_err(|e| blamed(channel::WINDOW, "a language could not be written", e))?;
    let body = written.ok_or_else(|| Refusal::of("noSuchDoc"))?;
    let hand = crate::signing(&session.state);
    session.retell(&kept.file, &body, hand);
    Ok(())
}

#[cfg(test)]
#[path = "tongues_test.rs"]
mod tests;
