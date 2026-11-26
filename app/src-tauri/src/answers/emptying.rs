use tisty_core::model::{DocId, FolderId};
use tisty_core::{Op, State};

use crate::{Answer, Refusal, Session, folder_open};

/// What a deleted folder does with what it holds: hand it up a level, hand it to another folder, or let it go too.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Contents {
    Up,
    Into(Option<FolderId>),
    Dropped,
}

impl Contents {
    pub(crate) fn asked(said: Option<&str>, into: Option<FolderId>) -> Answer<Self> {
        match said.unwrap_or("up") {
            "up" => Ok(Contents::Up),
            "into" => Ok(Contents::Into(into)),
            "drop" => Ok(Contents::Dropped),
            _ => Err(Refusal::of("internal")),
        }
    }
}

fn docs_in(state: &State, folder: FolderId) -> Vec<DocId> {
    let mut all: Vec<&tisty_core::model::Kept> = state
        .docs
        .values()
        .filter(|one| one.folder == Some(folder) && one.page_of.is_none())
        .collect();
    all.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));
    all.into_iter().map(|one| one.id).collect()
}

fn folders_in(state: &State, folder: FolderId) -> Vec<FolderId> {
    state
        .under(Some(folder))
        .into_iter()
        .map(|one| one.id)
        .collect()
}

pub(crate) fn emptied(session: &mut Session, id: FolderId, contents: Contents) -> Answer<()> {
    let holds =
        !docs_in(&session.state, id).is_empty() || !folders_in(&session.state, id).is_empty();
    // A folder's name inside AGENTS says whose files it holds, so they leave it only through Agents' own moves.
    if holds && tisty_core::agents::holds_folder(&session.state, id) {
        return Err(Refusal::of("agentsFolderHolds"));
    }
    match contents {
        Contents::Up => {
            let up = session.state.folders.get(&id).and_then(|one| one.parent);
            handed(session, id, up)
        }
        Contents::Into(into) => {
            fits(session, id, into)?;
            handed(session, id, into)
        }
        Contents::Dropped => dropped(session, id),
    }
}

fn fits(session: &Session, id: FolderId, into: Option<FolderId>) -> Answer<()> {
    let Some(at) = into else {
        return Ok(());
    };
    let state = &session.state;
    if !state.folders.contains_key(&at) {
        return Err(Refusal::of("noSuchFolder"));
    }
    folder_open(state, at, true)?;
    if tisty_core::agents::holds_folder(state, at) {
        return Err(Refusal::of("agentsHoldOnlyRules"));
    }
    if at == id || state.would_swallow(id, at) {
        return Err(Refusal::of("intoItself"));
    }
    let deepest = folders_in(state, id)
        .into_iter()
        .map(|one| state.tall_under(one))
        .max()
        .unwrap_or(0);
    if state.depth(Some(at)) + deepest > tisty_core::model::DEEPEST {
        return Err(Refusal::of("tooDeep"));
    }
    Ok(())
}

// One move at a time, each placed against the state the one before it left.
fn handed(session: &mut Session, id: FolderId, to: Option<FolderId>) -> Answer<()> {
    for folder in folders_in(&session.state, id) {
        let ops = super::shelves::beside_folders(&session.state, folder, to, None);
        super::shelves::stays_hooked(&session.state, &ops)?;
        session.commit_all(ops)?;
    }
    let mut went = Vec::new();
    for doc in docs_in(&session.state, id) {
        let mut ops = super::papers::beside_docs(&session.state, doc, to, None);
        ops.extend(super::tongues::following(&session.state, doc, to));
        session.commit_all(ops)?;
        went.extend(
            std::iter::once(doc)
                .chain(session.state.kin(doc))
                .filter_map(|one| session.state.docs.get(&one))
                .map(|one| one.file.clone()),
        );
    }
    super::agents_homes::moved_here(session, &went)
}

// Nothing goes unless all of it can: a locked document stops the whole folder before anything is lost.
fn dropped(session: &mut Session, id: FolderId) -> Answer<()> {
    let mut folders = vec![id];
    let mut at = 0;
    while at < folders.len() {
        folders.extend(folders_in(&session.state, folders[at]));
        at += 1;
    }
    let docs: Vec<DocId> = folders
        .iter()
        .flat_map(|one| docs_in(&session.state, *one))
        .collect();
    for doc in &docs {
        session.droppable(*doc)?;
    }
    // A document's other languages go with it, so one already gone is not asked for again.
    for doc in docs {
        if session.state.docs.contains_key(&doc) {
            session.drop_doc(&doc.to_string())?;
        }
    }
    for folder in folders.into_iter().skip(1).rev() {
        let op = Op::FolderDelete { id: folder };
        super::shelves::stays_hooked(&session.state, std::slice::from_ref(&op))?;
        session.commit(op)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "emptying_test.rs"]
mod tests;
