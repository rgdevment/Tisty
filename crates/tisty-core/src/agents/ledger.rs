use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::Refused;
use crate::witness::{self, channel};

const NAMED: &str = "ledger.json";
const READ_AT_MOST: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ledger {
    machine: String,
    #[serde(default)]
    consented: bool,
    #[serde(default)]
    all_off: bool,
    #[serde(default)]
    off: BTreeSet<String>,
    #[serde(default)]
    wrote: BTreeMap<String, String>,
    #[serde(default)]
    took: BTreeSet<String>,
    #[serde(default)]
    left: BTreeSet<String>,
    #[serde(default)]
    applied: BTreeMap<String, BTreeSet<String>>,
    #[serde(default)]
    led: BTreeSet<String>,
    #[serde(default)]
    waits: BTreeSet<String>,
    #[serde(default)]
    tisty_skills: bool,
    #[serde(default)]
    stayed: BTreeSet<String>,
    #[serde(default)]
    chosen: BTreeMap<String, String>,
    #[serde(default)]
    chosen_over: BTreeMap<String, BTreeSet<String>>,
    #[serde(default)]
    words: BTreeMap<String, String>,
    #[serde(default)]
    homing: Option<BTreeSet<String>>,
    #[serde(default)]
    hooks: BTreeMap<String, HookYes>,
}

/// The person's yes to one hook on this computer: the exact words, and the script bytes they became.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HookYes {
    pub print: String,
    pub name: String,
    pub script: String,
    pub event: String,
    pub tools: String,
    pub timeout: u32,
    pub agents: Vec<String>,
}

impl Ledger {
    pub fn fresh(here: &str) -> Self {
        Self {
            machine: here.to_string(),
            ..Self::default()
        }
    }

    // A ledger copied from another computer carries that computer's consent, never this one's.
    pub fn open(data: &Path, here: &str) -> Self {
        match Self::read(data) {
            Some(ledger) if ledger.machine == here => ledger,
            Some(_) => {
                witness::warn(
                    channel::AGENT,
                    "the agent ledger was written on another computer, so this one starts without its consent",
                    &[],
                );
                Self::fresh(here)
            }
            None => Self::fresh(here),
        }
    }

    /// This computer's ledger, or none when the one kept here was written on another computer.
    pub fn mine(data: &Path, here: &str) -> Option<Self> {
        match Self::read(data) {
            Some(ledger) if ledger.machine != here => None,
            Some(ledger) => Some(ledger),
            None => Some(Self::fresh(here)),
        }
    }

    fn read(data: &Path) -> Option<Self> {
        let at = super::folder(data).join(NAMED);
        std::fs::metadata(&at)
            .ok()
            .filter(|meta| meta.len() <= READ_AT_MOST)
            .and_then(|_| std::fs::read(&at).ok())
            .and_then(|bytes| serde_json::from_slice::<Self>(&bytes).ok())
    }

    pub fn keep(&self, data: &Path) -> Result<(), Refused> {
        let folder = super::folder(data);
        std::fs::create_dir_all(&folder)?;
        let _ = crate::paths::ours_alone(&folder);
        let said = serde_json::to_vec_pretty(self).map_err(|e| Refused::Io(e.to_string()))?;
        let at = folder.join(NAMED);
        crate::store::write_atomic(&at, &said)?;
        let _ = crate::paths::ours_alone(&at);
        Ok(())
    }

    pub fn consent(&mut self, given: bool) {
        self.consented = given;
    }

    // A computer that cannot name itself cannot tell its own consent from a copied one.
    pub fn consented(&self) -> bool {
        self.consented && self.named()
    }

    // Tisty's own skills have their own yes; like the other, it never holds on a computer that cannot name itself.
    pub fn tisty_skills(&self) -> bool {
        self.tisty_skills && self.named()
    }

    pub fn named(&self) -> bool {
        !self.machine.is_empty()
    }

    pub fn take_tisty_skills(&mut self, on: bool) {
        self.tisty_skills = on;
    }

    pub fn turn_all(&mut self, on: bool) {
        self.all_off = !on;
    }

    pub fn turn(&mut self, assistant: &str, on: bool) {
        match on {
            true => self.off.remove(assistant),
            false => self.off.insert(assistant.to_string()),
        };
    }

    pub fn syncs(&self, assistant: &str) -> bool {
        !self.all_off && !self.off.contains(assistant)
    }

    pub fn written(&self, at: &Path) -> Option<&str> {
        self.wrote.get(&keyed(at)).map(String::as_str)
    }

    pub fn settle(&mut self, at: &Path, bytes: &[u8]) {
        self.stayed.remove(&folded(at));
        self.wrote.insert(keyed(at), crate::attach::printed(bytes));
    }

    /// The person deleted what wrote this file and chose to leave it with the agent: until Tisty writes it again, it is theirs.
    pub fn stay(&mut self, at: &Path) {
        self.stayed.insert(folded(at));
    }

    pub fn stays(&self, at: &Path) -> bool {
        self.stayed.contains(&folded(at))
    }

    /// The person's «apply here» or «not on this computer», for this computer alone.
    pub fn take(&mut self, file: &str, here: bool) {
        let (into, out) = match here {
            true => (&mut self.took, &mut self.left),
            false => (&mut self.left, &mut self.took),
        };
        out.remove(file);
        into.insert(file.to_string());
        self.waits.remove(file);
    }

    /// Of those that would write one place, the person's choice on this computer, and the sides it was made over.
    pub fn choose(&mut self, place: &str, side: &str, over: impl IntoIterator<Item = String>) {
        self.chosen.insert(place.to_string(), side.to_string());
        self.chosen_over
            .insert(place.to_string(), over.into_iter().collect());
    }

