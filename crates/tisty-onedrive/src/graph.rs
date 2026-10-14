use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::blocking::Client as Http;
use serde::Deserialize;
use tisty_carrier::{Changes, Costs, Expect, Hitch, Limits, Remote, Seen, Tokens};

mod tree;
mod upload;
mod wire;

use tree::{DriveItem, ITEM_FIELDS, Tree, moved, within};
use wire::{given, json, judged, transfer_http};

use crate::{ASKED_WITHIN, names, unreachable};

const MIB: u64 = 1024 * 1024;
const GIB: u64 = 1024 * MIB;
// Thirty-two of the 320 KiB pieces an upload session takes, well below the 60 MiB it refuses.
const CHUNK: u64 = 10 * MIB;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphAt {
    pub api: String,
}

impl GraphAt {
    pub fn microsoft() -> Self {
        Self {
            api: "https://graph.microsoft.com/v1.0".into(),
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
    cursor: String,
}

enum Walked {
    Done(Vec<DriveItem>, String),
    /// The cursor is too old for OneDrive to say what changed since: everything is listed again.
    Gone,
}

pub struct OneDrive {
    tokens: Arc<Tokens>,
    http: Http,
    at: GraphAt,
    chunk: u64,
    root: Mutex<Option<String>>,
    known: Mutex<Option<Known>>,
    expected: Option<String>,
    vouched: AtomicBool,
}

impl OneDrive {
    pub fn new(tokens: Arc<Tokens>, at: GraphAt) -> Result<Self, Hitch> {
        Ok(Self {
            tokens,
            http: transfer_http()?,
            at,
            chunk: CHUNK,
            root: Mutex::new(None),
            known: Mutex::new(None),
            expected: None,
            vouched: AtomicBool::new(false),
        })
    }

    /// Reads and writes only once OneDrive says it is this account, so two accounts never mix.
    pub fn for_account(mut self, id: &str) -> Self {
        self.expected = Some(id.to_string());
        self
    }

    #[cfg(test)]
    fn with_chunk(mut self, chunk: u64) -> Self {
        self.chunk = chunk;
        self
    }

    pub fn account(&self) -> Result<Account, Hitch> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Me {
            id: String,
            #[serde(default)]
            mail: Option<String>,
            #[serde(default)]
            user_principal_name: Option<String>,
        }
        let url = format!("{}/me", self.at.api);
        let me: Me = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url)
                .query(&[("$select", "id,mail,userPrincipalName")])
        })?)?;
        let email = me
            .mail
            .filter(|one| !one.is_empty())
            .or(me.user_principal_name)
            .unwrap_or_default();
        Ok(Account { id: me.id, email })
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

    fn vouched_for(&self) -> Result<(), Hitch> {
        let Some(expected) = &self.expected else {
            return Ok(());
        };
        if !self.vouched.load(Ordering::Acquire) {
            self.verify(expected)?;
            self.vouched.store(true, Ordering::Release);
        }
        Ok(())
    }

    // The folder stays the app's own however the person renames or moves it, so it is asked for by what it is.
    fn root(&self) -> Result<String, Hitch> {
        if let Some(root) = self.root.lock().ok().and_then(|held| held.clone()) {
            return Ok(root);
        }
        let url = format!("{}/me/drive/special/approot", self.at.api);
        let root: DriveItem = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("$select", "id")])
        })?)?;
        if let Ok(mut held) = self.root.lock() {
            *held = Some(root.id.clone());
        }
        Ok(root.id)
    }

    fn walked(&self, from: &str) -> Result<Walked, Hitch> {
        #[derive(Deserialize)]
        struct Page {
            #[serde(default)]
            value: Vec<DriveItem>,
            #[serde(rename = "@odata.nextLink")]
            next: Option<String>,
            #[serde(rename = "@odata.deltaLink")]
            delta: Option<String>,
        }
        let mut link = from.to_string();
        let mut all = Vec::new();
        loop {
            self.ours(&link)?;
            let answer = self.sent(Some(ASKED_WITHIN), |http| http.get(&link))?;
            if answer.status().as_u16() == 410 {
                return Ok(Walked::Gone);
            }
            let page: Page = json(judged(answer)?)?;
            all.extend(page.value);
            match (page.next, page.delta) {
                (Some(next), _) => link = next,
                (None, Some(cursor)) => return Ok(Walked::Done(all, cursor)),
                (None, None) => {
                    return Err(Hitch::Broke(
                        "OneDrive gave no cursor to carry on from".into(),
                    ));
                }
            }
        }
    }

    // Asked twice at most: a cursor that lapses while the whole folder is being read is read once more from the start.
    fn enumerated(&self) -> Result<(), Hitch> {
        self.vouched_for()?;
        let root = self.root()?;
        let first = format!(
            "{}/me/drive/special/approot/delta?$select={ITEM_FIELDS}",
            self.at.api
        );
        for _ in 0..2 {
            if let Walked::Done(items, cursor) = self.walked(&first)? {
                let mut tree = Tree::under(&root);
                tree.applied(items);
                if let Ok(mut held) = self.known.lock() {
                    *held = Some(Known { tree, cursor });
                }
                return Ok(());
            }
        }
        Err(Hitch::Troubled(
            "OneDrive kept starting its listing over".into(),
        ))
    }

    fn shown(&self, under: &str) -> Vec<Seen> {
        self.known
            .lock()
            .ok()
            .and_then(|held| {
                held.as_ref().map(|known| {
                    known
                        .tree
                        .files()
                        .into_iter()
                        .filter(|(path, _)| within(path, under))
                        .map(|(_, seen)| seen)
                        .collect()
                })
            })
            .unwrap_or_default()
    }

    fn cursor(&self) -> Option<String> {
        self.known
            .lock()
            .ok()
            .and_then(|held| held.as_ref().map(|known| known.cursor.clone()))
    }

    // Applied to the tree already held, so a change elsewhere costs one ask and not the whole folder again.
    fn caught_up(&self, items: Vec<DriveItem>, cursor: String) -> Option<Changes> {
        let mut held = self.known.lock().ok()?;
        let known = held.as_mut()?;
        let before = known.tree.files();
        known.tree.applied(items);
        let (changed, gone) = moved(&before, &known.tree.files());
        known.cursor = cursor.clone();
        Some(Changes::Since {
            changed,
            gone,
            cursor,
        })
    }

    fn item(&self, name: &str, fields: Option<&str>) -> Result<Option<DriveItem>, Hitch> {
        let url = self.by_path(name, None)?;
        let answer = self.call(Some(ASKED_WITHIN), |http| {
            let asked = http.get(&url);
            match fields {
                Some(fields) => asked.query(&[("$select", fields)]),
                None => asked,
            }
        });
        match answer {
            Ok(answer) => Ok(Some(json(answer)?)),
            Err(Hitch::Missing(_)) => Ok(None),
            Err(other) => Err(other),
        }
    }

    fn item_url(&self, id: &str) -> Result<String, Hitch> {
        let mut at = url::Url::parse(&format!("{}/me/drive/items/", self.at.api))
            .map_err(|_| Hitch::Broke("not a OneDrive address".into()))?;
        at.path_segments_mut()
            .map_err(|_| Hitch::Broke("not a OneDrive address".into()))?
            .pop_if_empty()
            .push(id);
        Ok(at.into())
    }
}

