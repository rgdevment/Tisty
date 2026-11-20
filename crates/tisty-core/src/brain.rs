use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior};

use crate::{
    Paths, State,
    witness::{self, Fact, channel},
};

mod alike;
mod card;
mod decided;
mod linked;
mod network;
mod nodes;
mod own;
mod reading;

pub use alike::{Alike, Passed, Weighing};
pub use card::{Around, Card, DocFacts, Facts, Guessed, Near, Section, TaskFacts, Tongue, Used};
pub use decided::Decision;
pub use network::{
    EDGES_AT_MOST, Guess, LeftOut, NODES_AT_MOST, Network, Neuron, TIES, card_of, dates, kept_out,
    network, touched_last,
};
pub use reading::{Citing, Headed, Under, cited_by};

const SCHEMA: i64 = 9;
const FILE: &str = "brain.db";
const WAITS_FOR_OTHERS: std::time::Duration = std::time::Duration::from_secs(5);
const PAGES_WORTH_COMPACTING: i64 = 256;

// Nothing reads or writes `note` any more; it stays so an earlier build on schema 9 still finds it.
const TABLES: &str = "CREATE TABLE IF NOT EXISTS node(
        id TEXT PRIMARY KEY,
        kind TEXT NOT NULL,
        title TEXT NOT NULL,
        grp TEXT,
        state TEXT,
        uid TEXT,
        stamp TEXT NOT NULL);
    CREATE INDEX IF NOT EXISTS node_by_uid ON node(uid);
    CREATE INDEX IF NOT EXISTS node_by_title ON node(title);
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
        said TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS zone(
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS words(
        id TEXT PRIMARY KEY,
        stamp TEXT NOT NULL,
        lang TEXT,
        guess TEXT,
        kin TEXT NOT NULL,
        said TEXT NOT NULL);
    CREATE TABLE IF NOT EXISTS alike(
        a TEXT NOT NULL,
        b TEXT NOT NULL,
        score REAL NOT NULL,
        shared TEXT NOT NULL,
        method TEXT NOT NULL,
        PRIMARY KEY (a, b)) WITHOUT ROWID;
    CREATE INDEX IF NOT EXISTS alike_by_b ON alike(b);";

// `vector` held what a model shipped by schema 7 made of the text; no Tisty writes it any more.
const DROPPED: &str = "DROP TABLE IF EXISTS node;
    DROP TABLE IF EXISTS edge;
    DROP TABLE IF EXISTS card;
    DROP TABLE IF EXISTS note;
    DROP TABLE IF EXISTS zone;
    DROP TABLE IF EXISTS words;
    DROP TABLE IF EXISTS alike;
    DROP TABLE IF EXISTS vector;
    DELETE FROM meta;";

// An edge is what its source wrote, so one aimed at a node that left stays for when it comes back.
const GONE_WITH_A_NODE: [&str; 6] = [
    "DELETE FROM node WHERE id = ?1",
    "DELETE FROM edge WHERE source = ?1",
    "DELETE FROM card WHERE id = ?1",
    "DELETE FROM note WHERE id = ?1",
    "DELETE FROM words WHERE id = ?1",
    "DELETE FROM alike WHERE a = ?1 OR b = ?1",
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

pub struct Docs {
    root: PathBuf,
    told: BTreeMap<String, String>,
}

impl Docs {
    pub fn on_disk(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            told: crate::docs::told_bodies(root),
        }
    }

    fn told(&self, file: &str) -> &str {
        self.told.get(file).map_or("", String::as_str)
    }

    fn read(&self, file: &str) -> Option<String> {
        crate::docs::read(&self.root, file).ok()
    }
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
        if schema_of(&db)? == Some(SCHEMA) {
            return Ok(Some(Self { db }));
        }
        let tx = db.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let held = schema_of(&tx)?;
        if held.is_some_and(|one| one > SCHEMA) {
            witness::trace(
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
        if held != Some(SCHEMA) {
            tx.execute(
                "INSERT OR REPLACE INTO meta VALUES ('schema', ?)",
                [SCHEMA.to_string()],
            )?;
        }
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
        docs: &Docs,
        seen: &str,
    ) -> rusqlite::Result<Refreshed> {
        let ground = linked::Ground::new(state, docs);
        let wanted = nodes::nodes(state, docs, &ground);
        // Bodies are read before the lock is taken, so nobody waits on the disk.
        let before = stamps(&self.db)?;
        let mut fresh: BTreeMap<&str, own::Fresh> = wanted
            .iter()
            .filter(|(id, node)| before.get(*id) != Some(&node.stamp))
            .map(|(id, node)| (id.as_str(), own::fresh(state, &ground, id, node)))
            .collect();
        let zones = nodes::zones(state);
        let out = alike::kept_out(state, docs);
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let held = stamps(&tx)?;
        let mut written = 0;
        let mut rewritten = Vec::new();
        {
            let mut put = tx.prepare("INSERT OR REPLACE INTO node VALUES (?, ?, ?, ?, ?, ?, ?)")?;
            let mut untied = tx.prepare("DELETE FROM edge WHERE source = ?")?;
            let mut tie = tx.prepare("INSERT OR IGNORE INTO edge VALUES (?, ?, ?)")?;
            let mut carded = tx.prepare("INSERT OR REPLACE INTO card VALUES (?, ?, ?)")?;
            let mut uncarded = tx.prepare("DELETE FROM card WHERE id = ?")?;
            for (id, node) in &wanted {
                if held.get(id) == Some(&node.stamp) {
                    continue;
                }
                put.execute(rusqlite::params![
                    id,
                    node.kind.as_str(),
                    node.title,
                    node.group,
                    node.standing,
                    node.uid,
                    node.stamp
                ])?;
                untied.execute([id])?;
                let own = fresh
                    .remove(id.as_str())
                    .unwrap_or_else(|| own::fresh(state, &ground, id, node));
                for (target, kind) in own.ties {
                    tie.execute(rusqlite::params![id, target, kind.as_str()])?;
                }
                if let Some(counted) = own.words {
                    rewritten.push(alike::Rewritten {
                        id,
                        stamp: &node.stamp,
                        counted,
                    });
                }
                match own.said {
                    Some(said) => carded.execute(rusqlite::params![id, node.stamp, said])?,
                    None => uncarded.execute([id])?,
                };
                written += 1;
            }
        }
        if zones_of(&tx)? != zones {
            tx.execute("DELETE FROM zone", [])?;
            let mut named = tx.prepare("INSERT INTO zone VALUES (?, ?)")?;
            for (id, name) in &zones {
                named.execute([id, name])?;
            }
        }
        let gone: Vec<&String> = held
            .keys()
            .filter(|one| !wanted.contains_key(*one))
            .collect();
        for id in alike::unworded(&tx, &wanted, &out, &rewritten)? {
            if let Some(counted) = own::fresh(state, &ground, id, &wanted[id]).words {
                rewritten.push(alike::Rewritten {
                    id,
                    stamp: &wanted[id].stamp,
                    counted,
                });
            }
        }
        alike::kept_in_round(&tx, &rewritten, &out, &gone)?;
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
    // A swept store has nothing to index, and a round would put its folders back.
    if !paths.store().is_dir() {
        return None;
    }
    let mut brain = match round {
        Round::Starting => Brain::checked(paths.cache()),
        Round::Nudged | Round::Looking => Brain::open(paths.cache()),
    }?;
    // Taken before the state is read, so whatever lands meanwhile leaves a stamp the next round misses.
    let docs = Docs::on_disk(&paths.docs());
    let seen = seen_on_disk(paths, &docs);
    // On Windows the directory entry lags a write, so a nudge never trusts the stamp.
    if round != Round::Nudged && brain.seen().as_deref() == Some(seen.as_str()) {
        return Some(Refreshed::Unchanged);
    }
    let state = match crate::cache::project(&paths.store(), paths.cache()) {
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
    match brain.refresh(&state, &docs, &seen) {
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

fn seen_on_disk(paths: &Paths, docs: &Docs) -> String {
    let mut seen = crate::cache::fingerprint(&paths.store());
    for (id, told) in &docs.told {
        seen.push_str(&format!("\u{1}{id}:{told}"));
    }
    seen
}

fn stamps(db: &Connection) -> rusqlite::Result<BTreeMap<String, String>> {
    let mut asked = db.prepare("SELECT id, stamp FROM node")?;
    let rows = asked.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}

fn zones_of(db: &Connection) -> rusqlite::Result<BTreeMap<String, String>> {
    let mut asked = db.prepare("SELECT id, name FROM zone")?;
    let rows = asked.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}

fn schema_of(db: &Connection) -> rusqlite::Result<Option<i64>> {
    Ok(said(db, "schema")?.and_then(|one| one.parse().ok()))
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
    // The journal goes first: one left beside a fresh base would be replayed into it.
    ["-wal", "-shm", ""].iter().all(|tail| {
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
