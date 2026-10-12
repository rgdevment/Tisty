use std::collections::HashSet;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::blocking::Client as Http;
use serde::Deserialize;
use tisty_carrier::{Changes, Costs, Expect, Hitch, Limits, Remote, Seen, Tokens, named_well};

mod tree;
mod upload;
mod wire;

use tree::{FILE_FIELDS, FOLDER, File, Page, Tree, planted, roots_in, seen, split, within};
use wire::{given, json, quoted, transfer_http};

use crate::ASKED_WITHIN;

const ROOT: &str = "Tisty";
const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriveAt {
    pub api: String,
    pub upload: String,
}

impl DriveAt {
    pub fn google() -> Self {
        Self {
            api: "https://www.googleapis.com/drive/v3".into(),
            upload: "https://www.googleapis.com/upload/drive/v3".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub id: String,
    pub email: String,
}

struct Known {
    tree: Tree,
    cursor: Option<String>,
}

pub struct Drive {
    tokens: Arc<Tokens>,
    http: Http,
    at: DriveAt,
    chunk: u64,
    roots: Mutex<Option<Vec<String>>>,
    known: Mutex<Option<Known>>,
    // What this machine wrote or deleted, by id and version, so its own changes are never news.
    wrote: Mutex<HashSet<(String, Option<String>)>>,
}

impl Drive {
    pub fn new(tokens: Arc<Tokens>, at: DriveAt) -> Result<Self, Hitch> {
        Ok(Self {
            tokens,
            http: transfer_http()?,
            at,
            chunk: 8 * MIB,
            roots: Mutex::new(None),
            known: Mutex::new(None),
            wrote: Mutex::new(HashSet::new()),
        })
    }

    #[cfg(test)]
    fn with_chunk(mut self, chunk: u64) -> Self {
        self.chunk = chunk;
        self
    }

    pub fn account(&self) -> Result<Account, Hitch> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct User {
            email_address: String,
            permission_id: String,
        }
        #[derive(Deserialize)]
        struct About {
            user: User,
        }
        let url = format!("{}/about", self.at.api);
        let about: About = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url)
                .query(&[("fields", "user(emailAddress,permissionId)")])
        })?)?;
        Ok(Account {
            id: about.user.permission_id,
            email: about.user.email_address,
        })
    }

    // The store belongs to the account it was chosen with; another one is refused before anything mixes.
    pub fn verify(&self, expected: &str) -> Result<Account, Hitch> {
        let account = self.account()?;
        match account.id == expected {
            true => Ok(account),
            false => Err(Hitch::Elsewhere {
                found: account.email,
            }),
        }
    }

    fn root_known(&self) -> Result<Option<String>, Hitch> {
        self.with_known(|_| ())?;
        Ok(self
            .roots
            .lock()
            .ok()
            .and_then(|held| held.as_ref().and_then(|roots| roots.first().cloned())))
    }

    // Made only when something is written, never while reading, so a lagging search can never split the store.
    fn root_made(&self) -> Result<String, Hitch> {
        if let Some(root) = self.root_known()? {
            return Ok(root);
        }
        let made = self.folder(ROOT, "root")?;
        if let Ok(mut held) = self.roots.lock() {
            *held = Some(vec![made.clone()]);
        }
        Ok(made)
    }

    fn folder(&self, name: &str, parent: &str) -> Result<String, Hitch> {
        let url = format!("{}/files", self.at.api);
        let body = serde_json::json!({ "name": name, "mimeType": FOLDER, "parents": [parent] });
        let made: File = json(self.call(Some(ASKED_WITHIN), |http| {
            http.post(&url)
                .query(&[("fields", FILE_FIELDS)])
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_string())
        })?)?;
        self.owned(
            &made.id,
            Some(made.version.clone().unwrap_or_else(|| "0".into())),
        );
        Ok(made.id)
    }

    fn listed(&self, cursor: Option<String>) -> Result<(), Hitch> {
        let url = format!("{}/files", self.at.api);
        let fields = format!("nextPageToken,files({FILE_FIELDS})");
        let mut all = Vec::new();
        let mut next: Option<String> = None;
        loop {
            let page: Page = json(self.call(Some(ASKED_WITHIN), |http| {
                let mut asked = http.get(&url).query(&[
                    ("q", "trashed = false"),
                    ("orderBy", "createdTime"),
                    ("pageSize", "1000"),
                    ("fields", fields.as_str()),
                    ("spaces", "drive"),
                ]);
                if let Some(token) = &next {
                    asked = asked.query(&[("pageToken", token.as_str())]);
                }
                asked
            })?)?;
            all.extend(page.files);
            match page.next_page_token {
                Some(token) => next = Some(token),
                None => break,
            }
        }
        let roots = roots_in(ROOT, &all);
        if roots.is_empty() && self.binned()? {
            return Err(Hitch::Broke(
                "the Tisty folder in Google Drive is in the bin; take it out, and nothing here was changed".into(),
            ));
        }
        if let Ok(mut held) = self.roots.lock() {
            *held = Some(roots.clone());
        }
        if let Ok(mut held) = self.known.lock() {
            *held = Some(Known {
                tree: planted(&roots, all),
                cursor,
            });
        }
        Ok(())
    }

    // A store whose folder went to the bin is stopped, never read as an empty drive.
    fn binned(&self) -> Result<bool, Hitch> {
        let url = format!("{}/files", self.at.api);
        let asked = format!("name = '{ROOT}' and mimeType = '{FOLDER}' and trashed = true");
        let found: Page = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[
                ("q", asked.as_str()),
                ("fields", "files(id)"),
                ("spaces", "drive"),
            ])
        })?)?;
        Ok(!found.files.is_empty())
    }

    fn with_known<T>(&self, read: impl FnOnce(&Tree) -> T) -> Result<T, Hitch> {
        if self.known.lock().map_or(true, |held| held.is_none()) {
            self.listed(None)?;
        }
        let held = self
            .known
            .lock()
            .map_err(|_| Hitch::Broke("the listing was left half written".into()))?;
        let known = held
            .as_ref()
            .ok_or_else(|| Hitch::Broke("the listing went missing".into()))?;
        Ok(read(&known.tree))
    }

    fn noted(&self, change: impl FnOnce(&mut Tree)) {
        if let Ok(mut held) = self.known.lock()
            && let Some(known) = held.as_mut()
        {
            change(&mut known.tree);
        }
    }

    // A file is judged as Google holds it now with one small ask, never a whole listing per file.
    fn current(&self, name: &str) -> Result<Vec<File>, Hitch> {
        let mut copies =
            self.with_known(|tree| tree.files.get(name).cloned().unwrap_or_default())?;
        if let Some(first) = copies.first()
            && let Some(now) = self.fresh(first, name)?
        {
            copies[0] = now;
            return Ok(copies);
        }
        if self.split_store() {
            let cursor = self
                .known
                .lock()
                .ok()
                .and_then(|held| held.as_ref().and_then(|known| known.cursor.clone()));
            self.listed(cursor)?;
            return self.with_known(|tree| tree.files.get(name).cloned().unwrap_or_default());
        }
        let found = self.looked_up(name)?;
        self.noted(|tree| match found.is_empty() {
            true => {
                tree.files.remove(name);
            }
            false => {
                tree.files.insert(name.to_string(), found.clone());
            }
        });
        Ok(found)
    }

    // A name search walks one folder, so a store split across two roots is read whole again instead.
    fn split_store(&self) -> bool {
        self.roots
            .lock()
            .ok()
            .and_then(|held| held.as_ref().map(|roots| roots.len() > 1))
            .unwrap_or(false)
    }

    fn fresh(&self, file: &File, name: &str) -> Result<Option<File>, Hitch> {
        let url = format!("{}/files/{}", self.at.api, file.id);
        let now: File = match self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("fields", FILE_FIELDS)])
        }) {
            Ok(answer) => json(answer)?,
            Err(Hitch::Missing(_)) => return Ok(None),
            Err(other) => return Err(other),
        };
        let (_, leaf) = split(name);
        let still = !now.trashed && now.name == leaf && now.parents.first() == file.parents.first();
        Ok(still.then_some(now))
    }

    fn looked_up(&self, name: &str) -> Result<Vec<File>, Hitch> {
        let (folders, leaf) = split(name);
        let Some(parent) = self.folder_id(folders, false)? else {
            return Ok(Vec::new());
        };
        let url = format!("{}/files", self.at.api);
        let asked = format!(
            "name = '{}' and '{parent}' in parents and trashed = false",
            quoted(leaf)
        );
        let fields = format!("files({FILE_FIELDS})");
        let page: Page = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[
                ("q", asked.as_str()),
                ("orderBy", "createdTime"),
                ("fields", fields.as_str()),
                ("spaces", "drive"),
            ])
        })?)?;
        Ok(page
            .files
            .into_iter()
            .filter(|one| one.mime_type != FOLDER)
            .collect())
    }

    fn folder_id(&self, path: Option<&str>, making: bool) -> Result<Option<String>, Hitch> {
        let found = match making {
            true => Some(self.root_made()?),
            false => self.root_known()?,
        };
        let Some(mut at) = found else {
            return Ok(None);
        };
        let Some(path) = path else {
            return Ok(Some(at));
        };
        let mut walked = String::new();
        for part in path.split('/') {
            walked = match walked.is_empty() {
                true => part.to_string(),
                false => format!("{walked}/{part}"),
            };
            if let Some(id) = self.with_known(|tree| tree.folders.get(&walked).cloned())? {
                at = id;
                continue;
            }
            let found = match self.child_folder(&at, part)? {
                Some(id) => id,
                None if making => self.folder(part, &at)?,
                None => return Ok(None),
            };
            self.noted(|tree| {
                tree.folders.insert(walked.clone(), found.clone());
            });
            at = found;
        }
        Ok(Some(at))
    }

    fn child_folder(&self, parent: &str, name: &str) -> Result<Option<String>, Hitch> {
        let url = format!("{}/files", self.at.api);
        let asked = format!(
            "name = '{}' and mimeType = '{FOLDER}' and '{parent}' in parents and trashed = false",
            quoted(name)
        );
        let page: Page = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[
                ("q", asked.as_str()),
                ("orderBy", "createdTime"),
                ("fields", "files(id)"),
                ("spaces", "drive"),
            ])
        })?)?;
        Ok(page.files.into_iter().next().map(|one| one.id))
    }

    fn owned(&self, id: &str, version: Option<String>) {
        if let Ok(mut held) = self.wrote.lock() {
            held.insert((id.to_string(), version));
        }
    }

    fn shown(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.with_known(|tree| {
            tree.files
                .iter()
                .filter(|(path, _)| within(path, under))
                .filter_map(|(path, copies)| copies.first().map(|first| seen(path, first)))
                .collect()
        })
    }

    fn stirred(&self, since: &str) -> Result<(bool, String), Hitch> {
        #[derive(Deserialize)]
        struct Changed {
            #[serde(rename = "fileId")]
            file_id: String,
            #[serde(default)]
            removed: bool,
            file: Option<File>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Moved {
            #[serde(default)]
            changes: Vec<Changed>,
            next_page_token: Option<String>,
            new_start_page_token: Option<String>,
        }
        let url = format!("{}/changes", self.at.api);
        let mut token = since.to_string();
        let mut all = Vec::new();
        loop {
            let moved: Moved = json(self.call(Some(ASKED_WITHIN), |http| {
                http.get(&url).query(&[
                    ("pageToken", token.as_str()),
                    ("pageSize", "1000"),
                    ("spaces", "drive"),
                    ("fields", "nextPageToken,newStartPageToken,changes(fileId,removed,file(id,version,trashed))"),
                ])
            })?)?;
            all.extend(moved.changes);
            match (moved.next_page_token, moved.new_start_page_token) {
                (Some(next), _) => token = next,
                (None, Some(cursor)) => {
                    let wrote = self
                        .wrote
                        .lock()
                        .map(|held| held.clone())
                        .unwrap_or_default();
                    let ours = all.iter().all(|one| {
                        let version = match (&one.file, one.removed) {
                            (Some(file), false) if !file.trashed => {
                                Some(file.version.clone().unwrap_or_else(|| "0".into()))
                            }
                            _ => None,
                        };
                        wrote.contains(&(one.file_id.clone(), version))
                    });
                    return Ok((!ours, cursor));
                }
                (None, None) => {
                    return Err(Hitch::Broke(
                        "Google Drive gave no cursor to carry on from".into(),
                    ));
                }
            }
        }
    }
}

