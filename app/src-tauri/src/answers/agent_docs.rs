use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::agents::{self, Card, Ledger, Refused, Standing, Target};

use super::agents_sync::refused;
use crate::wiring::agents::{Homes, Placed, assistant, targets};
use crate::{Answer, Refusal, Session, held};

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum Said {
    Synced,
    Pending,
    Outside { mine: bool, gone: bool },
    Both,
    Off,
    NoConsent,
    Empty,
    Linked,
    Broken { why: String },
    Unreadable { why: String },
}

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    n: u64,
    bytes: u64,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Place {
    assistant: String,
    named: String,
    at: String,
    said: Said,
    /// How many documents the file is made of, so a composed file says so.
    shared: usize,
    changed: Option<String>,
    kept: Vec<Backup>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentDoc {
    kind: &'static str,
    level: String,
    named: Option<String>,
    name: String,
    code: bool,
    /// Sync is off for the whole AGENTS folder, so no single assistant can be turned back on here.
    folder_off: bool,
    /// Another document in AGENTS carries the same card, so this one is never written.
    twin: bool,
    places: Vec<Place>,
}

fn named(id: &str) -> String {
    assistant(id).map_or_else(|| id.to_string(), |one| one.name.to_string())
}

pub(crate) fn said(standing: Standing, piece: usize) -> Said {
    match standing {
        Standing::UpToDate | Standing::Agreed => Said::Synced,
        Standing::Write | Standing::Wrote => Said::Pending,
        Standing::Outside { now, piece: traced } => Said::Outside {
            mine: traced == Some(piece),
            gone: now.is_none(),
        },
        Standing::Both { .. } => Said::Both,
        Standing::Off => Said::Off,
        Standing::NoConsent => Said::NoConsent,
        Standing::Empty => Said::Empty,
        Standing::Linked => Said::Linked,
        Standing::Broken { why, .. } => Said::Broken {
            why: why.to_string(),
        },
        Standing::Unreadable(why) => Said::Unreadable { why },
    }
}

fn changed_at(at: &Path) -> Option<String> {
    let when = std::fs::metadata(at).ok()?.modified().ok()?;
    jiff::Timestamp::try_from(when)
        .ok()
        .map(|one| one.to_string())
}

pub(crate) fn place(data: &Path, ledger: &Ledger, target: &Target, piece: usize) -> Place {
    Place {
        assistant: target.assistant.clone(),
        named: named(&target.assistant),
        at: target.at.to_string_lossy().into_owned(),
        said: said(agents::standing(ledger, target), piece),
        shared: target.pieces.len(),
        changed: changed_at(&target.at),
        kept: agents::kept(data, &target.at)
            .into_iter()
            .map(|one| Backup {
                n: one.n,
                bytes: one.bytes,
            })
            .collect(),
    }
}

/// The files a document is written into, each with where the document stands among its pieces.
pub(crate) fn placed_for(file: &str, all: Vec<Placed>) -> Vec<(Target, usize)> {
    all.into_iter()
        .filter_map(|one| {
            let piece = one.files.iter().position(|named| named == file)?;
            Some((one.target, piece))
        })
        .collect()
}

fn ledger(data: &Path) -> Ledger {
    Ledger::open(data, &tisty_core::machine::here().unwrap_or_default())
}

struct Opened {
    card: Card,
    data: PathBuf,
    twin: bool,
    places: Vec<(Target, usize)>,
}

// Only a document in AGENTS is written anywhere; elsewhere its card is a property like any other.
fn opened(session: &Session, file: &str) -> Answer<Option<Opened>> {
    let bodies = super::agents_sync::bodies(session);
    let Some(card) = bodies
        .iter()
        .find(|(one, _)| one == file)
        .and_then(|(_, body)| Card::of(body))
    else {
        return Ok(None);
    };
    let address = card.address();
    let twins: Vec<&String> = bodies
        .iter()
        .filter(|(one, body)| {
            one != file && Card::of(body).is_some_and(|other| other.address() == address)
        })
        .map(|(one, _)| one)
        .collect();
    let all = match Homes::here() {
        Some(homes) => targets(&homes, &bodies),
        None => Vec::new(),
    };
    let written = |one: &Placed| one.files.iter().any(|named| twins.contains(&named));
    let twin = all.iter().any(written);
    let places = placed_for(file, all);
    Ok(Some(Opened {
        card,
        data: session.paths.data().to_path_buf(),
        twin: twin && places.is_empty(),
        places,
    }))
}

fn one_place(session: &Session, file: &str, at: &str) -> Answer<(PathBuf, Target, usize)> {
    let opened = opened(session, file)?.ok_or_else(|| Refusal::of("agentNoCard"))?;
    let (target, piece) = opened
        .places
        .into_iter()
        .find(|(target, _)| target.at.to_string_lossy() == at)
        .ok_or_else(|| Refusal::of("agentNoPlace"))?;
    Ok((opened.data, target, piece))
}

#[derive(serde::Serialize, Debug)]
#[serde(untagged)]
pub enum Told {
    Card(Option<AgentDoc>),
    Text(Option<String>),
}

/// The card of a document, or, given a file and something to do there, what doing it gives back.
#[tauri::command(async)]
pub fn agent_doc(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    at: Option<String>,
    asked: Option<Asked>,
) -> Answer<Told> {
    match at.zip(asked) {
        Some((at, asked)) => done_at(&session, &file, &at, asked).map(Told::Text),
        None => card(&session, &file).map(Told::Card),
    }
}

fn card(session: &Mutex<Session>, file: &str) -> Answer<Option<AgentDoc>> {
    let Some(opened) = opened(&held(session), file)? else {
        return Ok(None);
    };
    let ledger = ledger(&opened.data);
    let card = opened.card;
    Ok(Some(AgentDoc {
        kind: card.kind.word(),
        named: (card.level != agents::GLOBAL).then(|| named(&card.level)),
        code: card.code(),
        folder_off: ledger.consented() && !ledger.syncs(""),
        twin: opened.twin,
        places: opened
            .places
            .iter()
            .map(|(target, piece)| place(&opened.data, &ledger, target, *piece))
            .collect(),
        level: card.level,
        name: card.name,
    }))
}

#[derive(serde::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "do")]
pub enum Asked {
    /// The document as the file now says it, written in through the core so its history keeps what it said.
    Bring,
    /// What the file holds now, or one of its backups, as text to look at.
    Held {
        backup: Option<u64>,
    },
    Restore {
        backup: u64,
    },
}

pub(crate) fn done(
    data: &Path,
    ledger: &Ledger,
    target: &Target,
    piece: usize,
    asked: Asked,
) -> Answer<Option<String>> {
    match asked {
        Asked::Held { backup: None } if !ledger.consented() => Err(Refused::NoConsent),
        Asked::Bring => agents::bring(ledger, target, piece).map(Some),
        Asked::Held { backup: Some(n) } => {
            agents::read_kept(data, &target.at, n).map(|bytes| Some(text(&bytes)))
        }
        Asked::Held { backup: None } => now(&target.at).map(|bytes| Some(text(&bytes))),
        Asked::Restore { backup } => agents::restore(data, ledger, target, backup).map(|()| None),
    }
    .map_err(refused)
}

fn now(at: &Path) -> Result<Vec<u8>, Refused> {
    let meta = std::fs::symlink_metadata(at)?;
    if meta.file_type().is_symlink() {
        return Err(Refused::Linked);
    }
    if meta.len() > tisty_core::docs::READ_AT_MOST {
        return Err(Refused::TooBig);
    }
    Ok(std::fs::read(at)?)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn done_at(session: &Mutex<Session>, file: &str, at: &str, asked: Asked) -> Answer<Option<String>> {
    let (data, target, piece) = one_place(&held(session), file, at)?;
    // What writes waits for any sync round, which writes the same files under the lock.
    let mut writing = matches!(asked, Asked::Restore { .. } | Asked::Bring).then(|| held(session));
    match (
        done(&data, &ledger(&data), &target, piece, asked)?,
        writing.as_deref_mut(),
    ) {
        (Some(body), Some(session)) if asked == Asked::Bring => kept_in(
            session,
            Homes::here().as_ref(),
            file,
            &body,
            &target.assistant,
        ),
        (told, _) => Ok(told),
    }
}

// Brought words are written as the assistant whose file they come from, so the history keeps what the
// document said before, and a round settles the file at once.
pub(crate) fn kept_in(
    session: &mut Session,
    homes: Option<&Homes>,
    file: &str,
    body: &str,
    by: &str,
) -> Answer<Option<String>> {
    let was = tisty_core::docs::read(&session.paths.docs(), file)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    if tisty_core::docs::unchanged(&was, body) {
        return Err(Refusal::of("agentBringSame"));
    }
    super::agents_sync::rewritten_by(session, file, body, Some(by.to_string()))?;
    if let Some(homes) = homes {
        let mut ledger = ledger(session.paths.data());
        super::agents_sync::round_now(session, homes, &mut ledger);
    }
    Ok(None)
}

#[cfg(test)]
#[path = "agent_docs_test.rs"]
mod tests;