impl Remote for OneDrive {
    fn limits(&self) -> Limits {
        Limits {
            provider: "OneDrive",
            units_a_day: tisty_carrier::UNITS_A_DAY,
            bytes_a_day: tisty_carrier::BYTES_A_DAY,
            poll_every: Duration::from_secs(30),
            chunk: self.chunk,
            page: 200,
            most_per_file: 250 * GIB,
            folds_case: true,
            costs: Costs {
                list: 2,
                fetch: 1,
                put: 2,
                delete: 2,
                about: 1,
                changes: 1,
                checks: 0,
            },
        }
    }

    fn list(&self, under: &str) -> Result<Vec<Seen>, Hitch> {
        self.enumerated()?;
        Ok(self.shown(under))
    }

    fn about(&self, name: &str) -> Result<Option<Seen>, Hitch> {
        names::fitting(name)?;
        Ok(self
            .item(name, Some(ITEM_FIELDS))?
            .filter(DriveItem::is_file)
            .map(|one| one.seen(name)))
    }

    fn fetch(&self, name: &str, from: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        names::fitting(name)?;
        let item = self
            .item(name, None)?
            .filter(DriveItem::is_file)
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        let link = item.download.clone().ok_or_else(|| {
            Hitch::Broke("OneDrive gave no address to bring the file from".into())
        })?;
        // The address carries its own permission, and the range is only taken there.
        let mut asked = self.http.get(&link).timeout(given(item.size.unwrap_or(0)));
        if from > 0 {
            asked = asked.header(reqwest::header::RANGE, format!("bytes={from}-"));
        }
        let mut answer = asked.send().map_err(unreachable)?;
        let mut skipped = match answer.status().as_u16() {
            206 => 0,
            // OneDrive may send the whole file when it cannot cut it, so what was already here is passed over.
            200 => from,
            _ => {
                return Err(match judged(answer) {
                    Err(Hitch::Missing(_)) => Hitch::Missing(name.to_string()),
                    Err(other) => other,
                    Ok(_) => Hitch::Broke("OneDrive answered a download oddly".into()),
                });
            }
        };
        let mut block = vec![0u8; 64 * 1024];
        loop {
            let read = answer.read(&mut block).map_err(|_| {
                Hitch::Unreachable(None, "the line was cut while OneDrive was sending".into())
            })?;
            if read == 0 {
                break;
            }
            let passed = usize::try_from(skipped).unwrap_or(usize::MAX).min(read);
            skipped -= passed as u64;
            into.write_all(&block[passed..read])
                .map_err(|e| Hitch::Broke(e.to_string()))?;
        }
        Ok(item.seen(name))
    }