impl Remote for Drive {
    fn limits(&self) -> Limits {
        Limits {
            provider: "Google Drive",
            units_a_day: tisty_carrier::UNITS_A_DAY,
            bytes_a_day: tisty_carrier::BYTES_A_DAY,
            poll_every: Duration::from_secs(300),
            chunk: self.chunk,
            page: 1000,
            most_per_file: 5 * 1024 * 1024 * MIB,
            folds_case: false,
            costs: Costs {
                list: 100,
                fetch: 200,
                put: 50,
                delete: 50,
                about: 5,
                changes: 100,
            },
        }
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.listed(None)?;
        self.shown(under)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        named_well(name)?;
        Ok(self.current(name)?.first().map(|file| seen(name, file)))
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        named_well(name)?;
        let file = self
            .current(name)?
            .into_iter()
            .next()
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        let url = format!("{}/files/{}", self.at.api, file.id);
        let mut answer = self
            .call(
                Some(given(
                    file.size
                        .as_deref()
                        .and_then(|one| one.parse().ok())
                        .unwrap_or(0),
                )),
                |http| {
                    let asked = http.get(&url).query(&[("alt", "media")]);
                    match from {
                        0 => asked,
                        from => asked.header(reqwest::header::RANGE, format!("bytes={from}-")),
                    }
                },
            )
            .map_err(|hitch| match hitch {
                Hitch::Missing(_) => Hitch::Missing(name.to_string()),
                other => other,
            })?;
        let mut block = vec![0u8; 64 * 1024];
        loop {
            let read = answer
                .read(&mut block)
                .map_err(|e| Hitch::Unreachable(e.to_string()))?;
            if read == 0 {
                break;
            }
            into.write_all(&block[..read])
                .map_err(|e| Hitch::Broke(e.to_string()))?;
        }
        Ok(seen(name, &file))
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        named_well(name)?;
        let copies = self.current(name)?;
        let current = copies.first();
        let revision = current.map(|one| seen(name, one).revision);
        match (&expect, &revision) {
            (Expect::Absent, Some(_)) => return Err(Hitch::Changed(name.to_string())),
            (Expect::Revision(_), None) => return Err(Hitch::Missing(name.to_string())),
            (Expect::Revision(wanted), Some(now)) if wanted != now => {
                return Err(Hitch::Changed(name.to_string()));
            }
            _ => {}
        }
        let parent = match current {
            Some(_) => String::new(),
            None => self
                .folder_id(split(name).0, true)?
                .ok_or_else(|| Hitch::Broke("a folder could not be made".into()))?,
        };
        let made = self.sent(name, from, current, &parent)?;
        self.owned(
            &made.id,
            Some(made.version.clone().unwrap_or_else(|| "0".into())),
        );
        let landed = seen(name, &made);
        self.noted(|tree| {
            let copies = tree.files.entry(name.to_string()).or_default();
            match copies.first_mut() {
                Some(first) if current.is_some() => *first = made,
                _ => copies.insert(0, made),
            }
        });
        Ok(landed)
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        named_well(name)?;
        let all = self.current(name)?;
        let first = all
            .first()
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        if expect.is_some_and(|wanted| seen(name, first).revision != wanted) {
            return Err(Hitch::Changed(name.to_string()));
        }
        for one in &all {
            let url = format!("{}/files/{}", self.at.api, one.id);
            match self.call(Some(ASKED_WITHIN), |http| http.delete(&url)) {
                Ok(_) | Err(Hitch::Missing(_)) => self.owned(&one.id, None),
                Err(other) => return Err(other),
            }
        }
        self.noted(|tree| {
            tree.files.remove(name);
        });
        Ok(())
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        tisty_core::attach::hashed(local)
            .map(|(hex, _)| format!("sha256:{hex}"))
            .map_err(std::io::Error::other)
    }

