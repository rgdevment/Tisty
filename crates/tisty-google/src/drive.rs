use std::collections::{BTreeSet, HashSet};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use reqwest::blocking::Client as Http;
use serde::Deserialize;
use tisty_carrier::{Changes, Costs, Expect, Hitch, Limits, Remote, Seen, Tokens, named_well};

mod gap;
mod owed;
mod sessions;
mod tree;
mod upload;
mod wire;

use tree::{Changed, FILE_FIELDS, FOLDER, File, Page, Tree, planted, root_in, seen, split, within};
use wire::{given, json, quoted, transfer_http};

use crate::ASKED_WITHIN;

// Drive's hidden space for this app: Tisty's store never shows among the person's own files.
pub(super) const SPACE: &str = "appDataFolder";
// Google's units for what a verb's own cost does not cover (guides/limits).
pub(super) const READ: u64 = 5;
pub(super) const LIST: u64 = 100;
pub(super) const DOWNLOAD: u64 = 200;
pub(super) const EDIT: u64 = 50;
const MIB: u64 = 1024 * 1024;
const FRESH_FOR: Duration = Duration::from_secs(60);

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
    // Each cursor reached with the paths that moved to reach it, so one asked from an older cursor hears all of them again.
    passed: Vec<(String, BTreeSet<String>)>,
    current_at: Instant,
}

const PASSED_AT_MOST: usize = 64;

// Only an attachment's name says its bytes, so only its absence in a current tree is trusted without a search.
fn content_addressed(name: &str) -> bool {
    name.starts_with("attachments/")
}

impl Known {
    fn holds_up_to(&self, since: &str) -> bool {
        self.passed.iter().any(|(one, _)| one == since)
    }

    fn moved_to(&mut self, cursor: &str, touched: BTreeSet<String>) {
        self.cursor = Some(cursor.to_string());
        self.current_at = Instant::now();
        if self.passed.last().is_some_and(|(last, _)| last == cursor) {
            return;
        }
        self.passed.push((cursor.to_string(), touched));
        if self.passed.len() > PASSED_AT_MOST {
            self.passed.remove(0);
        }
    }

    // A quiet catch-up between rounds moves the cursor without a mark, so a day of them never pushes out the one the index holds.
    fn kept_at(&mut self, cursor: &str) {
        self.cursor = Some(cursor.to_string());
        self.current_at = Instant::now();
    }

    fn touched_since(&self, since: &str) -> BTreeSet<String> {
        self.passed
            .iter()
            .skip_while(|(one, _)| one != since)
            .skip(1)
            .flat_map(|(_, touched)| touched.iter().cloned())
            .collect()
    }
}

pub struct Drive {
    tokens: Arc<Tokens>,
    http: Http,
    at: DriveAt,
    chunk: u64,
    root: Mutex<Option<String>>,
    known: Mutex<Option<Known>>,
    // What this machine wrote or deleted, by id and version, so its own changes are never news.
    wrote: Mutex<HashSet<(String, Option<String>)>>,
    expected: Option<String>,
    vouched: std::sync::atomic::AtomicBool,
    owing: Option<std::path::PathBuf>,
    beyond: std::sync::atomic::AtomicU64,
}

impl Drive {
    pub fn new(tokens: Arc<Tokens>, at: DriveAt) -> Result<Self, Hitch> {
        Ok(Self {
            tokens,
            http: transfer_http()?,
            at,
            chunk: 8 * MIB,
            root: Mutex::new(None),
            known: Mutex::new(None),
            wrote: Mutex::new(HashSet::new()),
            expected: None,
            vouched: std::sync::atomic::AtomicBool::new(false),
            owing: None,
            beyond: std::sync::atomic::AtomicU64::new(0),
        })
    }

    /// Reads and writes only once Google says it is this account, so two accounts never mix.
    pub fn for_account(mut self, id: &str) -> Self {
        self.expected = Some(id.to_string());
        self
    }

    pub(super) fn spent(&self, units: u64) {
        self.beyond
            .fetch_add(units, std::sync::atomic::Ordering::Relaxed);
    }

