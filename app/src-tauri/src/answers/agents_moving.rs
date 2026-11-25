use std::collections::BTreeSet;
use std::sync::Mutex;

use tisty_core::State;
use tisty_core::agents::{self, Card, GLOBAL, Gate, Kind, Ledger, Seen};
use tisty_core::event::DeviceId;
use tisty_core::model::{DocId, FolderId, Kept};

use super::agents_homes::{approved, earlier_elsewhere, kept_of, read, under};
use super::agents_sync::{docs_in, gated, here, root_of, tree};
use crate::wiring::agents::Homes;
use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "is", rename_all = "camelCase")]
pub enum Scope {
    Everyone,
    Mine,
    Other { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Here {
    Writes,
    Keeps,
    Asks,
    Stops,
    Unapproved,
    Unread,
    Untouched,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Going {
    pub title: String,
    pub file: String,
    pub here: Here,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Moving {
    pub from: Scope,
    pub to: Scope,
    pub docs: Vec<Going>,
    pub older: Vec<String>,
}

#[derive(Clone, Copy)]
pub(crate) struct Around<'a> {
    pub(crate) ledger: Option<&'a Ledger>,
    pub(crate) homes: Option<&'a Homes>,
}

#[tauri::command(async)]
pub fn agents_moving(
    session: tauri::State<'_, Mutex<Session>>,
    kind: String,
    id: String,
    to: Option<String>,
) -> Answer<Option<Moving>> {
    let session = held(&session);
    let to: Option<FolderId> = to
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let ledger = Ledger::mine(session.paths.data(), &here());
    let homes = Homes::here();
    let around = Around {
        ledger: ledger.as_ref(),
        homes: homes.as_ref(),
    };
    match kind.as_str() {
        "doc" => {
            let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
            Ok(doc_moving(&session, around, id, to))
        }
        "folder" => {
            let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
            Ok(folder_moving(&session, around, id, to))
        }
        _ => Err(Refusal::of("noSuchDoc")),
    }
}

pub(crate) fn doc_moving(
    session: &Session,
    around: Around<'_>,
    id: DocId,
    to: Option<FolderId>,
) -> Option<Moving> {
    let state = &session.state;
    let kept = state
        .docs
        .get(&id)
        .filter(|kept| agents::holds(state, kept))?;
    let into = to.filter(|at| agents::holds_folder(state, *at))?;
    let files: Vec<String> = std::iter::once(id)
        .chain(state.kin(id))
        .filter_map(|one| state.docs.get(&one))
        .map(|one| one.file.clone())
        .collect();
    let from = agents::computer_of(state, kept.folder);
    let goes_to = agents::computer_of(state, Some(into));
    let together: BTreeSet<String> = files.iter().cloned().collect();
    moving(session, around, (from, goes_to), &files, &together)
}

pub(crate) fn folder_moving(
    session: &Session,
    around: Around<'_>,
    id: FolderId,
    parent: Option<FolderId>,
) -> Option<Moving> {
    let state = &session.state;
    let (from, to) = folder_scopes(state, id, parent)?;
    moving(
        session,
        around,
        (from, to),
        &under(state, id),
        &BTreeSet::new(),
    )
}

fn folder_scopes(
    state: &State,
    id: FolderId,
    parent: Option<FolderId>,
) -> Option<(Option<DeviceId>, Option<DeviceId>)> {
    let folder = state.folders.get(&id)?;
    let parent = parent.filter(|at| agents::holds_folder(state, *at))?;
    if !agents::holds_folder(state, id) {
        return None;
    }
    let to = match root_of(state) == Some(parent) {
        true => folder.computer.clone(),
        false => agents::computer_of(state, Some(parent)),
    };
    Some((agents::computer_of(state, Some(id)), to))
}

fn moving(
    session: &Session,
    around: Around<'_>,
    (from, to): (Option<DeviceId>, Option<DeviceId>),
    files: &[String],
    together: &BTreeSet<String>,
) -> Option<Moving> {
    if from == to {
        return None;
    }
    let me = session.store.device();
    let syncing = around.ledger.filter(|one| one.consented());
    let gate = syncing.map(|one| gated(session, one));
    let lands_here = to.as_ref().is_none_or(|one| one == me);
    let docs: Vec<Going> = files
        .iter()
        .filter_map(|file| {
            let kept = kept_of(session, file)?;
            let body = read(session, file).ok()?;
            let card = Card::of(&body)?;
            if foreign_memory(&card, kept, from.as_ref()) {
                return None;
            }
            let here = match (syncing.zip(gate.as_ref()), lands_here) {
                (None, _) => Here::Untouched,
                (Some((_, gate)), false) if gate.seen(file) == Some(Seen::Admitted) => Here::Stops,
                (Some(_), false) => Here::Untouched,
                (Some(_), true) if !approved(kept, &body) => Here::Unapproved,
                (Some((ledger, _)), true) if !read_here(around.homes, ledger, &card) => {
                    Here::Unread
                }
                (Some((_, gate)), true) if rivalled(gate, file, together, &card) => Here::Asks,
                (Some((_, gate)), true) if gate.seen(file) == Some(Seen::Admitted) => Here::Keeps,
                (Some(_), true) => Here::Writes,
            };
            Some(Going {
                title: kept.title.clone().unwrap_or_else(|| card.name.clone()),
                file: card.name.clone(),
                here,
            })
        })
        .collect();
    (!docs.is_empty()).then(|| Moving {
        from: scope(session, from.as_ref()),
        to: scope(session, to.as_ref()),
        docs,
        older: older(session),
    })
}

// What an agent remembers on one computer never becomes another's, so moving it changes nothing.
fn foreign_memory(card: &Card, kept: &Kept, from: Option<&DeviceId>) -> bool {
    card.kind == Kind::Memory && from.is_some() && from != kept.made_by.as_ref()
}

// The same standing set the move itself weighs: what is declined here stands against nothing, and a document's own versions travel with it.
fn rivalled(gate: &Gate, file: &str, together: &BTreeSet<String>, card: &Card) -> bool {
    gate.for_here()
        .filter(|(one, _)| *one != file && !together.contains(*one))
        .filter(|(one, _)| gate.seen(one) != Some(Seen::DeclinedHere))
        .any(|(_, other)| same_place(card, other))
}

fn read_here(homes: Option<&Homes>, ledger: &Ledger, card: &Card) -> bool {
    let Some(homes) = homes else {
        return false;
    };
    let mut readers = homes
        .detected()
        .into_iter()
        .filter(|one| ledger.syncs(one.id));
    match card.level == GLOBAL {
        true => readers.next().is_some(),
        false => readers.any(|one| one.id == card.level),
    }
}

fn same_place(one: &Card, other: &Card) -> bool {
    if one.address() == other.address() {
        return true;
    }
    let skills = one.kind == Kind::Skill && other.kind == Kind::Skill;
    let one_global = (one.level == GLOBAL) != (other.level == GLOBAL);
    skills && one_global && skill_of(&one.name) == skill_of(&other.name)
}

fn skill_of(name: &str) -> &str {
    name.split('/').next().unwrap_or(name)
}

pub(crate) fn scope(session: &Session, computer: Option<&DeviceId>) -> Scope {
    match computer {
        None => Scope::Everyone,
        Some(one) if one == session.store.device() => Scope::Mine,
        Some(one) => Scope::Other {
            name: session.state.called(one),
        },
    }
}

fn older(session: &Session) -> Vec<String> {
    let names: BTreeSet<String> = earlier_elsewhere(session)
        .iter()
        .map(|one| session.state.called(one))
        .collect();
    names.into_iter().collect()
}

pub(crate) fn folder_resolved(state: &State, id: FolderId, parent: Option<FolderId>) -> Answer<()> {
    let Some((from, to)) = folder_scopes(state, id, parent) else {
        return Ok(());
    };
    let waits = from != to
        && docs_in(state, &tree(state, id))
            .iter()
            .any(|kept| state.proposals.contains_key(&kept.id));
    match waits {
        true => Err(Refusal::of("agentProposalInFolder")),
        false => Ok(()),
    }
}

#[cfg(test)]
#[path = "agents_moving_test.rs"]
mod tests;
