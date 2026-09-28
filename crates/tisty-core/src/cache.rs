use std::path::Path;

use rusqlite::Connection;

use crate::{
    Result, State,
    event::Event,
    store,
    witness::{self, Fact, channel},
};

/// Tied to the event schema: an older build then misses the cache and meets the version guard.
const SCHEMA: i64 = crate::event::SCHEMA_VERSION as i64 + 6;

pub struct Cache {
    db: Connection,
}

impl Cache {
    pub fn open(cache_dir: &Path) -> Result<Option<Self>> {
        if let Err(e) = std::fs::create_dir_all(cache_dir) {
            witness::warn(
                channel::CACHE,
                "cache folder unusable",
                &[
                    ("at", Fact::Path(cache_dir.to_path_buf())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            return Ok(None);
        }
        let at = cache_dir.join("read.db");
        let db = match Connection::open(&at) {
            Ok(db) => db,
            Err(e) => {
                witness::warn(
                    channel::CACHE,
                    "cache unopenable",
                    &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                );
                return Ok(None);
            }
        };
        // A schema that moved is a cache that is thrown away whole, tables included: `IF NOT
        // EXISTS` keeps an old table's columns, and a write into it fails forever after.
        let held = db
            .query_row("SELECT value FROM meta WHERE key = 'schema'", [], |row| {
                row.get::<_, String>(0)
            })
            .ok();
        if held.is_some_and(|had| had != SCHEMA.to_string())
            && let Err(e) = db.execute_batch(
                "DROP TABLE IF EXISTS meta;
                 DROP TABLE IF EXISTS task;
                 DROP TABLE IF EXISTS task_body;
                 DROP TABLE IF EXISTS list;
                 DROP TABLE IF EXISTS folder;
                 DROP TABLE IF EXISTS doc;
                 DROP TABLE IF EXISTS tombstone;
                 DROP TABLE IF EXISTS paper;",
            )
        {
            witness::warn(
                channel::CACHE,
                "an older cache could not be cleared",
                &[("why", Fact::Why(e.to_string()))],
            );
            return Ok(None);
        }
        if let Err(e) = db.execute_batch(
            "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=NORMAL;
                 CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS task(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS task_body(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS list(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS folder(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS doc(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS tombstone(id TEXT PRIMARY KEY, source TEXT);
                 CREATE TABLE IF NOT EXISTS paper(
                     id TEXT PRIMARY KEY,
                     bytes INTEGER NOT NULL,
                     wrote INTEGER NOT NULL,
                     card TEXT NOT NULL,
                     flat TEXT NOT NULL DEFAULT '');
                 CREATE TABLE IF NOT EXISTS gist(
                     id TEXT PRIMARY KEY,
                     said TEXT NOT NULL);",
        ) {
            witness::warn(
                channel::CACHE,
                "cache schema refused",
                &[("why", Fact::Why(e.to_string()))],
            );
            return Ok(None);
        }
        Ok(Some(Self { db }))
    }

    pub fn load(&self, fingerprint: &str, bodies: bool) -> Option<State> {
        if self.meta("schema")? != SCHEMA.to_string() || self.meta("fingerprint")? != fingerprint {
            return None;
        }

        let mut state = State::default();
        state.signed = self
            .meta("signed")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.signed_before = self
            .meta("signed_before")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.devices = self
            .meta("devices")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.dropped = self
            .meta("dropped")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.shed = self
            .meta("shed")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.retired = self
            .meta("retired")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.agents = self
            .meta("agents")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.forebears = self
            .meta("forebears")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.assistants = self
            .meta("assistants")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.hosts = self
            .meta("hosts")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default();
        state.fill = if bodies {
            crate::state::Fill::Whole
        } else {
            crate::state::Fill::Summary
        };
        for (table, into) in [("task", true), ("list", false)] {
            let mut q = self.db.prepare(&format!("SELECT doc FROM {table}")).ok()?;
            let rows = q
                .query_map([], |r| r.get::<_, String>(0))
                .ok()?
                .filter_map(|r| r.ok());
            for doc in rows {
                if into {
                    let task: crate::Task = serde_json::from_str(&doc).ok()?;
                    if let Some(source) = &task.source {
                        state.sourced.insert(source.clone(), task.id);
                    }
                    state.tasks.insert(task.id, task);
                } else {
                    let list: crate::List = serde_json::from_str(&doc).ok()?;
                    state.lists.insert(list.id, list);
                }
            }
        }
        {
            let mut q = self.db.prepare("SELECT doc FROM folder").ok()?;
            let rows = q
                .query_map([], |r| r.get::<_, String>(0))
                .ok()?
                .filter_map(|r| r.ok());
            for doc in rows {
                let folder: crate::model::Folder = serde_json::from_str(&doc).ok()?;
                state.folders.insert(folder.id, folder);
            }
            let mut q = self.db.prepare("SELECT doc FROM doc").ok()?;
            let rows = q
                .query_map([], |r| r.get::<_, String>(0))
                .ok()?
                .filter_map(|r| r.ok());
            for doc in rows {
                let kept: crate::model::Kept = serde_json::from_str(&doc).ok()?;
                state.docs.insert(kept.id, kept);
            }
        }

        if bodies {
            let mut q = self.db.prepare("SELECT id, doc FROM task_body").ok()?;
            let rows = q
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .ok()?
                .filter_map(|r| r.ok());
            for (id, doc) in rows {
                let (Ok(id), Ok(body)) = (id.parse(), serde_json::from_str::<Body>(&doc)) else {
                    witness::warn(
                        channel::CACHE,
                        "a cached body could not be read, so the whole cache is built again",
                        &[
                            ("at", Fact::Id(id)),
                            ("bytes", Fact::Bytes(doc.len() as u64)),
                        ],
                    );
                    return None;
                };
                if let Some(task) = state.tasks.get_mut(&id) {
                    task.description = body.description;
                    task.log = body.log;
                    task.steps = body.steps;
                }
            }
        }

        let mut erased = self.db.prepare("SELECT id, source FROM tombstone").ok()?;
        let gone = erased
            .query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?))
            })
            .ok()?
            .filter_map(|r| r.ok());
        // A source filed again and erased again has two graves: the later one is the one the
        // log leaves it pointing at, and a task still alive under it wins over both.
        for (id, source) in gone {
            if let Ok(id) = id.parse::<ulid::Ulid>() {
                state.mark_erased(id);
                if let Some(source) = source
                    && state
                        .sourced
                        .get(&source)
                        .is_none_or(|held| state.is_erased(*held) && *held < id)
                {
                    state.sourced.insert(source, id);
                }
            }
        }
        Some(state)
    }

    pub fn store(&mut self, state: &State, fingerprint: &str) -> Result<()> {
        if !state.has_bodies() {
            return Ok(());
        }
        let tx = match self.db.transaction() {
            Ok(tx) => tx,
            Err(_) => return Ok(()),
        };
        let written = (|| -> rusqlite::Result<()> {
            tx.execute("DELETE FROM task", [])?;
            tx.execute("DELETE FROM task_body", [])?;
            tx.execute("DELETE FROM list", [])?;
            tx.execute("DELETE FROM folder", [])?;
            tx.execute("DELETE FROM doc", [])?;
            tx.execute("DELETE FROM tombstone", [])?;
            {
                let mut task = tx.prepare("INSERT INTO task VALUES (?,?)")?;
                let mut body = tx.prepare("INSERT INTO task_body VALUES (?,?)")?;
                for t in state.tasks.values() {
                    let (summary, detail) = split(t);
                    task.execute(rusqlite::params![t.id.to_string(), summary])?;
                    if let Some(detail) = detail {
                        body.execute(rusqlite::params![t.id.to_string(), detail])?;
                    }
                }
                let mut list = tx.prepare("INSERT INTO list VALUES (?,?)")?;
                for l in state.lists.values() {
                    let doc = serde_json::to_string(l).unwrap_or_default();
                    list.execute(rusqlite::params![l.id.to_string(), doc])?;
                }
                let mut folder = tx.prepare("INSERT INTO folder VALUES (?,?)")?;
                for f in state.folders.values() {
                    let doc = serde_json::to_string(f).unwrap_or_default();
                    folder.execute(rusqlite::params![f.id.to_string(), doc])?;
                }
                let mut kept = tx.prepare("INSERT INTO doc VALUES (?,?)")?;
                for d in state.docs.values() {
                    let doc = serde_json::to_string(d).unwrap_or_default();
                    kept.execute(rusqlite::params![d.id.to_string(), doc])?;
                }
            }
            {
                let mut gone = tx.prepare("INSERT INTO tombstone VALUES (?,?)")?;
                for id in state.erased() {
                    gone.execute(rusqlite::params![
                        id.to_string(),
                        source_of_the_grave(state, *id)
                    ])?;
                }
            }
            tx.execute(
                "INSERT OR REPLACE INTO meta VALUES ('schema', ?), ('fingerprint', ?), ('signed', ?), ('signed_before', ?), ('devices', ?), ('dropped', ?), ('retired', ?), ('shed', ?), ('agents', ?), ('assistants', ?), ('forebears', ?), ('hosts', ?)",
                rusqlite::params![
                    SCHEMA.to_string(),
                    fingerprint,
                    serde_json::to_string(&state.signed).unwrap_or_default(),
                    serde_json::to_string(&state.signed_before).unwrap_or_default(),
                    serde_json::to_string(&state.devices).unwrap_or_default(),
                    serde_json::to_string(&state.dropped).unwrap_or_default(),
                    serde_json::to_string(&state.retired).unwrap_or_default(),
                    serde_json::to_string(&state.shed).unwrap_or_default(),
                    serde_json::to_string(&state.agents).unwrap_or_default(),
                    serde_json::to_string(&state.assistants).unwrap_or_default(),
                    serde_json::to_string(&state.forebears).unwrap_or_default(),
                    serde_json::to_string(&state.hosts).unwrap_or_default(),
                ],
            )?;
            tx.commit()
        })();
        if let Err(e) = written {
            witness::warn(
                channel::CACHE,
                "cache not written",
                &[("why", Fact::Why(e.to_string()))],
            );
        }
        Ok(())
    }

    pub fn touch(&mut self, state: &State, entity: ulid::Ulid, fingerprint: &str) -> Result<()> {
        if !state.has_bodies() {
            self.invalidate();
            return Ok(());
        }
        // A cache thrown out stays thrown out: stamping the fingerprint for one row would
        // declare the whole of it current, and every row nobody touched since — an archived
        // document, a deleted list and its tombstone — would read back as it was.
        if self.meta("fingerprint").is_none_or(|one| one.is_empty()) {
            return Ok(());
        }
        let carried = (|| -> rusqlite::Result<()> {
            let id = entity.to_string();
            if let Some(folder) = state.folders.get(&entity) {
                let doc = serde_json::to_string(folder).unwrap_or_default();
                self.db.execute(
                    "INSERT OR REPLACE INTO folder VALUES (?,?)",
                    rusqlite::params![id, doc],
                )?;
                self.db.execute(
                    "INSERT OR REPLACE INTO meta VALUES ('fingerprint', ?)",
                    [fingerprint],
                )?;
                return Ok(());
            }
            if let Some(kept) = state.docs.get(&entity) {
                let doc = serde_json::to_string(kept).unwrap_or_default();
                self.db.execute(
                    "INSERT OR REPLACE INTO doc VALUES (?,?)",
                    rusqlite::params![id, doc],
                )?;
                self.db.execute(
                    "INSERT OR REPLACE INTO meta VALUES ('fingerprint', ?)",
                    [fingerprint],
                )?;
                return Ok(());
            }
            match (state.tasks.get(&entity), state.lists.get(&entity)) {
                (Some(task), _) => {
                    let (summary, detail) = split(task);
                    self.db.execute(
                        "INSERT OR REPLACE INTO task VALUES (?,?)",
                        rusqlite::params![id, summary],
                    )?;
                    match detail {
                        Some(detail) => self.db.execute(
                            "INSERT OR REPLACE INTO task_body VALUES (?,?)",
                            rusqlite::params![id, detail],
                        )?,
                        None => self
                            .db
                            .execute("DELETE FROM task_body WHERE id = ?", [&id])?,
                    };
                }
                (_, Some(list)) => {
                    let doc = serde_json::to_string(list).unwrap_or_default();
                    self.db.execute(
                        "INSERT OR REPLACE INTO list VALUES (?,?)",
                        rusqlite::params![id, doc],
                    )?;
                }
                _ => {
                    self.db.execute("DELETE FROM task WHERE id = ?", [&id])?;
                    self.db
                        .execute("DELETE FROM task_body WHERE id = ?", [&id])?;
                    self.db.execute("DELETE FROM list WHERE id = ?", [&id])?;
                    self.db.execute("DELETE FROM folder WHERE id = ?", [&id])?;
                    self.db.execute("DELETE FROM doc WHERE id = ?", [&id])?;
                    if state.is_erased(entity) {
                        let source = source_of_the_grave(state, entity);
                        if let Some(source) = &source {
                            self.db.execute(
                                "UPDATE tombstone SET source = NULL WHERE source = ?",
                                [source],
                            )?;
                        }
                        self.db.execute(
                            "INSERT OR REPLACE INTO tombstone VALUES (?,?)",
                            rusqlite::params![&id, source],
                        )?;
                    }
                }
            }
            self.db.execute(
                "INSERT OR REPLACE INTO meta VALUES ('fingerprint', ?)",
                [fingerprint],
            )?;
            Ok(())
        })();
        if let Err(e) = carried {
            witness::warn(
                channel::CACHE,
                "cache entry not carried",
                &[
                    ("id", Fact::Id(entity.to_string())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
        Ok(())
    }

    /// The card a document reads at, or nothing if the file has been written since it was
    /// worked out. Nobody is told a stale one: the caller reads the body again instead.
    pub fn card(&self, id: &str, stamp: (u64, u64)) -> Option<crate::docs::Card> {
        let (said, flat): (String, String) = self
            .db
            .query_row(
                "SELECT card, flat FROM paper WHERE id = ? AND bytes = ? AND wrote = ?",
                rusqlite::params![id, stamp.0 as i64, stamp.1 as i64],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .ok()?;
        let card: crate::docs::Card = serde_json::from_str(&said).ok()?;
        // A row written before the text was kept alongside would answer searches with silence.
        if flat.is_empty() && card.chars > 0 {
            return None;
        }
        Some(card)
    }

    pub fn note_card(&self, id: &str, stamp: (u64, u64), card: &crate::docs::Card, flat: &str) {
        let Ok(said) = serde_json::to_string(card) else {
            return;
        };
        if let Err(e) = self.db.execute(
            "INSERT OR REPLACE INTO paper VALUES (?, ?, ?, ?, ?)",
            rusqlite::params![id, stamp.0 as i64, stamp.1 as i64, said, flat],
        ) {
            // Nothing is lost when this fails — the body is read again — and the window holding
            // the database is the ordinary reason, so it is not the person's to read about.
            witness::trace(
                channel::CACHE,
                "what a document holds could not be remembered",
                &[
                    ("id", Fact::Id(id.into())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
        }
    }

    /// Which documents hold every one of these words, answered without opening a single file.
    /// The text was folded when it was kept, so the terms are matched the same way.
    pub fn holding(&self, terms: &[String]) -> Option<Vec<String>> {
        if terms.is_empty() {
            return Some(Vec::new());
        }
        let where_all = terms
            .iter()
            .map(|_| "flat LIKE '%' || ? || '%'")
            .collect::<Vec<_>>()
            .join(" AND ");
        let mut asked = self
            .db
            .prepare(&format!("SELECT id FROM paper WHERE {where_all}"))
            .ok()?;
        let found = asked
            .query_map(rusqlite::params_from_iter(terms.iter()), |row| row.get(0))
            .ok()?
            .filter_map(std::result::Result::ok)
            .collect();
        Some(found)
    }

    /// Unlike a card, this was written rather than worked out, so it is not thrown away when the
    /// body moves: it is handed back with the print it was written against, and read as old.
    pub fn gist(&self, id: &str) -> Option<crate::docs::Gist> {
        let said: String = self
            .db
            .query_row(
                "SELECT said FROM gist WHERE id = ?",
                rusqlite::params![id],
                |row| row.get(0),
            )
            .ok()?;
        serde_json::from_str(&said).ok()
    }

    pub fn note_gist(&self, id: &str, gist: &crate::docs::Gist) -> bool {
        let Ok(said) = serde_json::to_string(gist) else {
            return false;
        };
        match self.db.execute(
            "INSERT OR REPLACE INTO gist VALUES (?, ?)",
            rusqlite::params![id, said],
        ) {
            Ok(_) => true,
            Err(e) => {
                witness::warn(
                    channel::CACHE,
                    "what an agent wrote about a document could not be kept",
                    &[
                        ("id", Fact::Id(id.into())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
                false
            }
        }
    }

    /// A document deleted or renamed leaves its card behind, and nothing ever asks for it again.
    pub fn forget_cards(&self, kept: &std::collections::BTreeSet<String>) {
        let Ok(mut all) = self
            .db
            .prepare("SELECT id FROM paper UNION SELECT id FROM gist")
        else {
            return;
        };
        let Ok(found) = all.query_map([], |row| row.get::<_, String>(0)) else {
            return;
        };
        let gone: Vec<String> = found
            .filter_map(std::result::Result::ok)
            .filter(|one| !kept.contains(one))
            .collect();
        for one in gone {
            let _ = self
                .db
                .execute("DELETE FROM paper WHERE id = ?", rusqlite::params![one]);
            let _ = self
                .db
                .execute("DELETE FROM gist WHERE id = ?", rusqlite::params![one]);
        }
    }

    pub fn already(&self) -> crate::tidy::Already {
        self.meta("already")
            .and_then(|said| serde_json::from_str(&said).ok())
            .unwrap_or_default()
    }

    pub fn note_already(&self, done: &crate::tidy::Already) {
        let Ok(said) = serde_json::to_string(done) else {
            return;
        };
        if let Err(e) = self.db.execute(
            "INSERT OR REPLACE INTO meta VALUES ('already', ?)",
            rusqlite::params![said],
        ) {
            witness::warn(
                channel::CACHE,
                "what was already taken out could not be remembered",
                &[("why", Fact::Why(e.to_string()))],
            );
        }
    }

    pub fn invalidate(&mut self) {
        if let Err(e) = self
            .db
            .execute("DELETE FROM meta WHERE key = 'fingerprint'", [])
        {
            witness::warn(
                channel::CACHE,
                "cache not invalidated",
                &[("why", Fact::Why(e.to_string()))],
            );
        }
    }

    pub fn mark(&mut self, key: &str) {
        if self.meta("last_key").is_some_and(|had| had.as_str() >= key) {
            return;
        }
        let _ = self
            .db
            .execute("INSERT OR REPLACE INTO meta VALUES ('last_key', ?)", [key]);
    }

    fn meta(&self, key: &str) -> Option<String> {
        self.db
            .query_row("SELECT value FROM meta WHERE key = ?", [key], |r| r.get(0))
            .ok()
    }
}

fn split(task: &crate::Task) -> (String, Option<String>) {
    let mut summary = task.clone();
    let body = Body {
        description: summary.description.take(),
        log: std::mem::take(&mut summary.log),
        steps: std::mem::take(&mut summary.steps),
    };

    let detail = (body.description.is_some() || !body.log.is_empty() || !body.steps.is_empty())
        .then(|| serde_json::to_string(&body).unwrap_or_default());
    (serde_json::to_string(&summary).unwrap_or_default(), detail)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Body {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    log: Vec<crate::LogEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    steps: Vec<crate::Step>,
}

pub fn fingerprint(store_root: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    let Ok(devices) = std::fs::read_dir(store_root) else {
        return String::new();
    };

    for device in devices.filter_map(|e| e.ok()) {
        let Ok(files) = std::fs::read_dir(device.path()) else {
            continue;
        };
        for file in files.filter_map(|e| e.ok()) {
            let path = file.path();
            if path.extension().is_some_and(|e| e == "tisty")
                && let Ok(meta) = file.metadata()
            {
                parts.push(format!("{}:{}", path.display(), meta.len()));
            }
        }
    }
    parts.sort();
    parts.join("|")
}

pub fn audit(store_root: &Path, cache_dir: &Path) -> Result<Audit> {
    let truth = State::replay(&store::read_all(store_root)?);
    let Some(cache) = Cache::open(cache_dir)? else {
        return Ok(Audit::Unavailable);
    };

    let print = fingerprint(store_root);
    match cache.load(&print, true) {
        None => Ok(Audit::Stale { truth }),
        Some(held) if held == truth => Ok(Audit::Agrees { truth }),
        Some(held) => Ok(Audit::Diverged {
            tasks: (held.tasks.len(), truth.tasks.len()),
            lists: (held.lists.len(), truth.lists.len()),
            truth,
        }),
    }
}

pub enum Audit {
    Unavailable,
    Stale {
        truth: State,
    },
    Agrees {
        truth: State,
    },
    Diverged {
        tasks: (usize, usize),
        lists: (usize, usize),
        truth: State,
    },
}

impl Audit {
    pub fn state(&self) -> Option<&State> {
        match self {
            Audit::Unavailable => None,
            Audit::Stale { truth } | Audit::Agrees { truth } | Audit::Diverged { truth, .. } => {
                Some(truth)
            }
        }
    }
}

pub fn discard(cache_dir: &Path) -> Result<()> {
    if let Some(mut cache) = Cache::open(cache_dir)? {
        cache.invalidate();
    }
    Ok(())
}

fn reaches_pages(state: &State, id: &crate::model::DocId, d: &crate::event::Filed) -> bool {
    if d.page_of.is_some() {
        return true;
    }
    d.folder.is_some() && state.docs.values().any(|one| one.page_of == Some(*id))
}

pub fn advance(
    cache: Option<&mut Cache>,
    state: &State,
    events: &[crate::Event],
    store_root: &Path,
    overtaken: bool,
) -> String {
    let print = fingerprint(store_root);
    reached(cache, state, events, print, overtaken)
}

fn reached(
    cache: Option<&mut Cache>,
    state: &State,
    events: &[crate::Event],
    print: String,
    overtaken: bool,
) -> String {
    let Some(cache) = cache else { return print };

    if overtaken {
        cache.invalidate();
        return print;
    }

    if !state.has_bodies() {
        cache.invalidate();
        return print;
    }

    if events.iter().any(|e| {
        matches!(
            e.op,
            crate::Op::ListDelete { .. }
                | crate::Op::FolderDelete { .. }
                | crate::Op::DeviceJoin { .. }
                | crate::Op::DeviceHost { .. }
                | crate::Op::DeviceRemove { .. }
                | crate::Op::AttachRetire { .. }
                | crate::Op::Signed { .. }
                // These reach the pages of a document, and a row at a time cannot say so.
                | crate::Op::DocDelete { .. }
                | crate::Op::DocArchive { .. }
                | crate::Op::DocUnarchive { .. }
                | crate::Op::DocLock { .. }
                | crate::Op::DocUnlock { .. }
        ) || matches!(&e.op, crate::Op::DocMove { id, d } if reaches_pages(state, id, d))
    }) {
        cache.invalidate();
        return print;
    }

    // One transaction for the lot: a bulk of thousands would otherwise stamp the fingerprint
    // row by row, and a process cut halfway would leave a cache that calls itself current
    // while still holding tasks the log had already buried.
    let _ = cache.db.execute_batch("BEGIN");
    for event in events {
        if let Some(id) = event.entity_id() {
            let _ = cache.touch(state, id, &print);
        }
    }
    if cache.db.execute_batch("COMMIT").is_err() {
        let _ = cache.db.execute_batch("ROLLBACK");
        cache.invalidate();
        return print;
    }
    if let Some(onward) = highest(events) {
        cache.mark(&onward);
    }
    print
}

/// The tombstone keeps what the task was written from, so a cache read back knows the source
/// as the log does, and an assistant reading the same message is told it was let go.
fn source_of_the_grave(state: &State, id: ulid::Ulid) -> Option<String> {
    state
        .sourced
        .iter()
        .find(|(_, held)| **held == id)
        .map(|(source, _)| source.clone())
}

/// The log only ever grows, so a cache left behind by one file growing can be caught up from
/// that file's tail instead of replaying every event again.
fn grown(before: &str, now: &str) -> Option<(std::path::PathBuf, u64)> {
    if before.is_empty() || now.is_empty() {
        return None;
    }
    let was: Vec<&str> = before.split('|').collect();
    let is: Vec<&str> = now.split('|').collect();
    if was.len() != is.len() {
        return None;
    }
    let mut found = None;
    for (a, b) in was.iter().zip(is.iter()) {
        if a == b {
            continue;
        }
        if found.is_some() {
            return None;
        }
        let (at, was_len) = a.rsplit_once(':')?;
        let (also, is_len) = b.rsplit_once(':')?;
        let (was_len, is_len) = (was_len.parse::<u64>().ok()?, is_len.parse::<u64>().ok()?);
        if at != also || is_len <= was_len || was_len == 0 {
            return None;
        }
        found = Some((std::path::PathBuf::from(at), was_len));
    }
    found
}

/// Padded so that comparing the text compares the same way the tuple does.
fn keyed(event: &crate::Event) -> Option<String> {
    let (at, device, seq) = event.sort_key();
    let nanos = at.as_nanosecond();
    (nanos >= 0).then(|| format!("{nanos:039}\u{1}{}\u{1}{seq:020}", device.0))
}

fn highest(events: &[crate::Event]) -> Option<String> {
    events.iter().filter_map(keyed).max()
}

/// A tail that begins mid-line is not a tail: the file was rewritten, not appended to.
fn appended(at: &Path, from: u64) -> bool {
    use std::io::{Read, Seek};
    let Ok(mut file) = std::fs::File::open(at) else {
        return false;
    };
    if file.seek(std::io::SeekFrom::Start(from - 1)).is_err() {
        return false;
    }
    let mut byte = [0u8; 1];
    file.read_exact(&mut byte).is_ok() && byte[0] == b'\n'
}

fn caught_up(cache: &mut Cache, print: &str, bodies: bool) -> Option<State> {
    if !bodies {
        return None;
    }
    let before = cache.meta("fingerprint")?;
    let last = cache.meta("last_key")?;
    let (at, from) = grown(&before, print)?;
    if !at
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(store::is_segment)
    {
        return None;
    }
    if !appended(&at, from) {
        return None;
    }

    let mut state = cache.load(&before, bodies)?;
    let fresh = store::read_tail(&at, from).ok()?;
    if fresh.is_empty() {
        return None;
    }
    // Replaying sorts every event together. Applying a tail on top only lands in the same place
    // when nothing in it belongs before what the cache already holds.
    if fresh.iter().any(|one| keyed(one).is_none_or(|k| k <= last)) {
        return None;
    }

    for event in &fresh {
        state.apply(event);
    }
    reached(Some(cache), &state, &fresh, print.to_string(), false);
    Some(state)
}

pub fn project(store_root: &Path, cache_dir: &Path) -> Result<State> {
    projected(store_root, cache_dir, true)
}

pub fn summarised(store_root: &Path, cache_dir: &Path) -> Result<State> {
    projected(store_root, cache_dir, false)
}

fn projected(store_root: &Path, cache_dir: &Path, bodies: bool) -> Result<State> {
    let print = fingerprint(store_root);
    let mut cache = Cache::open(cache_dir)?;

    if let Some(cache) = &cache
        && let Some(state) = cache.load(&print, bodies)
    {
        return Ok(state);
    }

    if let Some(cache) = &mut cache
        && let Some(state) = caught_up(cache, &print, bodies)
    {
        return Ok(state);
    }

    let events: Vec<Event> = store::read_all(store_root)?;
    let state = State::replay(&events);

    if let Some(cache) = &mut cache {
        let _ = cache.store(&state, &print);
        if let Some(onward) = highest(&events) {
            cache.mark(&onward);
        }
        if !bodies && let Some(light) = cache.load(&print, false) {
            return Ok(light);
        }
    }
    Ok(state)
}

#[cfg(test)]
#[path = "cache_test.rs"]
mod tests;