    fn vouched_for(&self) -> Result<(), Hitch> {
        use std::sync::atomic::Ordering;
        let Some(expected) = &self.expected else {
            return Ok(());
        };
        if !self.vouched.load(Ordering::Acquire) {
            self.spent(READ);
            self.verify(expected)?;
            self.vouched.store(true, Ordering::Release);
        }
        Ok(())
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

    // Google's own name for the space stands for it until a listing says its id, which files there name as their parent.
    fn root(&self) -> Result<String, Hitch> {
        self.with_known(|_| ())?;
        Ok(self.roots().pop().unwrap_or_else(|| SPACE.to_string()))
    }

    fn roots(&self) -> Vec<String> {
        self.root
            .lock()
            .ok()
            .and_then(|held| held.clone())
            .into_iter()
            .collect()
    }

    fn space_id(&self) -> Result<String, Hitch> {
        let url = format!("{}/files/{SPACE}", self.at.api);
        self.spent(READ);
        let space: File = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("fields", "id")])
        })?)?;
        Ok(space.id)
    }

    fn folder(&self, name: &str, parent: &str) -> Result<String, Hitch> {
        let url = format!("{}/files", self.at.api);
        let body = serde_json::json!({ "name": name, "mimeType": FOLDER, "parents": [parent] });
        self.spent(EDIT);
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
        self.vouched_for()?;
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
                    ("spaces", SPACE),
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
        let root = match root_in(&all) {
            Ok(found) => found,
            Err(()) => Some(self.space_id()?),
        };
        if let Some(root) = root
            && let Ok(mut held) = self.root.lock()
        {
            *held = Some(root);
        }
        let roots = self.roots();
        if let Ok(mut held) = self.known.lock() {
            *held = Some(Known {
                tree: planted(&roots, all),
                passed: cursor
                    .iter()
                    .map(|one| (one.clone(), BTreeSet::new()))
                    .collect(),
                cursor,
                current_at: Instant::now(),
            });
        }
        Ok(())
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
        if copies.is_empty() && content_addressed(name) && self.kept_current()? {
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

    // Absence in a tree the changes feed caught up within a minute is trusted: a racing twin lands beside ours as after a search, and an attachment's holds the same bytes.
    fn kept_current(&self) -> Result<bool, Hitch> {
        let held = self.known.lock().ok().and_then(|held| {
            held.as_ref()
                .and_then(|known| Some((known.cursor.clone()?, known.current_at)))
        });
        let Some((cursor, at)) = held else {
            return Ok(false);
        };
        if at.elapsed() < FRESH_FOR {
            return Ok(true);
        }
        self.spent(LIST);
        let (theirs, next) = self.stirred(&cursor)?;
        let remembered = !theirs.is_empty();
        if self
            .caught_up(&cursor, &theirs, &next, remembered)
            .is_none()
        {
            self.listed(Some(next))?;
            self.laid_over(&theirs);
        }
        Ok(true)
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
        self.spent(LIST);
        let page: Page = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[
                ("q", asked.as_str()),
                ("orderBy", "createdTime"),
                ("fields", fields.as_str()),
                ("spaces", SPACE),
            ])
        })?)?;
        Ok(page
            .files
            .into_iter()
            .filter(|one| one.mime_type != FOLDER)
            .collect())
    }

    fn folder_id(&self, path: Option<&str>, making: bool) -> Result<Option<String>, Hitch> {
        let mut at = self.root()?;
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
            // Searched even in a current tree, so two machines making one folder at once still meet in the older one.
            let found = match self.child_folder(&at, part)? {
                Some(id) => id,
                None if making => self.folder(part, &at)?,
                None => return Ok(None),
            };
            self.noted(|tree| {
                tree.folders.insert(walked.clone(), found.clone());
                tree.folder_paths.insert(found.clone(), walked.clone());
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
        self.spent(LIST);
        let page: Page = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[
                ("q", asked.as_str()),
                ("orderBy", "createdTime"),
                ("fields", "files(id)"),
                ("spaces", SPACE),
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

    fn stirred(&self, since: &str) -> Result<(Vec<Changed>, String), Hitch> {
        self.vouched_for()?;
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Moved {
            #[serde(default)]
            changes: Vec<Changed>,
            next_page_token: Option<String>,
            new_start_page_token: Option<String>,
        }
        let url = format!("{}/changes", self.at.api);
        let fields =
            format!("nextPageToken,newStartPageToken,changes(fileId,removed,file({FILE_FIELDS}))");
        let mut token = since.to_string();
        let mut all = Vec::new();
        loop {
            let moved: Moved = json(self.call(Some(ASKED_WITHIN), |http| {
                http.get(&url).query(&[
                    ("pageToken", token.as_str()),
                    ("pageSize", "1000"),
                    ("spaces", SPACE),
                    ("fields", fields.as_str()),
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
                    all.retain(|one| {
                        let version = one
                            .standing()
                            .map(|file| file.version.clone().unwrap_or_else(|| "0".into()));
                        !wrote.contains(&(one.file_id.clone(), version))
                    });
                    return Ok((all, cursor));
                }
                (None, None) => {
                    return Err(Hitch::Broke(
                        "Google Drive gave no cursor to carry on from".into(),
                    ));
                }
            }
        }
    }

    // A listing can lag behind the changes that sent us to it, so what they say that it does not show yet is laid over it.
    fn laid_over(&self, theirs: &[Changed]) {
        let roots = self.roots();
        if roots.is_empty() {
            return;
        }
        let Ok(mut held) = self.known.lock() else {
            return;
        };
        let Some(known) = held.as_mut() else {
            return;
        };
        let listed: std::collections::HashMap<&str, u64> = known
            .tree
            .files
            .values()
            .flatten()
            .map(|one| (one.id.as_str(), gap::version(one)))
            .collect();
        let behind: Vec<Changed> = theirs
            .iter()
            .filter(
                |one| match (listed.get(one.file_id.as_str()), one.standing()) {
                    (Some(_), None) | (None, Some(_)) => true,
                    (Some(was), Some(now)) => gap::version(now) > *was,
                    (None, None) => false,
                },
            )
            .cloned()
            .collect();
        if behind.is_empty() {
            return;
        }
        if let Some((tree, _)) = tree::applied(&known.tree, &roots, &behind) {
            known.tree = tree;
        }
    }

    // Applied to the listing already held, so a change elsewhere costs one ask and not the whole drive again.
    fn caught_up(
        &self,
        since: &str,
        theirs: &[Changed],
        cursor: &str,
        remembered: bool,
    ) -> Option<Changes> {
        let roots = self.roots();
        if roots.is_empty() && !theirs.is_empty() {
            return None;
        }
        let mut held = self.known.lock().ok()?;
        let known = held.as_mut()?;
        let touched = match theirs.is_empty() {
            true => BTreeSet::new(),
            false => {
                let (tree, applied) = tree::applied(&known.tree, &roots, theirs)?;
                known.tree = tree;
                applied.changed.into_iter().chain(applied.gone).collect()
            }
        };
        match remembered {
            true => known.moved_to(cursor, touched),
            false => known.kept_at(cursor),
        }
        let (mut changed, mut gone) = (Vec::new(), Vec::new());
        for path in known.touched_since(since) {
            match known
                .tree
                .files
                .get(&path)
                .and_then(|copies| copies.first())
            {
                Some(file) => changed.push(seen(&path, file)),
                None => gone.push(path),
            }
        }
        Some(Changes::Since {
            changed,
            gone,
            cursor: cursor.to_string(),
        })
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
                checks: 5,
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
            let read = answer.read(&mut block).map_err(|_| {
                Hitch::Unreachable("the line was cut while Google Drive was sending".into())
            })?;
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
        if let Some(checked) = current
            && gap::leapt(checked, &made)
        {
            match self.handed_back(name, &made, checked) {
                // The bytes went up even though the write is refused, and the meter charges a refusal only its look.
                Err(Hitch::Changed(refused)) => {
                    let weighs = std::fs::metadata(from).map_or(0, |one| one.len());
                    self.spent(EDIT.saturating_mul(self.limits().requests_to_put(weighs)));
                    return Err(Hitch::Changed(refused));
                }
                other => other?,
            }
        }
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
            tree::ordered(copies);
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
        // The space has no bin, so the revision just checked is the last word before the bytes go for good.
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

    fn room(&self) -> Result<Option<u64>, Hitch> {
        #[derive(Deserialize)]
        struct Quota {
            limit: Option<String>,
            usage: Option<String>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct About {
            storage_quota: Quota,
        }
        self.vouched_for()?;
        let url = format!("{}/about", self.at.api);
        self.spent(READ);
        let about: About = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url)
                .query(&[("fields", "storageQuota(limit,usage)")])
        })?)?;
        let bytes = |one: Option<String>| one.and_then(|one| one.parse::<u64>().ok());
        Ok(bytes(about.storage_quota.limit)
            .map(|limit| limit.saturating_sub(bytes(about.storage_quota.usage).unwrap_or(0))))
    }

    fn spent_beyond(&self) -> u64 {
        self.beyond.swap(0, std::sync::atomic::Ordering::Relaxed)
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
        self.repaid();
        let Some(since) = since else {
            let url = format!("{}/changes/startPageToken", self.at.api);
            let start: Start = json(self.call(Some(ASKED_WITHIN), |http| http.get(&url))?)?;
            self.listed(Some(start.start_page_token.clone()))?;
            return Ok(Changes::Whole {
                seen: self.shown("")?,
                cursor: start.start_page_token,
            });
        };
        // Asked onward from where the listing stands, never again from the older cursor, so nothing is applied twice.
        let followed = self.known.lock().ok().and_then(|held| {
            held.as_ref()
                .filter(|known| known.holds_up_to(since))
                .and_then(|known| known.cursor.clone())
        });
        let (theirs, cursor) = self.stirred(followed.as_deref().unwrap_or(since))?;
        if let Ok(mut held) = self.wrote.lock() {
            held.clear();
        }
        if followed.is_some()
            && let Some(caught_up) = self.caught_up(since, &theirs, &cursor, true)
        {
            return Ok(caught_up);
        }
        self.listed(Some(cursor.clone()))?;
        self.laid_over(&theirs);
        Ok(match theirs.is_empty() {
            true => Changes::Since {
                changed: Vec::new(),
                gone: Vec::new(),
                cursor,
            },
            false => Changes::Whole {
                seen: self.shown("")?,
                cursor,
            },
        })
    }
}

#[cfg(test)]
#[path = "drive_test.rs"]
mod tests;
