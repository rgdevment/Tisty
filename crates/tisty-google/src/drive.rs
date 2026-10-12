use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use md5::{Digest, Md5};
use reqwest::blocking::{Client as Http, RequestBuilder, Response};
use serde::Deserialize;
use tisty_carrier::{Changes, Costs, Expect, Hitch, Limits, Remote, Seen, Tokens, named_well};

use crate::{WAITED_UNTOLD, http, told, waited};

const ROOT: &str = "Tisty";
const FOLDER: &str = "application/vnd.google-apps.folder";
const FILE_FIELDS: &str = "id,name,parents,mimeType,size,md5Checksum,version,trashed";
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

pub struct Drive {
    tokens: Arc<Tokens>,
    http: Http,
    at: DriveAt,
    chunk: u64,
    root: Mutex<Option<String>>,
    known: Mutex<Option<Tree>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct File {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    parents: Vec<String>,
    #[serde(default)]
    mime_type: String,
    size: Option<String>,
    md5_checksum: Option<String>,
    version: Option<String>,
    #[serde(default)]
    trashed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Page {
    #[serde(default)]
    files: Vec<File>,
    next_page_token: Option<String>,
}

#[derive(Clone)]
struct Tree {
    folders: HashMap<String, String>,
    files: Vec<(String, File)>,
}

impl Tree {
    fn named(&self, name: &str) -> impl Iterator<Item = &File> {
        self.files
            .iter()
            .filter(move |(path, _)| path == name)
            .map(|(_, file)| file)
    }
}

fn seen(name: &str, file: &File) -> Seen {
    Seen {
        name: name.to_string(),
        bytes: file
            .size
            .as_deref()
            .and_then(|one| one.parse().ok())
            .unwrap_or(0),
        hash: file
            .md5_checksum
            .as_deref()
            .map(|one| format!("md5:{one}"))
            .unwrap_or_default(),
        revision: format!("{}.{}", file.id, file.version.as_deref().unwrap_or("0")),
    }
}

fn quoted(text: &str) -> String {
    text.replace('\\', "\\\\").replace('\'', "\\'")
}

fn within(name: &str, under: &str) -> bool {
    let under = under.trim_end_matches('/');
    under.is_empty() || name.starts_with(&format!("{under}/"))
}

impl Drive {
    pub fn new(tokens: Arc<Tokens>, at: DriveAt) -> Result<Self, Hitch> {
        Ok(Self {
            tokens,
            http: http()?,
            at,
            chunk: 8 * MIB,
            root: Mutex::new(None),
            known: Mutex::new(None),
        })
    }

    pub fn with_chunk(mut self, chunk: u64) -> Self {
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
        let about: About = json(self.call(|http| {
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

    fn call(&self, build: impl Fn(&Http) -> RequestBuilder) -> Result<Response, Hitch> {
        let mut renewed = false;
        loop {
            let access = self.tokens.access()?;
            let answer = build(&self.http)
                .bearer_auth(&access)
                .send()
                .map_err(|e| Hitch::Unreachable(told(&e.without_url())))?;
            if answer.status().as_u16() == 401 && !renewed {
                self.tokens.spoiled(&access);
                renewed = true;
                continue;
            }
            return judged(answer);
        }
    }

    fn root(&self) -> Result<String, Hitch> {
        if let Some(id) = self.root.lock().ok().and_then(|held| held.clone()) {
            return Ok(id);
        }
        let url = format!("{}/files", self.at.api);
        let asked = format!(
            "name = '{ROOT}' and mimeType = '{FOLDER}' and 'root' in parents and trashed = false"
        );
        let found: Page = json(self.call(|http| {
            http.get(&url).query(&[
                ("q", asked.as_str()),
                ("orderBy", "createdTime"),
                ("fields", "files(id)"),
                ("spaces", "drive"),
            ])
        })?)?;
        let id = match found.files.into_iter().next() {
            Some(one) => one.id,
            None => self.folder(ROOT, "root")?,
        };
        if let Ok(mut held) = self.root.lock() {
            *held = Some(id.clone());
        }
        Ok(id)
    }

    fn folder(&self, name: &str, parent: &str) -> Result<String, Hitch> {
        let url = format!("{}/files", self.at.api);
        let body = serde_json::json!({ "name": name, "mimeType": FOLDER, "parents": [parent] });
        let made: File = json(self.call(|http| {
            http.post(&url)
                .query(&[("fields", FILE_FIELDS)])
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_string())
        })?)?;
        Ok(made.id)
    }

    fn tree(&self) -> Result<Tree, Hitch> {
        let root = self.root()?;
        let url = format!("{}/files", self.at.api);
        let fields = format!("nextPageToken,files({FILE_FIELDS})");
        let mut all = Vec::new();
        let mut next: Option<String> = None;
        loop {
            let page: Page = json(self.call(|http| {
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
        let tree = planted(&root, all);
        if let Ok(mut held) = self.known.lock() {
            *held = Some(tree.clone());
        }
        Ok(tree)
    }

    fn known(&self) -> Result<Tree, Hitch> {
        match self.known.lock().ok().and_then(|held| held.clone()) {
            Some(tree) => Ok(tree),
            None => self.tree(),
        }
    }

    fn noted(&self, change: impl FnOnce(&mut Tree)) {
        if let Ok(mut held) = self.known.lock()
            && let Some(tree) = held.as_mut()
        {
            change(tree);
        }
    }

    // A file is judged as Google holds it now with one small ask, never a whole listing per file.
    fn current(&self, name: &str) -> Result<(Tree, Vec<File>), Hitch> {
        let tree = self.known()?;
        let mut copies: Vec<File> = tree.named(name).cloned().collect();
        match copies.first().map(|one| one.id.clone()) {
            Some(id) => match self.fresh(&id)? {
                Some(now) => copies[0] = now,
                None => {
                    copies.remove(0);
                }
            },
            None => copies = self.looked_up(&tree, name)?,
        }
        Ok((tree, copies))
    }

    fn fresh(&self, id: &str) -> Result<Option<File>, Hitch> {
        let url = format!("{}/files/{id}", self.at.api);
        match self.call(|http| http.get(&url).query(&[("fields", FILE_FIELDS)])) {
            Ok(answer) => {
                let file: File = json(answer)?;
                Ok((!file.trashed).then_some(file))
            }
            Err(Hitch::Missing(_)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    fn looked_up(&self, tree: &Tree, name: &str) -> Result<Vec<File>, Hitch> {
        let (parent, leaf) = match name.rsplit_once('/') {
            Some((folders, leaf)) => match tree.folders.get(folders) {
                Some(id) => (id.clone(), leaf),
                None => return Ok(Vec::new()),
            },
            None => (self.root()?, name),
        };
        let url = format!("{}/files", self.at.api);
        let asked = format!(
            "name = '{}' and '{parent}' in parents and trashed = false",
            quoted(leaf)
        );
        let fields = format!("files({FILE_FIELDS})");
        let page: Page = json(self.call(|http| {
            http.get(&url).query(&[
                ("q", asked.as_str()),
                ("orderBy", "createdTime"),
                ("fields", fields.as_str()),
                ("spaces", "drive"),
            ])
        })?)?;
        Ok(page.files)
    }

    fn parent_of(&self, tree: &Tree, name: &str) -> Result<String, Hitch> {
        let mut at = self.root()?;
        let mut path = String::new();
        let Some((folders, _)) = name.rsplit_once('/') else {
            return Ok(at);
        };
        for part in folders.split('/') {
            path = if path.is_empty() {
                part.to_string()
            } else {
                format!("{path}/{part}")
            };
            at = match tree.folders.get(&path) {
                Some(id) => id.clone(),
                None => {
                    let made = self.folder(part, &at)?;
                    self.noted(|tree| {
                        tree.folders.insert(path.clone(), made.clone());
                    });
                    made
                }
            };
        }
        Ok(at)
    }

    fn sent(
        &self,
        name: &str,
        from: &Path,
        file: Option<&File>,
        parent: &str,
    ) -> Result<File, Hitch> {
        let bytes = std::fs::metadata(from)?.len();
        let leaf = name.rsplit('/').next().unwrap_or(name);
        let meta = match file {
            Some(_) => serde_json::json!({}),
            None => serde_json::json!({ "name": leaf, "parents": [parent] }),
        };
        let (url, method) = match file {
            Some(one) => (
                format!("{}/files/{}", self.at.upload, one.id),
                reqwest::Method::PATCH,
            ),
            None => (format!("{}/files", self.at.upload), reqwest::Method::POST),
        };
        match bytes <= self.chunk {
            true => self.sent_whole(&url, method, &meta, from),
            false => self.sent_in_parts(&url, method, &meta, from, bytes),
        }
    }

    fn sent_whole(
        &self,
        url: &str,
        method: reqwest::Method,
        meta: &serde_json::Value,
        from: &Path,
    ) -> Result<File, Hitch> {
        const BOUNDARY: &str = "tisty-part";
        let mut body = format!(
            "--{BOUNDARY}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n{meta}\r\n--{BOUNDARY}\r\nContent-Type: application/octet-stream\r\n\r\n"
        )
        .into_bytes();
        body.extend(std::fs::read(from)?);
        body.extend(format!("\r\n--{BOUNDARY}--\r\n").into_bytes());
        json(self.call(|http| {
            http.request(method.clone(), url)
                .query(&[("uploadType", "multipart"), ("fields", FILE_FIELDS)])
                .header(
                    reqwest::header::CONTENT_TYPE,
                    format!("multipart/related; boundary={BOUNDARY}"),
                )
                .body(body.clone())
        })?)
    }

    // A session lives at Google, so a cut mid-way costs the part in flight, not the whole file.
    fn sent_in_parts(
        &self,
        url: &str,
        method: reqwest::Method,
        meta: &serde_json::Value,
        from: &Path,
        bytes: u64,
    ) -> Result<File, Hitch> {
        let opened = self.call(|http| {
            http.request(method.clone(), url)
                .query(&[("uploadType", "resumable"), ("fields", FILE_FIELDS)])
                .header("X-Upload-Content-Length", bytes.to_string())
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(meta.to_string())
        })?;
        let session = opened
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|one| one.to_str().ok())
            .ok_or_else(|| Hitch::Broke("Google opened no upload session".into()))?
            .to_string();
        let mut file = std::fs::File::open(from)?;
        let mut at = 0u64;
        loop {
            let take = self.chunk.min(bytes - at);
            let mut part = vec![0u8; usize::try_from(take).unwrap_or(usize::MAX)];
            file.read_exact(&mut part)?;
            let range = format!("bytes {at}-{}/{bytes}", at + take - 1);
            let answer = self
                .http
                .put(&session)
                .header(reqwest::header::CONTENT_RANGE, range)
                .body(part)
                .send()
                .map_err(|e| Hitch::Unreachable(told(&e.without_url())))?;
            at += take;
            match answer.status().as_u16() {
                308 if at < bytes => continue,
                200 | 201 => return json(answer),
                _ => {
                    return Err(judged(answer)
                        .err()
                        .unwrap_or_else(|| Hitch::Broke("the upload ended early".into())));
                }
            }
        }
    }
}

fn planted(root: &str, all: Vec<File>) -> Tree {
    let folder_of: HashMap<&str, (&str, Option<&str>)> = all
        .iter()
        .filter(|one| one.mime_type == FOLDER)
        .map(|one| {
            (
                one.id.as_str(),
                (one.name.as_str(), one.parents.first().map(String::as_str)),
            )
        })
        .collect();
    let mut folders = HashMap::new();
    let mut files = Vec::new();
    for one in &all {
        let Some(path) = path_of(
            &folder_of,
            root,
            one.parents.first().map(String::as_str),
            &one.name,
        ) else {
            continue;
        };
        match one.mime_type == FOLDER {
            true => {
                folders.entry(path).or_insert_with(|| one.id.clone());
            }
            false => files.push((path, one.clone())),
        }
    }
    Tree { folders, files }
}

// A path climbs through folders Tisty made until it meets its own root; one that never does is not ours.
fn path_of<'a>(
    folder_of: &HashMap<&'a str, (&'a str, Option<&'a str>)>,
    root: &str,
    mut parent: Option<&'a str>,
    leaf: &str,
) -> Option<String> {
    let mut parts = vec![leaf.to_string()];
    for _ in 0..64 {
        let id = parent?;
        if id == root {
            parts.reverse();
            return Some(parts.join("/"));
        }
        let (name, up) = folder_of.get(id)?;
        parts.push((*name).to_string());
        parent = *up;
    }
    None
}

fn json<T: serde::de::DeserializeOwned>(answer: Response) -> Result<T, Hitch> {
    serde_json::from_reader(answer)
        .map_err(|_| Hitch::Broke("Google Drive answered in a shape Tisty does not read".into()))
}

#[derive(Deserialize)]
struct Refusal {
    error: RefusalBody,
}

#[derive(Deserialize)]
struct RefusalBody {
    #[serde(default)]
    message: String,
    #[serde(default)]
    errors: Vec<Reasoned>,
}

#[derive(Deserialize)]
struct Reasoned {
    #[serde(default)]
    reason: String,
}

fn judged(answer: Response) -> Result<Response, Hitch> {
    let status = answer.status().as_u16();
    if answer.status().is_success() {
        return Ok(answer);
    }
    let wait = waited(answer.headers());
    let refusal = serde_json::from_reader::<_, Refusal>(answer)
        .ok()
        .map(|one| one.error);
    let reason = refusal
        .as_ref()
        .and_then(|one| one.errors.first())
        .map(|one| one.reason.as_str())
        .unwrap_or_default();
    let message = refusal
        .as_ref()
        .map(|one| one.message.clone())
        .unwrap_or_default();
    Err(match (status, reason) {
        (401, _) => Hitch::Lost,
        (403, "storageQuotaExceeded" | "quotaExceeded") => Hitch::Full,
        (403, "userRateLimitExceeded" | "rateLimitExceeded") | (429, _) => Hitch::Limited {
            wait: wait.unwrap_or(WAITED_UNTOLD),
        },
        (403, "insufficientPermissions" | "appNotAuthorizedToFile") => Hitch::Lost,
        (404, _) => Hitch::Missing(message),
        (412, _) => Hitch::Changed(message),
        (500.., _) => match wait {
            Some(wait) => Hitch::Limited { wait },
            None => Hitch::Unreachable(format!("Google Drive answered {status}")),
        },
        _ => Hitch::Broke(format!("Google Drive refused: {status} {reason} {message}")),
    })
}

impl Remote for Drive {
    fn limits(&self) -> Limits {
        Limits {
            provider: "Google Drive",
            units_a_day: 100_000,
            bytes_a_day: 5 * 1024 * MIB,
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
        let tree = self.tree()?;
        let mut shown: Vec<Seen> = Vec::new();
        for (path, file) in &tree.files {
            if within(path, under) && !shown.iter().any(|one| one.name == *path) {
                shown.push(seen(path, file));
            }
        }
        Ok(shown)
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        named_well(name)?;
        Ok(self.current(name)?.1.first().map(|file| seen(name, file)))
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        named_well(name)?;
        let file = self
            .current(name)?
            .1
            .into_iter()
            .next()
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        let url = format!("{}/files/{}", self.at.api, file.id);
        let mut answer = self
            .call(|http| {
                let asked = http.get(&url).query(&[("alt", "media")]);
                match from {
                    0 => asked,
                    from => asked.header(reqwest::header::RANGE, format!("bytes={from}-")),
                }
            })
            .map_err(|hitch| match hitch {
                Hitch::Missing(_) => Hitch::Missing(name.to_string()),
                other => other,
            })?;
        std::io::copy(&mut answer, into).map_err(|e| Hitch::Unreachable(e.to_string()))?;
        Ok(seen(name, &file))
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        named_well(name)?;
        let (tree, copies) = self.current(name)?;
        let current = copies.into_iter().next();
        let revision = current.as_ref().map(|one| seen(name, one).revision);
        match (&expect, &revision) {
            (Expect::Absent, Some(_)) => return Err(Hitch::Changed(name.to_string())),
            (Expect::Revision(_), None) => return Err(Hitch::Missing(name.to_string())),
            (Expect::Revision(wanted), Some(now)) if wanted != now => {
                return Err(Hitch::Changed(name.to_string()));
            }
            _ => {}
        }
        let parent = self.parent_of(&tree, name)?;
        let made = self.sent(name, from, current.as_ref(), &parent)?;
        let landed = seen(name, &made);
        self.noted(
            |tree| match tree.files.iter_mut().find(|(path, _)| path == name) {
                Some(slot) => slot.1 = made,
                None => tree.files.push((name.to_string(), made)),
            },
        );
        Ok(landed)
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        named_well(name)?;
        let (_, all) = self.current(name)?;
        let first = all
            .first()
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        if expect.is_some_and(|wanted| seen(name, first).revision != wanted) {
            return Err(Hitch::Changed(name.to_string()));
        }
        for one in &all {
            let url = format!("{}/files/{}", self.at.api, one.id);
            match self.call(|http| http.delete(&url)) {
                Ok(_) | Err(Hitch::Missing(_)) => {}
                Err(other) => return Err(other),
            }
        }
        self.noted(|tree| tree.files.retain(|(path, _)| path != name));
        Ok(())
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        let mut file = std::fs::File::open(local)?;
        let mut hasher = Md5::new();
        let mut block = vec![0u8; 64 * 1024];
        loop {
            let read = file.read(&mut block)?;
            if read == 0 {
                break;
            }
            hasher.update(&block[..read]);
        }
        let hex: String = hasher
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        Ok(format!("md5:{hex}"))
    }

    // Changes name a file by its id alone; with any at all, the whole listing says where each one is.
    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Start {
            start_page_token: String,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Moved {
            #[serde(default)]
            changes: Vec<serde_json::Value>,
            next_page_token: Option<String>,
            new_start_page_token: Option<String>,
        }
        let Some(since) = since else {
            let url = format!("{}/changes/startPageToken", self.at.api);
            let start: Start = json(self.call(|http| http.get(&url))?)?;
            return Ok(Changes::Whole {
                seen: self.list("")?,
                cursor: start.start_page_token,
            });
        };
        let url = format!("{}/changes", self.at.api);
        let mut token = since.to_string();
        let mut stirred = false;
        loop {
            let moved: Moved = json(self.call(|http| {
                http.get(&url).query(&[
                    ("pageToken", token.as_str()),
                    ("pageSize", "1000"),
                    ("spaces", "drive"),
                    ("fields", "nextPageToken,newStartPageToken,changes(fileId)"),
                ])
            })?)?;
            stirred |= !moved.changes.is_empty();
            match (moved.next_page_token, moved.new_start_page_token) {
                (Some(next), _) => token = next,
                (None, Some(cursor)) => {
                    return Ok(match stirred {
                        false => Changes::Since {
                            changed: Vec::new(),
                            gone: Vec::new(),
                            cursor,
                        },
                        true => Changes::Whole {
                            seen: self.list("")?,
                            cursor,
                        },
                    });
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

#[cfg(test)]
#[path = "drive_test.rs"]
mod tests;