    pub fn chosen(&self, place: &str) -> Option<&str> {
        self.chosen.get(place).map(String::as_str)
    }

    /// A side that came after the choice was never weighed against it, so the choice does not cover it.
    pub fn chosen_over(&self, place: &str, side: &str) -> bool {
        self.chosen_over
            .get(place)
            .is_some_and(|over| over.contains(side))
    }

    /// The words of a document of this computer's own that were let in here, whoever approved them.
    pub fn let_in(&mut self, file: &str, print: &str) -> bool {
        self.words
            .insert(file.to_string(), print.to_string())
            .as_deref()
            != Some(print)
    }

    pub fn let_in_words(&self, file: &str) -> Option<&str> {
        self.words.get(file).map(String::as_str)
    }

    /// What an earlier Tisty left at the root and goes to this computer's folder, once known; none left means it went.
    pub fn homing(&self) -> Option<&BTreeSet<String>> {
        self.homing.as_ref()
    }

    pub fn home_all(&mut self, files: BTreeSet<String>) {
        self.homing = Some(files);
    }

    pub fn went_home(&mut self, file: &str) {
        if let Some(left) = &mut self.homing {
            left.remove(file);
        }
    }

    pub fn homed(&self) -> bool {
        self.homing.as_ref().is_some_and(BTreeSet::is_empty)
    }

    // Once a document waited here, approving its words here still leaves the choice to «apply here».
    pub fn wait_all<'a>(&mut self, files: impl IntoIterator<Item = &'a str>) -> bool {
        let before = self.waits.len();
        self.waits.extend(files.into_iter().map(str::to_string));
        self.waits.len() != before
    }

    pub fn waits(&self, file: &str) -> bool {
        self.waits.contains(file)
    }

    // What the person's hand let in here stays taken after another computer is the last to approve it.
    pub fn taken_all<'a>(&mut self, files: impl IntoIterator<Item = &'a str>) -> bool {
        let before = self.took.len();
        self.took.extend(documents(files));
        self.took.len() != before
    }

    pub fn register_hook(&mut self, file: &str, yes: HookYes) {
        self.hooks.insert(file.to_string(), yes);
    }

    pub fn hook_yes(&self, file: &str) -> Option<&HookYes> {
        self.hooks.get(file)
    }

    pub fn hook_named(&self, name: &str) -> Option<(&str, &HookYes)> {
        self.hooks
            .iter()
            .find(|(_, yes)| yes.name == name)
            .map(|(file, yes)| (file.as_str(), yes))
    }

    pub fn hooks(&self) -> impl Iterator<Item = (&str, &HookYes)> {
        self.hooks.iter().map(|(file, yes)| (file.as_str(), yes))
    }

    pub fn drop_hook(&mut self, file: &str) -> Option<HookYes> {
        self.hooks.remove(file)
    }

    pub fn took(&self, file: &str) -> bool {
        self.took.contains(file)
    }

    pub fn left(&self, file: &str) -> bool {
        self.left.contains(file)
    }

    // What went into a file stays taken here, so a later change of this computer's name drops nothing from it.
    pub fn applied_to(&mut self, at: &Path, files: impl IntoIterator<Item = String>) {
        let files: BTreeSet<String> = files.into_iter().collect();
        self.took
            .extend(documents(files.iter().map(String::as_str)));
        self.applied.insert(keyed(at), files);
    }

    pub fn applied_at(&self, at: &Path) -> impl Iterator<Item = &str> {
        self.applied
            .get(&keyed(at))
            .into_iter()
            .flatten()
            .map(String::as_str)
    }

    // A file written by a build that kept no record of its documents.
    pub fn unproven(&self, at: &Path) -> bool {
        let key = keyed(at);
        self.wrote.contains_key(&key) && !self.applied.contains_key(&key)
    }

    /// A file an earlier build made of several documents.
    pub fn mixed(&self, at: &Path) -> bool {
        self.applied_at(at).nth(1).is_some()
    }

    pub fn written_files(&self) -> impl Iterator<Item = (&Path, &BTreeSet<String>)> {
        self.applied
            .iter()
            .map(|(at, files)| (Path::new(at.as_str()), files))
    }

    // Once Tisty put its line in a file, a line missing later is the person's doing, never put back alone.
    pub fn lead(&mut self, at: &Path) {
        self.led.insert(keyed(at));
    }

    pub fn led(&self, at: &Path) -> bool {
        self.led.contains(&keyed(at))
    }

    pub fn forget(&mut self, at: &Path) {
        let key = keyed(at);
        self.wrote.remove(&key);
        self.applied.remove(&key);
        self.led.remove(&key);
        self.stayed.remove(&folded(at));
    }

    pub fn applied(&self) -> BTreeSet<&str> {
        self.applied
            .values()
            .flatten()
            .map(String::as_str)
            .collect()
    }
}

// Tisty's own skills are taken by their one switch, so their keys never count as a document taken here.
fn documents<'a>(files: impl IntoIterator<Item = &'a str>) -> impl Iterator<Item = String> {
    files
        .into_iter()
        .filter(|file| !super::builtin(file))
        .map(str::to_string)
}

fn keyed(at: &Path) -> String {
    at.to_string_lossy().into_owned()
}

// Windows and macOS keep `Voice` and `voice` in one file, so what the person left there is one choice.
fn folded(at: &Path) -> String {
    super::path_key(at)
}

#[cfg(test)]
#[path = "ledger_test.rs"]
mod tests;
