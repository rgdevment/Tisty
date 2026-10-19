use std::time::{SystemTime, UNIX_EPOCH};

use tisty_carrier::Hitch;

use super::owed::Owed;
use super::tree::{BIN, FILE_FIELDS, File, split};
use super::wire::json;
use super::{Drive, EDIT, READ};
use crate::ASKED_WITHIN;

// Older than this, what sits in the bin is a deletion cut short, never one another machine is still checking.
const LEFT_FOR: u64 = 60 * 60;

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs())
}

fn set_aside_at(name: &str) -> Option<u64> {
    name.split_once('-')?.0.parse().ok()
}

impl Drive {
    pub(super) fn looked_at(&self, id: &str) -> Result<File, Hitch> {
        self.spent(READ);
        self.file_now(id)
    }

    pub(super) fn file_now(&self, id: &str) -> Result<File, Hitch> {
        let url = format!("{}/files/{id}", self.at.api);
        json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("fields", FILE_FIELDS)])
        })?)
    }

    pub(super) fn deleted_for_good(&self, id: &str) -> Result<(), Hitch> {
        let url = format!("{}/files/{id}", self.at.api);
        match self.call(Some(ASKED_WITHIN), |http| http.delete(&url)) {
            Ok(_) | Err(Hitch::Missing(_)) => {
                self.owned(id, None);
                Ok(())
            }
            Err(other) => Err(other),
        }
    }

    // Set aside and read again before it goes for good, so a write that landed after the check is put back, not lost.
    pub(super) fn set_aside(&self, name: &str, first: &File) -> Result<(), Hitch> {
        let bin = self
            .folder_id(Some(BIN), true)?
            .ok_or_else(|| Hitch::Broke("Tisty's bin in Drive could not be made".into()))?;
        let parent = first
            .parents
            .first()
            .ok_or_else(|| Hitch::Broke(format!("{name} sits in no folder in Drive")))?;
        let leaf = split(name).1;
        self.moved_into(&first.id, &format!("{}-{leaf}", now()), (parent, &bin))?;
        let back = Owed::Unbinned {
            name: name.to_string(),
            id: first.id.clone(),
            parent: parent.clone(),
            bin: bin.clone(),
        };
        let now = match self.looked_at(&first.id) {
            Ok(now) => now,
            Err(Hitch::Missing(_)) => return Ok(()),
            Err(other) => return Err(self.put_back_later(back, other)),
        };
        if (&now.head_revision_id, &now.md5_checksum)
            == (&first.head_revision_id, &first.md5_checksum)
        {
            return Ok(());
        }
        if let Err(failed) = self.moved_into(&first.id, leaf, (&bin, parent)) {
            return Err(self.put_back_later(back, failed));
        }
        Err(Hitch::Changed(name.to_string()))
    }

    // Left in the bin it would be swept for good within the hour, so putting it back is written down for a later round.
    fn put_back_later(&self, back: Owed, failed: Hitch) -> Hitch {
        match self.owe(back) {
            Ok(()) => failed,
            Err(unwritten) => unwritten,
        }
    }

    pub(super) fn put_back(
        &self,
        name: &str,
        id: &str,
        (bin, parent): (&str, &str),
    ) -> Result<(), Hitch> {
        if self.looked_at(id)?.parents.first().map(String::as_str) != Some(bin) {
            return Ok(());
        }
        self.moved_into(id, split(name).1, (bin, parent))
            .map(|_| ())
    }

    pub(super) fn moved_into(
        &self,
        id: &str,
        named: &str,
        (from, to): (&str, &str),
    ) -> Result<File, Hitch> {
        let url = format!("{}/files/{id}", self.at.api);
        let body = serde_json::json!({ "name": named });
        self.spent(EDIT);
        json(self.call(Some(ASKED_WITHIN), |http| {
            http.patch(&url)
                .query(&[
                    ("addParents", to),
                    ("removeParents", from),
                    ("fields", FILE_FIELDS),
                ])
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_string())
        })?)
    }

    // Read again before it goes, since a listing held for hours may have seen a file that was put back since.
    pub(super) fn swept(&self) {
        let Some(owed) = self.owed() else {
            return;
        };
        let Some((bin, left)) = self.known.lock().ok().and_then(|mut held| {
            let tree = &mut held.as_mut()?.tree;
            Some((
                tree.folders.get(BIN)?.clone(),
                std::mem::take(&mut tree.binned),
            ))
        }) else {
            return;
        };
        let now = now();
        let mut kept = Vec::new();
        let mut cut_short = false;
        for one in left {
            let due = set_aside_at(&one.name).is_some_and(|at| now.saturating_sub(at) > LEFT_FOR)
                && !owed.iter().any(|debt| debt.puts_back(&one.id));
            if cut_short || !due {
                kept.push(one);
                continue;
            }
            match self.swept_one(&bin, &one) {
                Ok(true) => {}
                Ok(false) => kept.push(one),
                Err(_) => {
                    cut_short = true;
                    kept.push(one);
                }
            }
        }
        self.noted(|tree| {
            kept.retain(|one| tree.binned.iter().all(|listed| listed.id != one.id));
            tree.binned.extend(kept);
        });
    }

    // True once it is gone or left the bin; one whose bytes moved since the listing is a write that landed late, never a leftover.
    fn swept_one(&self, bin: &str, one: &File) -> Result<bool, Hitch> {
        let still = match self.looked_at(&one.id) {
            Ok(still) => still,
            Err(Hitch::Missing(_)) => return Ok(true),
            Err(other) => return Err(other),
        };
        if still.parents.first().map(String::as_str) != Some(bin) || still.name != one.name {
            return Ok(true);
        }
        if (&still.head_revision_id, &still.md5_checksum)
            != (&one.head_revision_id, &one.md5_checksum)
        {
            return Ok(false);
        }
        self.spent(EDIT);
        self.deleted_for_good(&one.id)?;
        Ok(true)
    }
}
