use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::{Bundled, Card, Kind, Ledger, Target};
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
    OtherComputer,
    /// Its `agent:` line and its folder say different computers, so it is nobody's until the person says.
    Apart,
    Away,
    Plain,
    Unread,
    /// A hook runs code, so it waits for this computer's own yes to these exact words, never for «apply here».
    HookWaiting,
    HookRegistered,
}

#[derive(Debug, Default)]
pub struct Gate {
    admitted: Vec<Admitted>,
    seen: BTreeMap<String, Seen>,
    held: BTreeSet<String>,
    carded: BTreeSet<String>,
    unclaimed: BTreeSet<String>,
    skills_named: BTreeMap<String, String>,
    elsewhere: BTreeSet<String>,
    here: BTreeMap<String, Card>,
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

    pub fn carded(&self, file: &str) -> bool {
        self.carded.contains(file)
    }

    /// Agent documents whose words came from another computer and this one has not yet taken or left.
    pub fn unclaimed(&self) -> impl Iterator<Item = &str> {
        self.unclaimed.iter().map(String::as_str)
    }

    /// A document that sits in the folder of another computer.
    pub fn elsewhere(&self, file: &str) -> bool {
        self.elsewhere.contains(file)
    }

    /// Documents whose words the person approved and that this computer may write: taken, waiting or left out here.
    pub fn for_here(&self) -> impl Iterator<Item = (&str, &Card)> {
        self.here.iter().map(|(file, card)| (file.as_str(), card))
    }

    pub fn card_here(&self, file: &str) -> Option<&Card> {
        self.here.get(file)
    }

    /// Agent documents whose words wait for the person's approval.
    pub fn unagreed(&self) -> usize {
        self.all_seen()
            .filter(|(file, seen)| {
                *seen == Seen::NotAgreed
                    && self.carded.contains(*file)
                    && !self.elsewhere.contains(*file)
            })
            .count()
    }

    // They come in with their own yes alone; a skill of the person's in Agents named alike, at any level, keeps them out.
    pub fn admit_builtins(&mut self, ledger: &Ledger, bundled: &[Bundled]) {
        if !ledger.tisty_skills() {
            return;
        }
        let pieces: Vec<Admitted> = bundled
            .iter()
            .filter(|one| !self.skill_named(&one.skill))
            .filter_map(|one| {
                Card::of(&one.body).map(|card| Admitted {
                    file: one.file(),
                    body: one.body.clone(),
                    card,
                })
            })
            .collect();
        self.admitted.extend(pieces);
    }

    /// Tisty's own skills alone, so a round through it never writes a document.
    pub fn shipped_only(mut self) -> Self {
        self.admitted.retain(|one| super::builtin(one.file()));
        self
    }

    /// The gate as it will be once these documents are deleted, so what they leave loose can be asked before.
    pub fn without(mut self, files: &BTreeSet<String>) -> Self {
        self.admitted.retain(|one| !files.contains(one.file()));
        self.seen.retain(|file, _| !files.contains(file));
        self.held.retain(|file| !files.contains(file));
        self.carded.retain(|file| !files.contains(file));
        self.unclaimed.retain(|file| !files.contains(file));
        self.skills_named.retain(|file, _| !files.contains(file));
        self.elsewhere.retain(|file| !files.contains(file));
        self.here.retain(|file, _| !files.contains(file));
        self
    }

    pub fn skill_named(&self, skill: &str) -> bool {
        self.skills_named.values().any(|one| *one == folded(skill))
    }

