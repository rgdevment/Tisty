use std::io::Write;

use serde::Deserialize;
use tisty_carrier::Hitch;

use super::owed::Owed;
use super::tree::{FILE_FIELDS, File};
use super::wire::json;
use super::{DOWNLOAD, Drive, EDIT, LIST};
use crate::ASKED_WITHIN;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Revision {
    id: String,
    md5_checksum: Option<String>,
    #[serde(default)]
    modified_time: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Revisions {
    #[serde(default)]
    revisions: Vec<Revision>,
    next_page_token: Option<String>,
}

pub(super) fn version(file: &File) -> u64 {
    file.version
        .as_deref()
        .and_then(|one| one.parse().ok())
        .unwrap_or(0)
}

// One step past the version checked is this write alone; more means another machine landed in between.
pub(super) fn leapt(checked: &File, landed: &File) -> bool {
    version(landed) > version(checked).saturating_add(1)
}

fn same_bytes(checked: &File, md5: Option<&str>) -> bool {
    checked.md5_checksum.is_some() && checked.md5_checksum.as_deref() == md5
}

// Only what landed after the checked revision and before this one can be another machine's; with the checked one gone from the list nothing is.
fn between(all: &[Revision], landed: &File, checked: &File) -> Option<String> {
    let time_of = |head: &Option<String>| {
        all.iter()
            .find(|one| head.as_deref() == Some(one.id.as_str()))
            .map(|one| one.modified_time.clone())
    };
    let after = time_of(&checked.head_revision_id)?;
    let before = time_of(&landed.head_revision_id);
    let ours = landed.head_revision_id.as_deref();
    all.iter()
        .filter(|one| one.modified_time > after && Some(one.id.as_str()) != ours)
        .filter(|one| {
            before
                .as_ref()
                .is_none_or(|before| one.modified_time < *before)
        })
        .filter(|one| !same_bytes(checked, one.md5_checksum.as_deref()))
        .filter(|one| one.md5_checksum != landed.md5_checksum)
        .max_by(|a, b| a.modified_time.cmp(&b.modified_time))
        .map(|one| one.id.clone())
}

impl Drive {
    // What could not be put back now is written down and put back before the next change is read.
    pub(super) fn handed_back(
        &self,
        name: &str,
        landed: &File,
        checked: &File,
    ) -> Result<(), Hitch> {
        let restored = match self.restored(name, landed, checked) {
            Ok(None) => return Ok(()),
            Ok(Some(restored)) => restored,
            Err(failed) => {
                self.owe(Owed::back(name, landed, checked));
                return Err(failed);
            }
        };
        // Never noted as this machine's own: the bytes put back are another machine's, and must come as its change.
        self.noted(|tree| {
            if let Some(first) = tree
                .files
                .get_mut(name)
                .and_then(|copies| copies.first_mut())
            {
                *first = restored;
            }
        });
        Err(Hitch::Changed(name.to_string()))
    }

    pub(super) fn restored(
        &self,
        name: &str,
        landed: &File,
        checked: &File,
    ) -> Result<Option<File>, Hitch> {
        let Some(theirs) = self.written_between(landed, checked)? else {
            tisty_core::witness::note(
                tisty_core::witness::channel::CLOUD,
                "Drive moved past this write alone, and no bytes of another machine were among its revisions",
                &[],
            );
            return Ok(None);
        };
        let held = self.revision_held(&landed.id, &theirs)?;
        self.spent(EDIT);
        let restored = self.sent(name, held.path(), Some(landed), "")?;
        self.unpinned(&landed.id, &theirs);
        Ok(Some(restored))
    }

    pub(super) fn written_between(
        &self,
        landed: &File,
        checked: &File,
    ) -> Result<Option<String>, Hitch> {
        let url = format!("{}/files/{}/revisions", self.at.api, landed.id);
        let mut all = Vec::new();
        let mut next: Option<String> = None;
        loop {
            self.spent(LIST);
            let page: Revisions = json(self.call(Some(ASKED_WITHIN), |http| {
                let mut asked = http.get(&url).query(&[
                    (
                        "fields",
                        "nextPageToken,revisions(id,md5Checksum,modifiedTime)",
                    ),
                    ("pageSize", "1000"),
                ]);
                if let Some(token) = &next {
                    asked = asked.query(&[("pageToken", token.as_str())]);
                }
                asked
            })?)?;
            all.extend(page.revisions);
            match page.next_page_token {
                Some(token) => next = Some(token),
                None => break,
            }
        }
        Ok(between(&all, landed, checked))
    }

    // Only a pinned revision downloads and a pin never comes off, so once its bytes are back on top it is deleted.
    fn revision_held(&self, id: &str, revision: &str) -> Result<tempfile::NamedTempFile, Hitch> {
        self.pinned(id, revision)?;
        self.revision_brought(id, revision)
    }

    pub(super) fn unpinned(&self, id: &str, revision: &str) {
        let url = format!("{}/files/{id}/revisions/{revision}", self.at.api);
        self.spent(EDIT);
        if let Err(hitch) = self.call(Some(ASKED_WITHIN), |http| http.delete(&url)) {
            tisty_core::witness::warn(
                tisty_core::witness::channel::CLOUD,
                "a revision kept to bring back another machine's bytes could not be deleted",
                &[("why", tisty_core::witness::Fact::Why(format!("{hitch:?}")))],
            );
        }
    }

    fn revision_brought(&self, id: &str, revision: &str) -> Result<tempfile::NamedTempFile, Hitch> {
        let url = format!("{}/files/{id}/revisions/{revision}", self.at.api);
        self.spent(DOWNLOAD);
        let mut answer = self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("alt", "media")])
        })?;
        let mut held = tempfile::NamedTempFile::new()?;
        answer
            .copy_to(&mut held)
            .map_err(|e| Hitch::Unreachable(crate::told(&e.without_url())))?;
        held.flush()?;
        Ok(held)
    }

    pub(super) fn pinned(&self, id: &str, revision: &str) -> Result<(), Hitch> {
        let url = format!("{}/files/{id}/revisions/{revision}", self.at.api);
        self.spent(EDIT);
        let body = serde_json::json!({ "keepForever": true }).to_string();
        self.call(Some(ASKED_WITHIN), |http| {
            http.patch(&url)
                .query(&[("fields", "id")])
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.clone())
        })?;
        Ok(())
    }

    /// Sent to the bin rather than deleted, so bytes another machine wrote meanwhile can come back.
    pub(super) fn binned_unless_moved(&self, name: &str, checked: &File) -> Result<(), Hitch> {
        let binned = self.trashed(&checked.id, true)?;
        if same_bytes(checked, binned.md5_checksum.as_deref()) || !leapt(checked, &binned) {
            self.owned(&checked.id, None);
            return Ok(());
        }
        // Never noted as this machine's own: what comes back out of the bin is another machine's, and must come as its change.
        if let Err(failed) = self.trashed(&checked.id, false) {
            self.owe(Owed::Unbinned {
                id: checked.id.clone(),
            });
            return Err(failed);
        }
        Err(Hitch::Changed(name.to_string()))
    }

    pub(super) fn trashed(&self, id: &str, trashed: bool) -> Result<File, Hitch> {
        let url = format!("{}/files/{id}", self.at.api);
        self.spent(EDIT);
        let body = serde_json::json!({ "trashed": trashed }).to_string();
        json(self.call(Some(ASKED_WITHIN), |http| {
            http.patch(&url)
                .query(&[("fields", FILE_FIELDS)])
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.clone())
        })?)
    }
}
