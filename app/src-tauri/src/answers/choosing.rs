use std::path::{Path, PathBuf};
use std::sync::Mutex;

use tisty_core::config::{Holds, Leaving, Sync};

use super::storing::{After, changed_to, cloud_joined};
use crate::{Answer, Refusal, Session, held};

// Only a restored store asks whose store a place holds, since the first look at a cloud brings its whole history.
pub(crate) fn restored_store(session: &Session) -> Option<PathBuf> {
    session
        .config
        .restored_at
        .is_some()
        .then(|| session.paths.store())
}

pub(crate) fn went_back_on(
    restored: Option<&Path>,
    theirs: impl FnOnce() -> Option<String>,
) -> bool {
    restored.is_some_and(|store| {
        theirs().is_some_and(|theirs| {
            tisty_core::store::peek_identity(store).is_some_and(|ours| ours.trim() == theirs.trim())
        })
    })
}

pub(crate) fn folder_way(data: &Path, dest: &str) -> Answer<Sync> {
    let at = PathBuf::from(dest);
    let tangled = at.starts_with(data)
        || data.starts_with(&at)
        || at
            .canonicalize()
            .ok()
            .zip(data.canonicalize().ok())
            .is_some_and(|(a, b)| a.starts_with(&b) || b.starts_with(&a));
    if tangled {
        return Err(Refusal::about("remoteInsideStore", dest));
    }
    Ok(Sync::folder(at))
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Standing {
    way: Option<Sync>,
    holds: Holds,
    here: tisty_carrier::Here,
    joined: bool,
    restored: Option<PathBuf>,
}

impl Standing {
    pub(crate) fn of(session: &Session) -> Self {
        Self {
            way: session.config.sync.clone(),
            holds: session.config.holds(),
            here: session.here(),
            joined: cloud_joined(session),
            restored: restored_store(session),
        }
    }

    fn weighed(&self, dest: Option<&str>) -> Answer<Sync> {
        let chosen = match dest {
            Some(dest) => folder_way(&self.here.data, dest)?,
            None => Sync::alone(),
        };
        if let Some(at) = chosen.place()
            && went_back_on(self.restored.as_deref(), || {
                tisty_carrier::considering(at).theirs()
            })
        {
            return Err(Refusal::about("restoredApart", at.display().to_string()));
        }
        match self.stranded_by_leaving(&chosen) {
            Some(refusal) => Err(refusal),
            None => Ok(chosen),
        }
    }

    pub(crate) fn stranded_by_leaving(&self, chosen: &Sync) -> Option<Refusal> {
        let was = self.way.as_ref().filter(|was| *was != chosen)?;
        match was.leaving() {
            Leaving::Free => None,
            Leaving::Cloud => self.joined.then(|| Refusal::of("cloudToLeave")),
            Leaving::Later => Some(Refusal::of("syncLaterToLeave")),
            Leaving::Place => {
                let old = was.place()?;
                tisty_carrier::strands_on_leaving(was, chosen, self.holds, &self.here)
                    .then(|| Refusal::about("sharedAwayToLeave", old.display().to_string()))
            }
        }
    }
}

pub(crate) fn choose(session: &Mutex<Session>, dest: Option<String>) -> Answer<After> {
    choose_weighing(session, dest, Standing::weighed)
}

fn choose_weighing(
    session: &Mutex<Session>,
    dest: Option<String>,
    weighing: impl Fn(&Standing, Option<&str>) -> Answer<Sync>,
) -> Answer<After> {
    let dest = dest
        .map(|one| one.trim().to_string())
        .filter(|one| !one.is_empty());
    let mut weighed = refreshed(&mut held(session));
    loop {
        let chosen = weighing(&weighed, dest.as_deref())?;
        let mut session = held(session);
        let now = refreshed(&mut session);
        // What was weighed says nothing about a place chosen meanwhile, which is weighed in its turn.
        if now == weighed {
            return changed_to(&mut session, chosen, |one| choosing(one, now.joined));
        }
        weighed = now;
    }
}

fn refreshed(session: &mut Session) -> Standing {
    session.refresh_settings();
    Standing::of(session)
}

pub(crate) fn choosing(
    chosen: Sync,
    joined: bool,
) -> impl FnOnce(&mut tisty_core::Config) -> Option<Refusal> {
    move |config| {
        let leaving = config
            .sync
            .as_ref()
            .filter(|was| **was != chosen)
            .map(|was| was.leaving());
        match leaving {
            Some(Leaving::Later) => Some(Refusal::of("syncLaterToLeave")),
            Some(Leaving::Cloud) if joined => Some(Refusal::of("cloudToLeave")),
            _ => {
                config.sync = Some(chosen);
                None
            }
        }
    }
}

#[cfg(test)]
#[path = "choosing_test.rs"]
mod tests;
