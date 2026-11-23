use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::agents::{self, Card, Ledger, Refused, Seen, Standing, Target};

use super::agents_sync::{refused, unled};
use crate::wiring::agents::{Homes, Placed, assistant, lead_of, targets};
use crate::{Answer, Refusal, Session, held};

#[derive(serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum Said {
    Synced,
    Pending,
    Unled,
    Held,
    Unproven,
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
    pinned: bool,
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
    bytes: Option<u64>,
    kept: Vec<Backup>,
    /// The documents that hold this file back, by title.
    held_by: Vec<String>,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentDoc {
    kind: &'static str,
    level: String,
    named: Option<String>,
    name: String,
    label: String,
    code: bool,
    /// Sync is off for the whole AGENTS folder, so no single assistant can be turned back on here.
    folder_off: bool,
    /// Another document in AGENTS carries the same card, so this one is never written.
    twin: bool,
    /// Whether this computer writes it, and if not, why: the person's choice or approval waits.
    here: &'static str,
    /// The words the card was drawn from, so an approval names the text the person saw.
    print: String,
    places: Vec<Place>,
    /// An assistant's change waiting beside the approved text, shown as a comparison.
    proposal: Option<Box<super::agent_proposals::Proposal>>,
    /// Words waiting for the person's approval: what changed since the last approval, and whose hand wrote them.
    agreeing: Option<Box<Agreeing>>,
    own: agents::OwnFront,
}

#[derive(serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Agreeing {
    hand: agents::Hand,
    approved: &'static str,
    lines: Vec<super::agents_sync::Line>,
}

fn agreeing_of(session: &Session, file: &str, body: &str) -> Option<Box<Agreeing>> {
    let kept = session.state.docs.values().find(|one| one.file == file)?;
    let said = agents::agreeing(
        &session.state,
        session.paths.data(),
        kept,
        body,
        session.store.device(),
    );
    let (approved, lines) = match said.approved {
        agents::Approved::Never => ("never", Vec::new()),
        agents::Approved::Elsewhere => ("elsewhere", Vec::new()),
        agents::Approved::Kept(was) => (
            "kept",
            super::agents_sync::compared(was.as_bytes(), body.as_bytes()),
        ),
    };
    Some(Box::new(Agreeing {
        hand: said.hand,
        approved,
        lines,
    }))
}

fn named(id: &str) -> String {
    assistant(id).map_or_else(|| id.to_string(), |one| one.name.to_string())
}