    pub fn target(&self, ledger: &Ledger, root: PathBuf, at: PathBuf, assistant: &str) -> Target {
        let held = ledger.applied_at(&at).any(|file| self.held.contains(file));
        Target {
            unproven: ledger.unproven(&at) || ledger.mixed(&at),
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
        let (seen, admitted, card) = weighed(asked, kept);
        let elsewhere = super::computer_of(asked.state, kept.folder)
            .is_some_and(|computer| computer != *asked.me);
        if elsewhere {
            gate.elsewhere.insert(kept.file.clone());
        }
        if let Some(card) = card {
            gate.carded.insert(kept.file.clone());
            if card.kind == Kind::Skill && !elsewhere {
                gate.skills_named
                    .insert(kept.file.clone(), folded(super::skill_of(&card.name)));
            }
            if matches!(
                seen,
                Seen::Admitted | Seen::WaitingHere | Seen::DeclinedHere
            ) {
                gate.here.insert(kept.file.clone(), card);
            }
        }
        if !elsewhere && unclaimed(kept, asked.ledger, asked.me) {
            gate.unclaimed.insert(kept.file.clone());
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

// Windows and macOS keep `Tisty` and `tisty` in one folder.
fn folded(skill: &str) -> String {
    skill.to_lowercase()
}

fn weighed(asked: &Asked, kept: &Kept) -> (Seen, Option<Admitted>, Option<Card>) {
    if asked.state.held_away(kept) {
        return (Seen::Away, None, None);
    }
    let Ok(body) = crate::docs::read(asked.docs, &kept.file) else {
        return (Seen::Unread, None, None);
    };
    let card = Card::of(&body);
    let (seen, admitted) = admitted(asked, kept, body, card.clone());
    (seen, admitted, card)
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
    let computer = super::computer_of(state, kept.folder);
    let elsewhere = computer.as_ref().is_some_and(|one| one != *me);
    if elsewhere && placed_by_person(state, kept) {
        return (Seen::OtherComputer, None);
    }
    if kept.agreed.as_deref() != Some(crate::docs::print_of_text(&body).as_str()) {
        return (Seen::NotAgreed, None);
    }
    let Some(card) = card else {
        return (Seen::Plain, None);
    };
    // An earlier Tisty saves such a page as any other, so its approval stands for words it may have rewritten.
    if card.computer.is_some() && !kept.agreed_knowing {
        return (Seen::NotAgreed, None);
    }
    if !placed_by_person(state, kept) {
        return (Seen::PlacedByAgent, None);
    }
    let made_here = kept.made_by.as_ref() == Some(*me);
    if card.kind == Kind::Memory && !made_here {
        return (Seen::OtherComputersMemory, None);
    }
    if computer != card.computer {
        return (Seen::Apart, None);
    }
    if card.kind == Kind::Hook {
        return (hook_seen(ledger, kept), None);
    }
    if ledger.left(&kept.file) {
        return (Seen::DeclinedHere, None);
    }
    // Words another computer approved for this one wait here each time, whatever it took before.
    let let_in = kept.agreed_by.as_ref() == Some(*me)
        || ledger.let_in_words(&kept.file) == kept.agreed.as_deref();
    if card.computer.is_some() && !let_in {
        return (Seen::WaitingHere, None);
    }
    // Words approved here count as applied here only for a document that never waited on this computer.
    let agreed_here = kept.agreed_by.as_ref() == Some(*me) && !ledger.waits(&kept.file);
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

fn hook_seen(ledger: &Ledger, kept: &Kept) -> Seen {
    let yes = ledger.hook_yes(&kept.file).map(|yes| yes.print.as_str());
    match yes.is_some() && yes == kept.agreed.as_deref() {
        true => Seen::HookRegistered,
        false => Seen::HookWaiting,
    }
}

// Words last approved elsewhere, or never approved and made elsewhere, are another computer's until this one chooses.
pub fn unclaimed(kept: &Kept, ledger: &Ledger, me: &DeviceId) -> bool {
    let me = Some(me);
    let elsewhere = match &kept.agreed_by {
        Some(by) => Some(by) != me,
        None => kept.made_by.as_ref() != me,
    };
    elsewhere && !ledger.took(&kept.file) && !ledger.left(&kept.file)
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
        Some(
            Seen::Admitted
            | Seen::DeclinedHere
            | Seen::Plain
            | Seen::OtherComputer
            | Seen::HookWaiting
            | Seen::HookRegistered,
        ) => false,
        Some(
            Seen::NotAgreed
            | Seen::PlacedByAgent
            | Seen::Unread
            | Seen::OtherComputersMemory
            | Seen::WaitingHere
            | Seen::Apart,
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