    fn put(&self, name: &str, from: &Path, expect: Expect) -> Result<Seen, Hitch> {
        names::fitting(name)?;
        self.vouched_for()?;
        let bytes = std::fs::metadata(from)?.len();
        let landed = match bytes <= self.chunk {
            true => self.whole(name, from, &expect)?,
            false => self.in_parts(name, from, &expect, bytes)?,
        };
        Ok(landed.seen(name))
    }

    fn delete(&self, name: &str, expect: Option<&str>) -> Result<(), Hitch> {
        names::fitting(name)?;
        let item = self
            .item(name, Some(ITEM_FIELDS))?
            .ok_or_else(|| Hitch::Missing(name.to_string()))?;
        if expect.is_some_and(|wanted| item.etag.as_deref() != Some(wanted)) {
            return Err(Hitch::Changed(name.to_string()));
        }
        let url = self.item_url(&item.id)?;
        let answer = self.sent(Some(ASKED_WITHIN), |http| {
            let asked = http.delete(&url);
            match expect {
                Some(wanted) => asked.header(reqwest::header::IF_MATCH, wanted),
                None => asked,
            }
        })?;
        match answer.status().as_u16() {
            200 | 204 => Ok(()),
            404 => Err(Hitch::Missing(name.to_string())),
            412 => Err(Hitch::Changed(name.to_string())),
            _ => judged(answer).map(|_| ()),
        }
    }

    fn room(&self) -> Result<Option<u64>, Hitch> {
        #[derive(Deserialize)]
        struct Quota {
            remaining: Option<i64>,
        }
        #[derive(Deserialize)]
        struct Drive {
            quota: Option<Quota>,
        }
        self.vouched_for()?;
        let url = format!("{}/me/drive", self.at.api);
        let drive: Drive = json(self.call(Some(ASKED_WITHIN), |http| {
            http.get(&url).query(&[("$select", "quota")])
        })?)?;
        Ok(drive
            .quota
            .and_then(|quota| quota.remaining)
            .map(|left| u64::try_from(left).unwrap_or(0)))
    }

    fn hash_of(&self, local: &Path) -> std::io::Result<String> {
        crate::quickxor::of_file(local).map(|hash| format!("quickxor:{hash}"))
    }

    fn changes(&self, since: Option<&str>) -> Result<Changes, Hitch> {
        if let Some(since) = since
            && self.cursor().as_deref() == Some(since)
        {
            self.vouched_for()?;
            if let Walked::Done(items, cursor) = self.walked(since)?
                && let Some(caught_up) = self.caught_up(items, cursor)
            {
                return Ok(caught_up);
            }
        }
        self.enumerated()?;
        Ok(Changes::Whole {
            seen: self.shown(""),
            cursor: self.cursor().unwrap_or_default(),
        })
    }
}

#[cfg(test)]
#[path = "graph_test.rs"]
mod tests;
