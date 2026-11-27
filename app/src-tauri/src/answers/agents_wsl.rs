use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use tisty_core::agents::{self, Bundled, Gate, Ledger};

use super::agents_sync::{Rounded, here, loose_in, refused, root_of, rounded};
use super::tisty_skills::{bundled_here, unwritten};
use crate::wiring::agents::{self as places, Homes};
use crate::{Answer, Refusal, Session, held, wiring};

const SETTLED: [&str; 5] = ["upToDate", "wrote", "agreed", "noConsent", "off"];

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InDistro {
    pub name: String,
    pub agents: bool,
    pub skills: bool,
    pub running: bool,
    pub looked: bool,
    pub wrote: usize,
    pub left: usize,
}

fn ledger_of(data: &Path, name: &str) -> Answer<Ledger> {
    wiring::distro_id(name)
        .and_then(|id| Ledger::of_distro(data, &here(), &id))
        .ok_or_else(|| Refusal::about("wslUnread", name))
}

fn told(name: &str, ledger: &Ledger, running: bool) -> InDistro {
    InDistro {
        name: name.to_string(),
        agents: ledger.consented(),
        skills: ledger.tisty_skills(),
        running,
        ..InDistro::default()
    }
}

// What reaches a distro is what this computer took; whether it is written there is the distro's own yes.
fn gate_for(session: &Session, mine: &Ledger, distro: &Ledger, bundled: &[Bundled]) -> Gate {
    let docs = session.paths.docs();
    let asked = agents::Asked {
        state: &session.state,
        docs: &docs,
        root: root_of(&session.state),
        ledger: mine,
        me: session.store.device(),
    };
    let mut gate = agents::gate_holding(&asked, distro);
    if distro.tisty_skills() {
        gate.admit_builtins(distro, bundled);
    }
    gate
}

fn homes_in(name: &str) -> Answer<Homes> {
    wiring::distro_home(name)
        .and_then(|found| Homes::in_distro(&found.home, &found.root))
        .ok_or_else(|| Refusal::about("wslHome", name))
}

pub(crate) fn round_in(
    session: &Session,
    homes: &Homes,
    mine: &Ledger,
    ledger: &mut Ledger,
    bundled: &[Bundled],
) -> Rounded {
    let data = session.paths.data();
    let gate = gate_for(session, mine, ledger, bundled);
    let placed = places::targets(homes, &gate, ledger);
    if !ledger.tisty_skills() {
        unwritten(session, homes, ledger, &placed);
    }
    let mut round = rounded(data, ledger, &placed);
    round.loose = taken_away(session, homes, &gate, ledger, &placed);
    round
}

// Nothing comes back from a distro, so a file no document places any more goes, unless someone changed it there.
fn taken_away(
    session: &Session,
    homes: &Homes,
    gate: &Gate,
    ledger: &mut Ledger,
    placed: &[places::Placed],
) -> Vec<super::agents_sync::Loose> {
    let rooted = root_of(&session.state).is_some();
    let mut kept = Vec::new();
    for (loose, target) in loose_in(homes, gate, ledger, placed, rooted) {
        let gone = !loose.changed
            && agents::removed_as_written(session.paths.data(), ledger, &target).unwrap_or(false);
        if !gone {
            kept.push(loose);
        }
    }
    kept
}

fn counted(mut out: InDistro, round: &Rounded) -> InDistro {
    out.looked = true;
    out.wrote = round
        .told
        .iter()
        .filter(|one| one.standing == "wrote")
        .count();
    out.left = round
        .told
        .iter()
        .filter(|one| !SETTLED.contains(&one.standing))
        .count()
        + round.loose.len();
    out
}

fn looked_in(
    session: &Session,
    name: &str,
    mut ledger: Ledger,
    homes: Option<Homes>,
) -> Answer<InDistro> {
    let out = told(name, &ledger, true);
    let copies = ledger
        .written_files()
        .any(|(at, _)| agents::shipped_copy(&ledger, at));
    if !(ledger.consented() || ledger.tisty_skills() || copies) {
        return Ok(out);
    }
    let homes = match homes {
        Some(found) => found,
        None => homes_in(name)?,
    };
    // Judged only while it answers: a distro gone away would read as every file taken away.
    if !running(name) {
        return Ok(InDistro {
            running: false,
            ..out
        });
    }
    let mine = Ledger::open(session.paths.data(), &here());
    let round = round_in(session, &homes, &mine, &mut ledger, &bundled_here());
    Ok(counted(out, &round))
}

fn running(name: &str) -> bool {
    wiring::running_distros().iter().any(|one| one == name)
}

// Only the ledgers Windows keeps are read, so asking about a stopped distro never starts it.
#[tauri::command(async)]
pub fn agents_wsl(
    session: tauri::State<'_, Mutex<Session>>,
    names: Vec<String>,
) -> Answer<Vec<InDistro>> {
    let session = held(&session);
    let up: BTreeSet<String> = wiring::running_distros().into_iter().collect();
    names
        .iter()
        .map(|name| {
            let ledger = ledger_of(session.paths.data(), name)?;
            Ok(told(name, &ledger, up.contains(name)))
        })
        .collect()
}

#[tauri::command(async)]
pub fn agents_wsl_set(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    agents: Option<bool>,
    skills: Option<bool>,
    wake: bool,
) -> Answer<InDistro> {
    let session = held(&session);
    let data = session.paths.data();
    let mut ledger = ledger_of(data, &name)?;
    if let Some(on) = agents {
        ledger.consent(on);
    }
    if let Some(on) = skills {
        ledger.take_tisty_skills(on);
    }
    ledger.keep(data).map_err(refused)?;
    let up = running(&name);
    if !(up || wake) {
        return Ok(told(&name, &ledger, up));
    }
    let homes = homes_in(&name)?;
    looked_in(&session, &name, ledger, Some(homes))
}

#[tauri::command(async)]
pub fn agents_wsl_round(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    wake: bool,
) -> Answer<InDistro> {
    let session = held(&session);
    let ledger = ledger_of(session.paths.data(), &name)?;
    match running(&name) || wake {
        true => looked_in(&session, &name, ledger, None),
        false => Ok(told(&name, &ledger, false)),
    }
}

#[cfg(test)]
#[path = "agents_wsl_test.rs"]
mod tests;
