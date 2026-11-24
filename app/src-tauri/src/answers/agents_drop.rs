use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use tisty_core::agents::{self, Gate, Ledger, Target};
use tisty_core::model::DocId;

use super::agents_sync::{
    Loose, gated, gated_without, here, homes, joined_here, loose_in, paused_readers, refused,
    root_of, round_now, unlead,
};
use crate::wiring::agents::{self as places, ASSISTANTS, Homes, Placed};
use crate::{Answer, Refusal, Session, held};

#[derive(Debug, Default, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reach {
    /// The files «take away too» removes; the answer names them back, and nothing else goes.
    pub at: Vec<String>,
    pub agents: Vec<String>,
    pub changed: Vec<String>,
    pub paused: Vec<String>,
    pub rewritten: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotDropped {
    pub id: String,
    pub refusal: Refusal,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotTaken {
    pub at: String,
    pub refusal: Refusal,
}

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dropped {
    pub gone: Vec<String>,
    pub refused: Vec<NotDropped>,
    pub untaken: Vec<NotTaken>,
    pub unmarked: Vec<NotTaken>,
}

pub struct Answered {
    pub away: bool,
    pub at: Vec<String>,
}

fn going(session: &Session, ids: &[DocId]) -> BTreeSet<String> {
    let state = &session.state;
    ids.iter()
        .flat_map(|id| state.kin(*id).into_iter().chain([*id]))
        .filter_map(|id| state.docs.get(&id))
        .flat_map(|kept| {
            std::iter::once(kept.file.clone()).chain(
                state
                    .pages_of(kept.id)
                    .into_iter()
                    .map(|one| one.file.clone()),
            )
        })
        .collect()
}

fn wrote_any(ledger: &Ledger, at: &Path, going: &BTreeSet<String>) -> bool {
    ledger.applied_at(at).any(|file| going.contains(file))
}

fn reader_ids(homes: &Homes, target: &Target) -> Vec<&'static str> {
    places::readers_of(homes, &target.at, &target.assistant)
        .into_iter()
        .map(|one| one.id)
        .collect()
}

struct Foreseen<'a> {
    homes: &'a Homes,
    ledger: &'a Ledger,
    going: &'a BTreeSet<String>,
    gate: Gate,
    placed: Vec<Placed>,
    rooted: bool,
}

impl<'a> Foreseen<'a> {
    fn new(
        session: &Session,
        homes: &'a Homes,
        ledger: &'a Ledger,
        going: &'a BTreeSet<String>,
    ) -> Self {
        let gate = gated_without(session, ledger, going);
        let placed = places::targets(homes, &gate, ledger);
        Self {
            homes,
            ledger,
            going,
            gate,
            placed,
            rooted: root_of(&session.state).is_some(),
        }
    }

    fn freed(&self) -> Vec<(Loose, Target)> {
        loose_in(
            self.homes,
            &self.gate,
            self.ledger,
            &self.placed,
            self.rooted,
        )
        .into_iter()
        .filter(|(_, target)| wrote_any(self.ledger, &target.at, self.going))
        .collect()
    }

    // From the ledger alone: a paused agent's file is named, never opened.
    fn paused(&self) -> BTreeSet<&'static str> {
        let (homes, ledger) = (self.homes, self.ledger);
        let placing = |at: &Path| {
            self.placed
                .iter()
                .any(|one| agents::same_file(&one.target.at, at))
        };
        ledger
            .written_files()
            .filter(|(at, _)| wrote_any(ledger, at, self.going) && !placing(at))
            .filter(|(at, _)| !places::agents_own(homes, at) && !agents::shipped_copy(ledger, at))
            .filter_map(|(at, _)| {
                let (id, root) = places::owner_of(homes, at)?;
                Some(self.gate.target(ledger, root, at.to_path_buf(), id))
            })
            .flat_map(|target| paused_readers(homes, ledger, &target))
            .collect()
    }

    // A file other documents still make up is written again without these ones, never taken away.
    fn rewritten(&self) -> BTreeSet<&'static str> {
        self.placed
            .iter()
            .filter(|one| !one.target.held() && wrote_any(self.ledger, &one.target.at, self.going))
            .flat_map(|one| reader_ids(self.homes, &one.target))
            .filter(|id| self.ledger.syncs(id))
            .collect()
    }
}

