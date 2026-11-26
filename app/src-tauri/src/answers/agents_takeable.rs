use std::sync::Mutex;

use tisty_core::agents::{self, Found, Kind, Ledger};

use super::agents_sync::{gathered, here, homes, takeable_with};
use crate::wiring::agents::{self as places, Homes};
use crate::{Answer, Session, held};

/// Without this computer's yes no agent folder is listed at all, and the answer says so with nothing.
#[tauri::command(async)]
pub fn agents_takeable(session: tauri::State<'_, Mutex<Session>>) -> Answer<Option<Vec<String>>> {
    let homes = homes()?;
    let session = held(&session);
    Ok(takeable(&session, &homes, &here()))
}

pub(crate) fn takeable(session: &Session, homes: &Homes, here: &str) -> Option<Vec<String>> {
    if !Ledger::open(session.paths.data(), here).consented() {
        return None;
    }
    let found = rules_and_skills(homes);
    Some(takeable_with(
        &gathered(session, here),
        homes,
        &found,
        &super::tisty_skills::bundled_here(),
    ))
}

fn rules_and_skills(homes: &Homes) -> Vec<Found> {
    let spots: Vec<agents::Place> = homes
        .detected()
        .into_iter()
        .flat_map(|one| places::spots(homes, one))
        .filter(|one| one.kind != Kind::Memory)
        .collect();
    agents::found(&spots)
}

#[cfg(test)]
#[path = "agents_takeable_test.rs"]
mod tests;