pub(crate) fn said(standing: Standing, piece: usize) -> Said {
    match standing {
        Standing::UpToDate | Standing::Agreed => Said::Synced,
        Standing::Write | Standing::Wrote => Said::Pending,
        Standing::Held | Standing::Foreign { .. } => Said::Held,
        Standing::Unproven => Said::Unproven,
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

fn bytes_at(at: &Path) -> Option<u64> {
    std::fs::metadata(at).ok().map(|one| one.len())
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
        said: match lead_of(target).is_some_and(|lead| unled(ledger, &lead)) {
            true => Said::Unled,
            false => said(agents::standing(ledger, target), piece),
        },
        shared: target.pieces.len(),
        changed: changed_at(&target.at),
        bytes: bytes_at(&target.at),
        held_by: Vec::new(),
        kept: agents::kept(data, &target.at)
            .into_iter()
            .map(|one| Backup {
                n: one.n,
                bytes: one.bytes,
                pinned: one.pinned,
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
    body: String,
    print: String,
    gate: agents::Gate,
    label: String,
    data: PathBuf,
    ledger: Ledger,
    twin: bool,
    here: Seen,
    places: Vec<(Target, usize)>,
}

// Words approved here that still wait say so, or approving would read as applying.
fn here_word(seen: Seen, agreed_here: bool) -> &'static str {
    match seen {
        Seen::Admitted => "applied",
        Seen::WaitingHere if agreed_here => "approvedWaiting",
        Seen::WaitingHere => "waiting",
        Seen::DeclinedHere => "declined",
        Seen::NotAgreed => "notAgreed",
        Seen::PlacedByAgent => "placedByAgent",
        Seen::OtherComputersMemory => "otherMemory",
        Seen::Away | Seen::Plain | Seen::Unread => "away",
    }
}

fn agreed_here(session: &Session, file: &str) -> bool {
    session
        .state
        .docs
        .values()
        .find(|one| one.file == file)
        .is_some_and(|kept| kept.agreed_by.as_ref() == Some(session.store.device()))
}

// Only a document in AGENTS is written anywhere; elsewhere its card is a property like any other.
fn opened(
    session: &Session,
    homes: Option<&Homes>,
    machine: &str,
    file: &str,
) -> Answer<Option<Opened>> {
    let bodies = super::agents_sync::bodies(session);
    let Some((card, label, body)) = bodies
        .iter()
        .find(|(one, _)| one == file)
        .and_then(|(_, body)| Some((Card::of(body)?, agents::shown(body)?.label, body.clone())))
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
    let data = session.paths.data().to_path_buf();
    let ledger = Ledger::open(&data, machine);
    let gate = super::agents_sync::gated(session, &ledger);
    let here = gate.seen(file).unwrap_or(Seen::Away);
    let all = match homes {
        Some(homes) => targets(homes, &gate, &ledger),
        None => Vec::new(),
    };
    let written = |one: &Placed| one.files.iter().any(|named| twins.contains(&named));
    let twin = all.iter().any(written);
    let places = placed_for(file, all);
    Ok(Some(Opened {
        card,
        print: tisty_core::docs::print_of_text(&body),
        body,
        label,
        data,
        ledger,
        gate,
        twin: twin && places.is_empty() && here == Seen::Admitted,
        here,
        places,
    }))
}

fn one_place(session: &Session, file: &str, at: &str) -> Answer<(PathBuf, Target, usize)> {
    let machine = tisty_core::machine::here().unwrap_or_default();
    let opened = opened(session, Homes::here().as_ref(), &machine, file)?
        .ok_or_else(|| Refusal::of("agentNoCard"))?;
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
    Card(Option<Box<AgentDoc>>),
    Text(Option<String>),
}

fn carded(card: Option<AgentDoc>) -> Told {
    Told::Card(card.map(Box::new))
}

/// The card of a document, or, given a file and something to do there, what doing it gives back.
#[tauri::command(async)]
pub fn agent_doc(
    session: tauri::State<'_, Mutex<Session>>,
    file: String,
    at: Option<String>,
    asked: Option<Asked>,
) -> Answer<Told> {
    super::agents_sync::joined_here(&mut held(&session))?;
    match (at, asked) {
        (_, Some(Asked::Take { here })) => taken_here(&session, &file, here).map(carded),
        (_, Some(Asked::Agree { print })) => agreed(&session, &file, &print).map(carded),
        (_, Some(Asked::Approve { print, over })) => settled(&session, &file, |one| {
            super::agent_proposals::approve(one, &file, &print, &over)
        }),
        (_, Some(Asked::Discard { print })) => settled(&session, &file, |one| {
            super::agent_proposals::discard(one, &file, &print)
        }),
        (Some(at), Some(asked)) => done_at(&session, &file, &at, asked).map(Told::Text),
        _ => card(&session, &file).map(carded),
    }
}

fn settled(
    session: &Mutex<Session>,
    file: &str,
    then: impl FnOnce(&mut Session) -> Answer<()>,
) -> Answer<Told> {
    let machine = tisty_core::machine::here().unwrap_or_default();
    {
        let mut one = held(session);
        then(&mut one)?;
        if let Some(homes) = Homes::here() {
            let mut ledger = Ledger::open(one.paths.data(), &machine);
            super::agents_sync::round_now(&one, &homes, &mut ledger);
        }
    }
    card(session, file).map(carded)
}

fn taken_here(session: &Mutex<Session>, file: &str, here: bool) -> Answer<Option<AgentDoc>> {
    let machine = tisty_core::machine::here().unwrap_or_default();
    take_here(&held(session), Homes::here().as_ref(), &machine, file, here)?;
    card(session, file)
}

fn agreed(session: &Mutex<Session>, file: &str, print: &str) -> Answer<Option<AgentDoc>> {
    let machine = tisty_core::machine::here().unwrap_or_default();
    agree(
        &mut held(session),
        Homes::here().as_ref(),
        &machine,
        file,
        print,
    )?;
    card(session, file)
}

pub(crate) fn take_here(
    session: &Session,
    homes: Option<&Homes>,
    machine: &str,
    file: &str,
    here: bool,
) -> Answer<()> {
    let data = session.paths.data();
    let mut ledger = Ledger::open(data, machine);
    ledger.take(file, here);
    ledger.keep(data).map_err(refused)?;
    if let Some(homes) = homes {
        super::agents_sync::round_now(session, homes, &mut ledger);
    }
    Ok(())
}

// Only the words the person was shown become theirs, so they may reach the agents' files.
pub(crate) fn agree(
    session: &mut Session,
    homes: Option<&Homes>,
    machine: &str,
    file: &str,
    print: &str,
) -> Answer<()> {
    let body = tisty_core::docs::read(&session.paths.docs(), file)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    if tisty_core::docs::print_of_text(&body) != print {
        return Err(Refusal::of("agentDocChanged"));
    }
    let hand = crate::signing(&session.state);
    session.retell_written(file, &body, hand);
    if let Some(homes) = homes {
        let mut ledger = Ledger::open(session.paths.data(), machine);
        super::agents_sync::round_now(session, homes, &mut ledger);
    }
    Ok(())
}

fn held_by(
    state: &tisty_core::State,
    gate: &agents::Gate,
    ledger: &Ledger,
    target: &Target,
) -> Vec<String> {
    ledger
        .applied_at(&target.at)
        .filter(|file| gate.held(file))
        .map(|file| {
            state
                .docs
                .values()
                .find(|one| one.file == file)
                .and_then(|one| one.title.clone())
                .unwrap_or_else(|| file.to_string())
        })
        .collect()
}

fn card(session: &Mutex<Session>, file: &str) -> Answer<Option<AgentDoc>> {
    let machine = tisty_core::machine::here().unwrap_or_default();
    let mut session = held(session);
    session.reload()?;
    super::agents_sync::joined_here(&mut session)?;
    card_of(&session, Homes::here().as_ref(), &machine, file)
}

pub(crate) fn card_of(
    session: &Session,
    homes: Option<&Homes>,
    machine: &str,
    file: &str,
) -> Answer<Option<AgentDoc>> {
    let Some(opened) = opened(session, homes, machine, file)? else {
        return Ok(None);
    };
    let ledger = &opened.ledger;
    let card = opened.card;
    Ok(Some(AgentDoc {
        proposal: super::agent_proposals::proposal_of(session, file, &opened.body).map(Box::new),
        agreeing: (opened.here == Seen::NotAgreed)
            .then(|| agreeing_of(session, file, &opened.body))
            .flatten(),
        own: agents::own_front(&opened.body),
        kind: card.kind.word(),
        named: (card.level != agents::GLOBAL).then(|| named(&card.level)),
        code: card.code(),
        folder_off: ledger.consented() && !ledger.syncs(""),
        twin: opened.twin,
        here: here_word(opened.here, agreed_here(session, file)),
        print: opened.print,
        places: opened
            .places
            .iter()
            .map(|(target, piece)| Place {
                held_by: held_by(&session.state, &opened.gate, ledger, target),
                ..place(&opened.data, ledger, target, *piece)
            })
            .collect(),
        level: card.level,
        name: card.name,
        label: opened.label,
    }))
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "do")]
pub enum Asked {
    /// The document as the file now says it, written in through the core so its history keeps what it said.
    Bring {
        #[serde(default)]
        print: Option<String>,
    },
    /// What the file holds now, or one of its backups, as text to look at.
    Held {
        backup: Option<u64>,
    },
    Restore {
        backup: u64,
    },
    /// «Apply here» or «not on this computer», kept in this computer's ledger alone.
    Take {
        here: bool,
    },
    Agree {
        print: String,
    },
    /// The proposal the card showed, printed `print`, over the text it showed it against, printed `over`.
    Approve {
        print: String,
        over: String,
    },
    Discard {
        print: String,
    },
}

pub(crate) fn done(
    data: &Path,
    ledger: &Ledger,
    target: &Target,
    piece: usize,
    asked: &Asked,
) -> Answer<Option<String>> {
    match *asked {
        Asked::Held { backup: None } if !ledger.consented() => Err(Refused::NoConsent),
        Asked::Bring { .. } => agents::bring(ledger, target, piece).map(Some),
        Asked::Held { backup: Some(n) } => {
            agents::read_kept(data, &target.at, n).map(|bytes| Some(text(&bytes)))
        }
        Asked::Held { backup: None } => now(&target.at).map(|bytes| Some(text(&bytes))),
        Asked::Restore { backup } => agents::restore(data, ledger, target, backup).map(|()| None),
        Asked::Take { .. }
        | Asked::Agree { .. }
        | Asked::Approve { .. }
        | Asked::Discard { .. } => Ok(None),
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
    let mut writing =
        matches!(asked, Asked::Restore { .. } | Asked::Bring { .. }).then(|| held(session));
    match (
        done(&data, &ledger(&data), &target, piece, &asked)?,
        writing.as_deref_mut(),
        &asked,
    ) {
        (Some(body), Some(session), Asked::Bring { print }) => kept_in(
            session,
            Homes::here().as_ref(),
            Brought {
                file,
                body: &body,
                by: &target.assistant,
                seen: print.as_deref(),
            },
        ),
        (told, _, _) => Ok(told),
    }
}

// Brought words are written as the assistant whose file they come from, so the history keeps what the
// document said before, and a round settles the file at once.
pub(crate) struct Brought<'a> {
    pub(crate) file: &'a str,
    pub(crate) body: &'a str,
    pub(crate) by: &'a str,
    pub(crate) seen: Option<&'a str>,
}

pub(crate) fn kept_in(
    session: &mut Session,
    homes: Option<&Homes>,
    brought: Brought,
) -> Answer<Option<String>> {
    let Brought {
        file,
        body,
        by,
        seen,
    } = brought;
    let was = tisty_core::docs::read(&session.paths.docs(), file)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    // Words written after the card was drawn are never brought over unseen.
    if seen.is_some_and(|seen| tisty_core::docs::print_of_text(&was) != seen) {
        return Err(Refusal::of("agentDocChanged"));
    }
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
