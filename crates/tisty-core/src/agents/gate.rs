use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{Card, Kind, Ledger, Target};
use crate::State;
use crate::event::DeviceId;
use crate::model::{FolderId, Kept};

/// A document this computer may write into an agent's file: only `gate` makes one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Admitted {
    file: String,
    body: String,
    card: Card,
}

impl Admitted {
    pub fn file(&self) -> &str {
        &self.file
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn card(&self) -> &Card {
        &self.card
    }

    #[cfg(test)]
    pub(crate) fn trusted(file: &str, body: &str) -> Option<Self> {
        Card::of(body).map(|card| Self {
            file: file.to_string(),
            body: body.to_string(),
            card,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Seen {
    Admitted,
    WaitingHere,
    DeclinedHere,
    NotAgreed,
    PlacedByAgent,
    OtherComputersMemory,
    Away,
    Plain,
    Unread,
}

#[derive(Debug, Default)]
pub struct Gate {
    admitted: Vec<Admitted>,
    seen: BTreeMap<String, Seen>,
    held: BTreeSet<String>,
    carded: BTreeSet<String>,
}

impl Gate {
    pub fn admitted(&self) -> &[Admitted] {
        &self.admitted
    }

    pub fn seen(&self, file: &str) -> Option<Seen> {
        self.seen.get(file).copied()
    }

    pub fn all_seen(&self) -> impl Iterator<Item = (&str, Seen)> {
        self.seen.iter().map(|(file, seen)| (file.as_str(), *seen))
    }

    pub fn held(&self, file: &str) -> bool {
        self.held.contains(file)
    }

    /// Agent documents whose words wait for the person's approval.
    pub fn unagreed(&self) -> usize {
        self.all_seen()
            .filter(|(file, seen)| *seen == Seen::NotAgreed && self.carded.contains(*file))
            .count()
    }

    pub fn target(&self, ledger: &Ledger, root: PathBuf, at: PathBuf, assistant: &str) -> Target {
        let held = ledger.applied_at(&at).any(|file| self.held.contains(file));
        Target {
            unproven: ledger.unproven(&at),
            root,
            at,
            assistant: assistant.to_string(),
            pieces: Vec::new(),
            held,
        }
    }
}

pub struct Asked<'a> {
    pub state: &'a State,
    pub docs: &'a Path,
    pub root: Option<FolderId>,
    pub ledger: &'a Ledger,
    pub me: &'a DeviceId,
}

pub fn gate(asked: &Asked) -> Gate {
    let mut gate = Gate::default();
    let Some(root) = asked.root else {
        return gate;
    };
    let tree = super::tree(asked.state, root);
    for kept in asked.state.docs.values() {
        if kept.task_of.is_some() || !kept.folder.is_some_and(|at| tree.contains(&at)) {
            continue;
        }
        let (seen, admitted, carded) = weighed(asked, kept);
        if carded {
            gate.carded.insert(kept.file.clone());
        }
        gate.seen.insert(kept.file.clone(), seen);
        gate.admitted.extend(admitted);
    }
    gate.held = asked
        .ledger
        .applied()
        .into_iter()
        .filter(|file| held(asked, gate.seen.get(*file).copied(), file))
        .map(str::to_string)
        .collect();
    gate
}

fn weighed(asked: &Asked, kept: &Kept) -> (Seen, Option<Admitted>, bool) {
    if asked.state.held_away(kept) {
        return (Seen::Away, None, false);
    }
    let Ok(body) = crate::docs::read(asked.docs, &kept.file) else {
        return (Seen::Unread, None, false);
    };
    let card = Card::of(&body);
    let carded = card.is_some();
    let (seen, admitted) = admitted(asked, kept, body, card);
    (seen, admitted, carded)
}

fn admitted(
    asked: &Asked,
    kept: &Kept,
    body: String,
    card: Option<Card>,
) -> (Seen, Option<Admitted>) {
    let Asked {
        state, ledger, me, ..
    } = asked;
    if kept.agreed.as_deref() != Some(crate::docs::print_of_text(&body).as_str()) {
        return (Seen::NotAgreed, None);
    }
    let Some(card) = card else {
        return (Seen::Plain, None);
    };
    if !placed_by_person(state, kept) {
        return (Seen::PlacedByAgent, None);
    }
    let made_here = kept.made_by.as_ref() == Some(*me);
    if card.kind == Kind::Memory && !made_here {
        return (Seen::OtherComputersMemory, None);
    }
    if ledger.left(&kept.file) {
        return (Seen::DeclinedHere, None);
    }
    let agreed_here = kept.agreed_by.as_ref() == Some(*me);
    if !(ledger.took(&kept.file) || agreed_here) {
        return (Seen::WaitingHere, None);
    }
    let admitted = Admitted {
        file: kept.file.clone(),
        body,
        card,
    };
    (Seen::Admitted, Some(admitted))
}

fn person(state: &State, who: Option<&DeviceId>) -> bool {
    who.is_some_and(|who| !state.assistants.contains(who))
}

fn placed_by_person(state: &State, kept: &Kept) -> bool {
    let up = kept.page_of.and_then(|up| state.docs.get(&up));
    person(state, kept.placed_by.as_ref())
        && up.is_none_or(|up| person(state, up.placed_by.as_ref()))
        && folders_by_person(state, kept.folder)
}

fn folders_by_person(state: &State, mut at: Option<FolderId>) -> bool {
    let mut deep = 0;
    while let Some(id) = at {
        let Some(folder) = state.folders.get(&id) else {
            return true;
        };
        if !person(state, folder.placed_by.as_ref()) || deep > crate::model::DEEPEST {
            return false;
        }
        at = folder.parent;
        deep += 1;
    }
    true
}

// What this computer once wrote and now fails, or left by an agent's hand, holds its file as it is.
fn held(asked: &Asked, seen: Option<Seen>, file: &str) -> bool {
    match seen {
        Some(Seen::Admitted | Seen::DeclinedHere | Seen::Plain) => false,
        Some(
            Seen::NotAgreed
            | Seen::PlacedByAgent
            | Seen::Unread
            | Seen::OtherComputersMemory
            | Seen::WaitingHere,
        ) => true,
        Some(Seen::Away) | None => asked
            .state
            .docs
            .values()
            .find(|one| one.file == file)
            .is_some_and(|kept| !placed_by_person(asked.state, kept)),
    }
}

#[cfg(test)]
#[path = "gate_test.rs"]
mod tests;
