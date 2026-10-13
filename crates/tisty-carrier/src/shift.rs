use std::path::Path;

use tisty_core::config::Holds;

use crate::{Carrier, Deferred, Here, Moved, Reached, Round, STORE, Trouble, Way};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Owed {
    pub files: usize,
    pub bytes: u64,
}

impl Owed {
    pub fn nothing(&self) -> bool {
        self.files == 0
    }

    fn add(mut self, bytes: u64) -> Self {
        self.files += 1;
        self.bytes += bytes;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shift {
    Homing {
        owed: Owed,
        deferred: Option<Deferred>,
    },
    Filling {
        owed: Owed,
        deferred: Option<Deferred>,
    },
    Unsettled,
    Ready,
}

pub(crate) fn settled(moved: Moved) -> Result<Option<Moved>, Trouble> {
    if let Some(trouble) = moved.refused {
        return Err(trouble);
    }
    let open = moved.unprojected
        || !moved.undecided.is_empty()
        || !moved.astray.is_empty()
        || !moved.coming.is_empty()
        || !moved.waiting.is_empty();
    Ok((!open).then_some(moved))
}

pub fn shift(
    from: Option<&dyn Carrier>,
    to: Option<&dyn Carrier>,
    here: &Here,
    alive: &[String],
    saying: &mut dyn FnMut(Reached),
) -> Result<Shift, Trouble> {
    if let Some(from) = from {
        let moved = from.bring_home(
            here,
            Round {
                way: Way::Pull,
                alive,
                holds: Holds::Everywhere,
                saying,
            },
        )?;
        let Some(moved) = settled(moved)? else {
            return Ok(Shift::Unsettled);
        };
        let owed = homing(from, here);
        if !owed.nothing() || moved.deferred.is_some() {
            return Ok(Shift::Homing {
                owed,
                deferred: moved.deferred,
            });
        }
    }
    if let Some(to) = to {
        let moved = carried(to, here, Way::Push, alive, saying)?;
        let Some(moved) = settled(moved)? else {
            return Ok(Shift::Unsettled);
        };
        let owed = filling(to, here);
        if !owed.nothing() || moved.deferred.is_some() {
            return Ok(Shift::Filling {
                owed,
                deferred: moved.deferred,
            });
        }
    }
    Ok(Shift::Ready)
}

pub fn homing(from: &dyn Carrier, here: &Here) -> Owed {
    let there = from.shelf();
    known(here)
        .filter(|(reference, bytes)| {
            !at_home(here, reference, *bytes)
                && there
                    .as_ref()
                    .is_none_or(|there| there.get(reference) == Some(bytes))
        })
        .fold(Owed::default(), |owed, (_, bytes)| owed.add(bytes))
}

pub fn filling(to: &dyn Carrier, here: &Here) -> Owed {
    let there = to.shelf();
    let takes = |bytes: u64| !to.frees_space() || bytes <= tisty_core::attach::COPIED_IN_DOC;
    known(here)
        .filter(|(reference, bytes)| {
            takes(*bytes)
                && at_home(here, reference, *bytes)
                && there
                    .as_ref()
                    .is_none_or(|there| there.get(reference) != Some(bytes))
        })
        .fold(Owed::default(), |owed, (_, bytes)| owed.add(bytes))
}

fn carried(
    carrier: &dyn Carrier,
    here: &Here,
    way: Way,
    alive: &[String],
    saying: &mut dyn FnMut(Reached),
) -> Result<Moved, Trouble> {
    carrier.carry(
        here,
        Round {
            way,
            alive,
            holds: Holds::Everywhere,
            saying,
        },
    )
}

pub(crate) fn logged(here: &Here) -> tisty_core::State {
    let store = here.data.join(STORE);
    tisty_core::cache::project(&store, &here.aside)
        .ok()
        .or_else(|| {
            tisty_core::store::read_all(&store)
                .ok()
                .map(|events| tisty_core::State::replay(&events))
        })
        .unwrap_or_default()
}

fn known(here: &Here) -> impl Iterator<Item = (String, u64)> {
    let state = logged(here);
    let buried = tisty_sync::buried_now(&state, &here.data);
    state
        .kept
        .into_iter()
        .filter(move |(reference, _)| !buried.contains(reference))
        .map(|(reference, (_, bytes))| (reference, bytes))
}

fn at_home(here: &Here, reference: &str, bytes: u64) -> bool {
    tisty_core::attach::resolve(reference, &here.data).is_ok_and(|at| whole(&at, bytes))
}

fn whole(at: &Path, bytes: u64) -> bool {
    std::fs::metadata(at).is_ok_and(|told| told.is_file() && told.len() == bytes)
        && !tisty_core::holes::a_hole(at)
}
