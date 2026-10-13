use std::time::{SystemTime, UNIX_EPOCH};

use tisty_carrier::Hitch;

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
        let url = format!("{}/files/{id}", self.at.api);
        self.spent(READ);
        json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("fields", FILE_FIELDS)])
        })?)
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
        self.moved(&first.id, &format!("{}-{leaf}", now()), (parent, &bin))?;
        let now = match self.looked_at(&first.id) {
            Ok(now) => now,
            Err(Hitch::Missing(_)) => return Ok(()),
            Err(other) => return Err(other),
        };
        if (&now.head_revision_id, &now.md5_checksum)
            == (&first.head_revision_id, &first.md5_checksum)
        {
            return Ok(());
        }
        self.moved(&first.id, leaf, (&bin, parent))?;
        Err(Hitch::Changed(name.to_string()))
    }

    fn moved(&self, id: &str, named: &str, (from, to): (&str, &str)) -> Result<(), Hitch> {
        let url = format!("{}/files/{id}", self.at.api);
        let body = serde_json::json!({ "name": named });
        self.spent(EDIT);
        self.call(Some(ASKED_WITHIN), |http| {
            http.patch(&url)
                .query(&[
                    ("addParents", to),
                    ("removeParents", from),
                    ("fields", "id"),
                ])
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_string())
        })?;
        Ok(())
    }

    // Read again before it goes, since a listing held for hours may have seen a file that was put back since.
    pub(super) fn swept(&self) {
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
        let spent = left.into_iter().filter(|one| {
            set_aside_at(&one.name).is_some_and(|at| now.saturating_sub(at) > LEFT_FOR)
        });
        for one in spent {
            let still = match self.looked_at(&one.id) {
                Ok(still) => still,
                Err(Hitch::Missing(_)) => continue,
                Err(_) => return,
            };
            if still.parents.first() != Some(&bin) || still.name != one.name {
                continue;
            }
            let url = format!("{}/files/{}", self.at.api, one.id);
            self.spent(EDIT);
            match self.call(Some(ASKED_WITHIN), |http| http.delete(&url)) {
                Ok(_) | Err(Hitch::Missing(_)) => self.owned(&one.id, None),
                Err(_) => return,
            }
        }
    }
}
