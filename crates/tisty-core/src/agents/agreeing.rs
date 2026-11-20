use std::path::Path;

use crate::State;
use crate::event::DeviceId;
use crate::model::Kept;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Hand {
    Here,
    Elsewhere,
    Agent,
    Earlier,
    Outside,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Approved {
    Never,
    Kept(String),
    Elsewhere,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agreeing {
    pub hand: Hand,
    pub approved: Approved,
}

/// What the person is about to approve: the words last approved, when this computer keeps them, and whose hand wrote these.
pub fn agreeing(state: &State, data: &Path, kept: &Kept, body: &str, me: &DeviceId) -> Agreeing {
    let approved = match kept.agreed.as_deref() {
        None => Approved::Never,
        Some(print) => crate::history::read(data, &kept.file, print)
            .map_or(Approved::Elsewhere, Approved::Kept),
    };
    Agreeing {
        hand: hand(state, data, kept, body, me),
        approved,
    }
}

fn hand(state: &State, data: &Path, kept: &Kept, body: &str, me: &DeviceId) -> Hand {
    let print = crate::docs::print_of_text(body);
    if kept.print.as_deref() != Some(print.as_str()) {
        return Hand::Outside;
    }
    let agent = kept.wrote_via.is_some()
        || kept
            .wrote_by
            .as_ref()
            .is_some_and(|who| state.assistants.contains(who));
    if agent {
        return Hand::Agent;
    }
    if crate::history::read(data, &kept.file, &print).is_some() {
        return Hand::Earlier;
    }
    match kept.wrote_by.as_ref() == Some(me) {
        true => Hand::Here,
        false => Hand::Elsewhere,
    }
}

#[cfg(test)]
#[path = "agreeing_test.rs"]
mod tests;