    // Changes name a file by its id alone; with any not this machine's own, the whole listing says where each one is.
    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Start {
            start_page_token: String,
        }
        let Some(since) = since else {
            let url = format!("{}/changes/startPageToken", self.at.api);
            let start: Start = json(self.call(Some(ASKED_WITHIN), |http| http.get(&url))?)?;
            self.listed(Some(start.start_page_token.clone()))?;
            return Ok(Changes::Whole {
                seen: self.shown("")?,
                cursor: start.start_page_token,
            });
        };
        let (stirred, cursor) = self.stirred(since)?;
        if let Ok(mut held) = self.wrote.lock() {
            held.clear();
        }
        let followed = self.known.lock().ok().is_some_and(|held| {
            held.as_ref()
                .is_some_and(|known| known.cursor.as_deref() == Some(since))
        });
        if stirred || !followed {
            self.listed(Some(cursor.clone()))?;
        } else if let Ok(mut held) = self.known.lock()
            && let Some(known) = held.as_mut()
        {
            known.cursor = Some(cursor.clone());
        }
        Ok(match stirred {
            false => Changes::Since {
                changed: Vec::new(),
                gone: Vec::new(),
                cursor,
            },
            true => Changes::Whole {
                seen: self.shown("")?,
                cursor,
            },
        })
    }
}

#[cfg(test)]
#[path = "drive_test.rs"]
mod tests;