fn named(ids: &BTreeSet<&str>) -> Vec<String> {
    ASSISTANTS
        .iter()
        .filter(|one| ids.contains(one.id))
        .map(|one| one.name.to_string())
        .collect()
}

fn ids_of(raw: &[String]) -> Vec<DocId> {
    raw.iter().filter_map(|one| one.parse().ok()).collect()
}

pub fn reach(session: &Session, homes: &Homes, here: &str, ids: &[String]) -> Reach {
    let ledger = Ledger::open(session.paths.data(), here);
    if !ledger.consented() {
        return Reach::default();
    }
    let going = going(session, &ids_of(ids));
    let foreseen = Foreseen::new(session, homes, &ledger, &going);
    let (mut goes, mut changed, paused) = (BTreeSet::new(), BTreeSet::new(), foreseen.paused());
    let mut at = Vec::new();
    for (one, target) in foreseen.freed() {
        match one.changed {
            false => {
                goes.extend(reader_ids(homes, &target));
                at.push(one.at);
            }
            true => changed.extend(reader_ids(homes, &target)),
        }
    }
    Reach {
        at,
        agents: named(&goes),
        changed: named(&changed),
        paused: named(&paused),
        rewritten: named(&foreseen.rewritten()),
    }
}

// Only a path the question showed, still loose, unchanged and read by no paused agent once the documents are gone, is acted on.
pub fn dropped(
    session: &mut Session,
    homes: &Homes,
    here: &str,
    ids: &[String],
    answer: &Answered,
) -> Answer<Dropped> {
    let mut out = Dropped::default();
    for id in ids {
        match session.drop_doc(id) {
            Ok(()) => out.gone.push(id.clone()),
            Err(refusal) => out.refused.push(NotDropped {
                id: id.clone(),
                refusal,
            }),
        }
    }
    let data = session.paths.data().to_path_buf();
    let mut ledger = Ledger::open(&data, here);
    if !ledger.consented() || answer.at.is_empty() {
        return Ok(out);
    }
    let gate = gated(session, &ledger);
    let placed = places::targets(homes, &gate, &ledger);
    let rooted = root_of(&session.state).is_some();
    let loose: Vec<(Loose, Target)> = loose_in(homes, &gate, &ledger, &placed, rooted)
        .into_iter()
        .filter(|(one, _)| answer.at.contains(&one.at))
        .filter(|(one, _)| !one.changed)
        .collect();
    for (one, target) in &loose {
        if !answer.away {
            ledger.stay(&target.at);
            continue;
        }
        match agents::removed_as_written(&data, &mut ledger, target) {
            Ok(true) => unlead(&data, &mut ledger, target),
            Ok(false) => {}
            Err(why) => out.untaken.push(NotTaken {
                at: one.at.clone(),
                refusal: refused(why),
            }),
        }
    }
    if !answer.away
        && !loose.is_empty()
        && let Err(why) = ledger.keep(&data)
    {
        let refusal = refused(why);
        out.unmarked.extend(loose.iter().map(|(one, _)| NotTaken {
            at: one.at.clone(),
            refusal: refusal.clone(),
        }));
    }
    round_now(session, homes, &mut ledger);
    Ok(out)
}

#[tauri::command(async)]
pub fn agents_reach(session: tauri::State<'_, Mutex<Session>>, ids: Vec<String>) -> Answer<Reach> {
    let homes = homes()?;
    let mut session = held(&session);
    session.reload()?;
    joined_here(&mut session)?;
    Ok(reach(&session, &homes, &here(), &ids))
}

#[tauri::command(async)]
pub fn agents_drop(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    ids: Vec<String>,
    away: bool,
    at: Vec<String>,
) -> Answer<Dropped> {
    let homes = homes()?;
    let done = {
        let mut session = held(&session);
        session.reload()?;
        joined_here(&mut session)?;
        dropped(&mut session, &homes, &here(), &ids, &Answered { away, at })?
    };
    if !done.gone.is_empty() {
        super::folio::stops_what_lost_its_document(&app);
    }
    Ok(done)
}

#[cfg(test)]
#[path = "agents_drop_test.rs"]
mod tests;
