use std::collections::BTreeMap;
use std::sync::Mutex;

use tisty_core::model::{DocId, FolderId, Kept};
use tisty_core::{Op, State};

use crate::{Answer, Refusal, Session, blamed, doc_out, held};
use tisty_core::witness::channel;

/// A copy of the document in another language, joined to its languages: its words copied whole
/// so it can be translated section by section, its pages with it.
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
    let said = said_in(session, &kept, from)?;
    if said == lang || taken(&session.state, doc, &lang) {
        return Err(Refusal::of("tongueTaken"));
    }
    let anchor = session.state.anchor_of(&kept);
    let made = session.copy_doc_as(id, &|body| {
        tisty_markdown::front::with_field(body, "lang", &lang)
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
        let ops = parted(&session.state, &kept);
        session.commit_all(ops)?;
        return Ok(());
    };
    let of: DocId = of.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let up = joinable(&session.state, of)?;
    if session.state.anchor_of(&up) == session.state.anchor_of(&kept) {
        return Ok(());
    }
    said_in(session, &up, of_lang)?;
    let said = said_in(session, &kept, lang)?;
    if taken(&session.state, of, &said) {
        return Err(Refusal::of("tongueTaken"));
    }
    let anchor = session.state.anchor_of(&up);
    // The group stands in one folder, so what joins it goes where it is, with its own languages.
    let mut ops: Vec<Op> = std::iter::once(doc)
        .chain(session.state.kin(doc))
        .filter(|one| {
            session
                .state
                .docs
                .get(one)
                .is_some_and(|one| one.folder != up.folder)
        })
        .map(|id| Op::DocMove {
            id,
            d: tisty_core::event::Filed {
                folder: Some(up.folder),
                page_of: None,
                order: None,
            },
        })
        .collect();
    ops.push(Op::DocTongue {
        id: doc,
        d: tisty_core::event::Tongue { of: Some(anchor) },
    });
    session.commit_all(ops)?;
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

fn taken(state: &State, id: DocId, lang: &str) -> bool {
    let tongues = state.tongues_of(id);
    match tongues.is_empty() {
        true => state.docs.get(&id).and_then(|one| one.lang.as_deref()) == Some(lang),
        false => tongues.iter().any(|one| one.lang.as_deref() == Some(lang)),
    }
}

// The language a document says, written into it first when the person named one it did not say.
fn said_in(session: &mut Session, kept: &Kept, named: Option<String>) -> Answer<String> {
    let named = named.map(|one| language(&one)).transpose()?;
    match (named, kept.lang.clone()) {
        (None, Some(said)) => Ok(said),
        (None, None) => Err(Refusal::of("tongueUnsaid")),
        (Some(named), said) if said.as_deref() == Some(named.as_str()) => Ok(named),
        (Some(named), _) => {
            if session.state.shut(kept.id) {
                return Err(Refusal::of("documentLocked"));
            }
            let written = tisty_core::docs::amend(
                &session.paths.docs(),
                session.paths.data(),
                &kept.file,
                |was| Some(tisty_markdown::front::with_field(was, "lang", &named)),
            )
            .map_err(|e| blamed(channel::WINDOW, "a language could not be written", e))?;
            let body = written.ok_or_else(|| Refusal::of("noSuchDoc"))?;
            let hand = crate::signing(&session.state);
            session.retell(&kept.file, &body, hand);
            Ok(named)
        }
    }
}

// Out of its languages; an anchor hands the others to the first of them so they stay together.
fn parted(state: &State, kept: &Kept) -> Vec<Op> {
    let out = |id: DocId, of: Option<DocId>| Op::DocTongue {
        id,
        d: tisty_core::event::Tongue { of },
    };
    if kept.translation_of.is_some() {
        return vec![out(kept.id, None)];
    }
    let mut others = state
        .papers()
        .filter(|one| one.translation_of == Some(kept.id))
        .collect::<Vec<_>>();
    others.sort_by_key(|one| (one.joined.or(one.made), one.id));
    let Some((next, rest)) = others.split_first() else {
        return Vec::new();
    };
    std::iter::once(out(next.id, None))
        .chain(rest.iter().map(|one| out(one.id, Some(next.id))))
        .collect()
}

#[cfg(test)]
#[path = "tongues_test.rs"]
mod tests;
