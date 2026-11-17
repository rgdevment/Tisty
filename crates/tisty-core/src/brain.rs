use std::collections::BTreeMap;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};
use sha2::{Digest, Sha256};

use crate::{
    Paths, State,
    witness::{self, Fact, channel},
};

const SCHEMA: i64 = 1;
const FILE: &str = "brain.db";
const WAITS_FOR_OTHERS: std::time::Duration = std::time::Duration::from_secs(5);
const PAGES_WORTH_COMPACTING: i64 = 256;

const TABLES: &str = "CREATE TABLE IF NOT EXISTS node(
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        title TEXT NOT NULL,
        stamp TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS edge(
        source TEXT NOT NULL,
        target TEXT NOT NULL,
        kind TEXT NOT NULL,
        PRIMARY KEY (source, target, kind)) WITHOUT ROWID;
    CREATE INDEX IF NOT EXISTS edge_by_target ON edge(target);
    CREATE TABLE IF NOT EXISTS card(
        id TEXT PRIMARY KEY,
        stamp TEXT NOT NULL,
        said TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS note(
        id TEXT PRIMARY KEY,
        said TEXT NOT NULL);";

const DROPPED: &str = "DROP TABLE IF EXISTS node;
    DROP TABLE IF EXISTS edge;
    DROP TABLE IF EXISTS card;
    DROP TABLE IF EXISTS note;
    DELETE FROM meta;";

const GONE_WITH_A_NODE: [&str; 4] = [
    "DELETE FROM node WHERE id = ?1",
    "DELETE FROM edge WHERE source = ?1 OR target = ?1",
    "DELETE FROM card WHERE id = ?1",
    "DELETE FROM note WHERE id = ?1",
];

pub struct Brain {
    db: Connection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refreshed {
    Unchanged,
    Moved { written: usize, removed: usize },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Round {
    Starting,
    Nudged,
    Looking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Doc,
    Page,
    Task,
    Tag,
    File,
}

impl Kind {
    fn as_str(self) -> &'static str {
        match self {
            Kind::Doc => "doc",
            Kind::Page => "page",
            Kind::Task => "task",
            Kind::Tag => "tag",
            Kind::File => "file",
        }
    }
}

struct Node {
    kind: Kind,
    title: String,
    stamp: String,
}

impl Brain {
    pub fn open(cache_dir: &Path) -> Option<Self> {
        if let Err(e) = std::fs::create_dir_all(cache_dir) {
            warned("the brain's folder is unusable", cache_dir, &e.to_string());
            return None;
        }
        let at = cache_dir.join(FILE);
        match Self::opened(&at) {
            Ok(brain) => brain,
            Err(e) if damaged(&e) => {
                warned(
                    "the brain was damaged, so it is built again",
                    &at,
                    &e.to_string(),
                );
                if !forget(cache_dir) {
                    return None;
                }
                Self::opened(&at)
                    .inspect_err(|e| warned("the brain is unopenable", &at, &e.to_string()))
                    .ok()
                    .flatten()
            }
            Err(e) => {
                warned("the brain is unopenable", &at, &e.to_string());
                None
            }
        }
    }

    pub fn checked(cache_dir: &Path) -> Option<Self> {
        let brain = Self::open(cache_dir)?;
        if brain.whole() {
            return Some(brain);
        }
        drop(brain);
        let at = cache_dir.join(FILE);
        warned(
            "the brain was damaged, so it is built again",
            &at,
            "quick_check",
        );
        if !forget(cache_dir) {
            return None;
        }
        Self::open(cache_dir)
    }

    fn opened(at: &Path) -> rusqlite::Result<Option<Self>> {
        let mut db = Connection::open(at)?;
        db.busy_timeout(WAITS_FOR_OTHERS)?;
        db.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )?;
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let held = said(&tx, "schema")?.and_then(|one| one.parse::<i64>().ok());
        if held.is_some_and(|one| one > SCHEMA) {
            witness::warn(
                channel::BRAIN,
                "a newer Tisty keeps the brain, so this one leaves it be",
                &[("at", Fact::Path(at.to_path_buf()))],
            );
            return Ok(None);
        }
        if held.is_some_and(|one| one < SCHEMA) {
            tx.execute_batch(DROPPED)?;
        }
        tx.execute_batch(TABLES)?;
        tx.execute(
            "INSERT OR REPLACE INTO meta VALUES ('schema', ?)",
            [SCHEMA.to_string()],
        )?;
        tx.commit()?;
        Ok(Some(Self { db }))
    }

    fn whole(&self) -> bool {
        self.db
            .query_row("PRAGMA quick_check", [], |row| row.get::<_, String>(0))
            .is_ok_and(|said| said == "ok")
    }

    fn seen(&self) -> Option<String> {
        said(&self.db, "seen").ok().flatten()
    }

    pub fn refresh(
        &mut self,
        state: &State,
        docs_root: &Path,
        seen: &str,
    ) -> rusqlite::Result<Refreshed> {
        let wanted = nodes(state, docs_root);
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let held = stamps(&tx)?;
        let mut written = 0;
        {
            let mut put = tx.prepare("INSERT OR REPLACE INTO node VALUES (?, ?, ?, ?)")?;
            for (id, node) in &wanted {
                if held.get(id) != Some(&node.stamp) {
                    put.execute(rusqlite::params![
                        id,
                        node.kind.as_str(),
                        node.title,
                        node.stamp
                    ])?;
                    written += 1;
                }
            }
        }
        let gone: Vec<&String> = held
            .keys()
            .filter(|one| !wanted.contains_key(*one))
            .collect();
        for sql in GONE_WITH_A_NODE {
            let mut taken = tx.prepare(sql)?;
            for id in &gone {
                taken.execute([id])?;
            }
        }
        tx.execute("INSERT OR REPLACE INTO meta VALUES ('seen', ?)", [seen])?;
        tx.commit()?;
        Ok(match (written, gone.len()) {
            (0, 0) => Refreshed::Unchanged,
            (written, removed) => Refreshed::Moved { written, removed },
        })
    }

    fn compact_if_worth_it(&self) {
        let counted = |pragma: &str| {
            self.db
                .query_row(pragma, [], |row| row.get::<_, i64>(0))
                .unwrap_or(0)
        };
        let pages = counted("PRAGMA page_count");
        let free = counted("PRAGMA freelist_count");
        if pages > PAGES_WORTH_COMPACTING && free * 4 > pages {
            let _ = self.db.execute_batch("VACUUM");
        }
        let _ = self.db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE)");
    }
}

