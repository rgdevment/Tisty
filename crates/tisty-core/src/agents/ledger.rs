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
        let at = super::folder(data).join(NAMED);
        let read = std::fs::metadata(&at)
            .ok()
            .filter(|meta| meta.len() <= READ_AT_MOST)
            .and_then(|_| std::fs::read(&at).ok())
            .and_then(|bytes| serde_json::from_slice::<Self>(&bytes).ok());
        match read {
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
        self.consented && !self.machine.is_empty()
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
        self.wrote.insert(keyed(at), crate::attach::printed(bytes));
    }

    /// The person's «apply here» or «not on this computer», for this computer alone.
    pub fn take(&mut self, file: &str, here: bool) {
        let (into, out) = match here {
            true => (&mut self.took, &mut self.left),
            false => (&mut self.left, &mut self.took),
        };
        out.remove(file);
        into.insert(file.to_string());
    }

    // What the person's hand let in here stays taken after another computer is the last to approve it.
    pub fn taken_all<'a>(&mut self, files: impl IntoIterator<Item = &'a str>) -> bool {
        let before = self.took.len();
        self.took.extend(files.into_iter().map(str::to_string));
        self.took.len() != before
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
        self.took.extend(files.iter().cloned());
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

    pub fn applied(&self) -> BTreeSet<&str> {
        self.applied
            .values()
            .flatten()
            .map(String::as_str)
            .collect()
    }
}

fn keyed(at: &Path) -> String {
    at.to_string_lossy().into_owned()
}

#[cfg(test)]
#[path = "ledger_test.rs"]
mod tests;
