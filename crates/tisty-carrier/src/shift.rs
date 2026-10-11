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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shift {
    Homing {
        owed: Owed,
        deferred: Option<Deferred>,
    },
    Filling {
        owed: Owed,
        deferred: Option<Deferred>,
    },
    Ready,
}

pub fn shift(
    from: Option<&dyn Carrier>,
    to: Option<&dyn Carrier>,
    here: &Here,
    alive: &[String],
    saying: &mut dyn FnMut(Reached),
) -> Result<Shift, Trouble> {
    if let Some(from) = from
        && !homing(from, here).nothing()
    {
        let moved = carried(from, here, Way::Pull, alive, saying)?;
        let owed = homing(from, here);
        if !owed.nothing() {
            return Ok(Shift::Homing {
                owed,
                deferred: moved.deferred,
            });
        }
    }
    if let Some(to) = to {
        let moved = carried(to, here, Way::Push, alive, saying)?;
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
    known(here)
        .filter(|(reference, bytes)| {
            !at_home(here, reference, *bytes) && from.keeps(reference) != Some(false)
        })
        .fold(Owed::default(), |owed, (_, bytes)| owed.add(bytes))
}

pub fn filling(to: &dyn Carrier, here: &Here) -> Owed {
    known(here)
        .filter(|(reference, bytes)| {
            at_home(here, reference, *bytes) && to.keeps(reference) != Some(true)
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

fn known(here: &Here) -> impl Iterator<Item = (String, u64)> {
    let store = here.data.join(STORE);
    let state = tisty_core::cache::project(&store, &here.aside)
        .ok()
        .or_else(|| {
            tisty_core::store::read_all(&store)
                .ok()
                .map(|events| tisty_core::State::replay(&events))
        })
        .unwrap_or_default();
    let retired = state.retired;
    state
        .kept
        .into_iter()
        .filter(move |(reference, _)| !retired.contains(reference))
        .map(|(reference, (_, bytes))| (reference, bytes))
}

fn at_home(here: &Here, reference: &str, bytes: u64) -> bool {
    tisty_core::attach::resolve(reference, &here.data).is_ok_and(|at| whole(&at, bytes))
}

fn whole(at: &Path, bytes: u64) -> bool {
    std::fs::metadata(at).is_ok_and(|told| told.is_file() && told.len() == bytes)
        && !tisty_core::holes::a_hole(at)
}