pub fn kept_up(paths: &Paths, round: Round) -> Option<Refreshed> {
    let mut brain = match round {
        Round::Starting => Brain::checked(paths.cache()),
        Round::Nudged | Round::Looking => Brain::open(paths.cache()),
    }?;
    // Taken before the state is read, so whatever lands meanwhile leaves a stamp the next round misses.
    let seen = seen_on_disk(paths);
    // On Windows the directory entry lags a write, so a nudge never trusts the stamp.
    if round != Round::Nudged && brain.seen().as_deref() == Some(seen.as_str()) {
        return Some(Refreshed::Unchanged);
    }
    let state = match crate::cache::summarised(&paths.store(), paths.cache()) {
        Ok(state) => state,
        Err(e) => {
            warned(
                "the brain could not read the store",
                &paths.store(),
                &e.to_string(),
            );
            return None;
        }
    };
    match brain.refresh(&state, &paths.docs(), &seen) {
        Ok(done) => {
            if done != Refreshed::Unchanged {
                brain.compact_if_worth_it();
            }
            Some(done)
        }
        Err(e) => {
            warned(
                "the brain was not brought up to date",
                paths.cache(),
                &e.to_string(),
            );
            if damaged(&e) {
                drop(brain);
                forget(paths.cache());
            }
            None
        }
    }
}

fn seen_on_disk(paths: &Paths) -> String {
    format!(
        "{}\u{1}{}",
        crate::cache::fingerprint(&paths.store()),
        crate::docs::print(&paths.docs())
    )
}

fn nodes(state: &State, docs_root: &Path) -> BTreeMap<String, Node> {
    let mut all = BTreeMap::new();
    for kept in state.papers() {
        let told = serde_json::to_string(kept).unwrap_or_default();
        all.insert(
            kept.file.clone(),
            Node {
                kind: match kept.page_of {
                    Some(_) => Kind::Page,
                    None => Kind::Doc,
                },
                title: kept.title.clone().unwrap_or_else(|| kept.file.clone()),
                stamp: stamped(&[&told, &body_on_disk(docs_root, &kept.file)]),
            },
        );
    }
    for task in state.tasks.values() {
        all.insert(
            task.id.to_string(),
            Node {
                kind: Kind::Task,
                title: task.title.clone(),
                stamp: stamped(&[&without_its_bodies(task)]),
            },
        );
    }
    for tag in state.tags() {
        all.insert(
            format!("#{tag}"),
            Node {
                kind: Kind::Tag,
                title: tag.as_str().to_string(),
                stamp: String::new(),
            },
        );
    }
    for (at, (sha256, _)) in &state.kept {
        all.insert(
            at.clone(),
            Node {
                kind: Kind::File,
                title: at.rsplit('/').next().unwrap_or(at).to_string(),
                stamp: sha256.clone(),
            },
        );
    }
    all
}

// A state read with its bodies must stamp a task as the summary state does.
fn without_its_bodies(task: &crate::Task) -> String {
    let mut light = task.clone();
    light.description = None;
    light.log.clear();
    light.steps.clear();
    serde_json::to_string(&light).unwrap_or_default()
}

fn body_on_disk(docs_root: &Path, file: &str) -> String {
    let Some(told) = crate::docs::resolve(docs_root, file)
        .ok()
        .and_then(|at| std::fs::metadata(at).ok())
    else {
        return String::new();
    };
    let when = told
        .modified()
        .ok()
        .and_then(|at| at.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|since| since.as_nanos())
        .unwrap_or_default();
    format!("{}:{when}", told.len())
}

fn stamped(parts: &[&str]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    }
    crate::signing::hexed(&hasher.finalize())
}

fn stamps(db: &Connection) -> rusqlite::Result<BTreeMap<String, String>> {
    let mut asked = db.prepare("SELECT id, stamp FROM node")?;
    let rows = asked.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}

fn said(db: &Connection, key: &str) -> rusqlite::Result<Option<String>> {
    db.query_row("SELECT value FROM meta WHERE key = ?", [key], |row| {
        row.get(0)
    })
    .optional()
}

fn damaged(e: &rusqlite::Error) -> bool {
    matches!(
        e.sqlite_error_code(),
        Some(rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase)
    )
}

fn forget(cache_dir: &Path) -> bool {
    ["", "-wal", "-shm"].iter().all(|tail| {
        let at = cache_dir.join(format!("{FILE}{tail}"));
        match std::fs::remove_file(&at) {
            Ok(()) => true,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => true,
            Err(e) => {
                warned(
                    "the damaged brain could not be taken away",
                    &at,
                    &e.to_string(),
                );
                false
            }
        }
    })
}

fn warned(said: &'static str, at: &Path, why: &str) {
    witness::warn(
        channel::BRAIN,
        said,
        &[
            ("at", Fact::Path(at.to_path_buf())),
            ("why", Fact::Why(why.to_string())),
        ],
    );
}

#[cfg(test)]
#[path = "brain_test.rs"]
mod tests;
