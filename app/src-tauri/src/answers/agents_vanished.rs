use std::sync::Mutex;

use tisty_core::agents::{self, Ledger};

use super::agents_sync::{Rounded, gated, here, homes, joined_here, round_now};
use crate::wiring::agents::{self as places, Homes};
use crate::{Answer, Session, held};

#[derive(Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Restored {
    pub wrote: usize,
    pub failed: usize,
    pub round: Rounded,
}

pub(crate) fn restored_here(session: &Session, homes: &Homes, here: &str) -> Restored {
    let data = session.paths.data();
    let mut ledger = Ledger::open(data, here);
    let placed = places::targets(homes, &gated(session, &ledger), &ledger);
    let mut out = Restored::default();
    let gone: Vec<&agents::Target> = placed
        .iter()
        .map(|one| &one.target)
        .filter(|target| agents::vanished(&ledger, target))
        .collect();
    for target in gone {
        match agents::restored(data, &mut ledger, target) {
            Ok(()) => out.wrote += 1,
            Err(why) => {
                out.failed += 1;
                tisty_core::witness::warn(
                    tisty_core::witness::channel::AGENT,
                    "a file taken away from an agent could not be written again",
                    &[("why", tisty_core::witness::Fact::Why(why.to_string()))],
                );
            }
        }
    }
    if out.wrote > 0 {
        let _ = ledger.keep(data);
    }
    out.round = round_now(session, homes, &mut ledger);
    out
}

/// «Write them again»: only files taken away whole come back, never one the person changed.
#[tauri::command(async)]
pub fn agents_restore(session: tauri::State<'_, Mutex<Session>>) -> Answer<Restored> {
    let homes = homes()?;
    let mut session = held(&session);
    joined_here(&mut session)?;
    Ok(restored_here(&session, &homes, &here()))
}

#[cfg(test)]
#[path = "agents_vanished_test.rs"]
mod tests;
