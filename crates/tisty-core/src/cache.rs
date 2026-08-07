use std::path::Path;

use rusqlite::Connection;

use crate::{Result, State, event::Event, store};

const SCHEMA: i64 = 1;

/// The projection of a log that has not changed. Derived, disposable, and never
/// synced: losing it costs one slow read, trusting it wrongly would cost more.
pub struct Cache {
    db: Connection,
}

impl Cache {
    pub fn open(cache_dir: &Path) -> Result<Option<Self>> {
        if std::fs::create_dir_all(cache_dir).is_err() {
            return Ok(None);
        }
        let Ok(db) = Connection::open(cache_dir.join("read.db")) else {
            return Ok(None);
        };
        if db
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=NORMAL;
                 CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS task(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS list(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS tombstone(id TEXT PRIMARY KEY);",
            )
            .is_err()
        {
            return Ok(None);
        }
        Ok(Some(Self { db }))
    }

    /// The state as of the fingerprint given, or nothing if the log moved on.
    pub fn load(&self, fingerprint: &str) -> Option<State> {
        if self.meta("schema")? != SCHEMA.to_string() || self.meta("fingerprint")? != fingerprint {
            return None;
        }

        let mut state = State::default();
        for (table, into) in [("task", true), ("list", false)] {
            let mut q = self.db.prepare(&format!("SELECT doc FROM {table}")).ok()?;
            let rows = q
                .query_map([], |r| r.get::<_, String>(0))
                .ok()?
                .filter_map(|r| r.ok());
            for doc in rows {
                if into {
                    let task: crate::Task = serde_json::from_str(&doc).ok()?;
                    state.tasks.insert(task.id, task);
                } else {
                    let list: crate::List = serde_json::from_str(&doc).ok()?;
                    state.lists.insert(list.id, list);
                }
            }
        }
        let mut erased = self.db.prepare("SELECT id FROM tombstone").ok()?;
        let ids = erased
            .query_map([], |r| r.get::<_, String>(0))
            .ok()?
            .filter_map(|r| r.ok());
        for id in ids {
            if let Ok(id) = id.parse() {
                state.mark_erased(id);
            }
        }
        Some(state)
    }

    /// Written only after the log itself is on disk, so a crash between the two
    /// leaves the cache behind and repairable, never ahead and inventing.
    pub fn store(&mut self, state: &State, fingerprint: &str) -> Result<()> {
        let tx = match self.db.transaction() {
            Ok(tx) => tx,
            Err(_) => return Ok(()),
        };
        let written = (|| -> rusqlite::Result<()> {
            tx.execute("DELETE FROM task", [])?;
            tx.execute("DELETE FROM list", [])?;
            tx.execute("DELETE FROM tombstone", [])?;
            {
                let mut task = tx.prepare("INSERT INTO task VALUES (?,?)")?;
                for t in state.tasks.values() {
                    let doc = serde_json::to_string(t).unwrap_or_default();
                    task.execute(rusqlite::params![t.id.to_string(), doc])?;
                }
                let mut list = tx.prepare("INSERT INTO list VALUES (?,?)")?;
                for l in state.lists.values() {
                    let doc = serde_json::to_string(l).unwrap_or_default();
                    list.execute(rusqlite::params![l.id.to_string(), doc])?;
                }
            }
            {
                let mut gone = tx.prepare("INSERT INTO tombstone VALUES (?)")?;
                for id in state.erased() {
                    gone.execute([id.to_string()])?;
                }
            }
            tx.execute(
                "INSERT OR REPLACE INTO meta VALUES ('schema', ?), ('fingerprint', ?)",
                rusqlite::params![SCHEMA.to_string(), fingerprint],
            )?;
            tx.commit()
        })();
        let _ = written;
        Ok(())
    }

    /// Rewrites only what an event touched. Rebuilding on every write would
    /// make the cache slower than the log it stands in for.
    pub fn touch(&mut self, state: &State, entity: ulid::Ulid, fingerprint: &str) -> Result<()> {
        let _ = (|| -> rusqlite::Result<()> {
            let id = entity.to_string();
            match (state.tasks.get(&entity), state.lists.get(&entity)) {
                (Some(task), _) => {
                    let doc = serde_json::to_string(task).unwrap_or_default();
                    self.db.execute(
                        "INSERT OR REPLACE INTO task VALUES (?,?)",
                        rusqlite::params![id, doc],
                    )?;
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
                    self.db.execute("DELETE FROM list WHERE id = ?", [&id])?;
                    if state.is_erased(entity) {
                        self.db
                            .execute("INSERT OR REPLACE INTO tombstone VALUES (?)", [&id])?;
                    }
                }
            }
            self.db.execute(
                "INSERT OR REPLACE INTO meta VALUES ('fingerprint', ?)",
                [fingerprint],
            )?;
            Ok(())
        })();
        Ok(())
    }

    /// Some events reach further than their own entity — erasing a list moves
    /// every task it held. Those give up the fast path instead of guessing.
    pub fn invalidate(&mut self) {
        let _ = self
            .db
            .execute("DELETE FROM meta WHERE key = 'fingerprint'", []);
    }

    fn meta(&self, key: &str) -> Option<String> {
        self.db
            .query_row("SELECT value FROM meta WHERE key = ?", [key], |r| r.get(0))
            .ok()
    }
}

/// Names and sizes of every log file. A byte more anywhere and the cache is
/// stale — cheap to compute and impossible to be wrong about in the safe
/// direction: it only ever says «rebuild» when it should not have to.
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

/// What the cache holds against what the log says. The cache is only ever a
/// photograph, so the log wins every disagreement — this reports, never repairs.
pub fn audit(store_root: &Path, cache_dir: &Path) -> Result<Audit> {
    let truth = State::replay(&store::read_all(store_root)?);
    let Some(cache) = Cache::open(cache_dir)? else {
        return Ok(Audit::Unavailable);
    };

    let print = fingerprint(store_root);
    match cache.load(&print) {
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
    /// No cache to check, which is never a problem.
    Unavailable,
    /// The log moved on; the next read rebuilds.
    Stale {
        truth: State,
    },
    Agrees {
        truth: State,
    },
    /// The cache claims something the log does not. Rebuilding fixes it.
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

/// Throws the cache away so the next read builds it from the log again.
pub fn discard(cache_dir: &Path) -> Result<()> {
    if let Some(mut cache) = Cache::open(cache_dir)? {
        cache.invalidate();
    }
    Ok(())
}

/// The projection, from cache when the log has not moved and from the log when
/// it has. Falls back to reading the log whenever anything at all goes wrong.
pub fn project(store_root: &Path, cache_dir: &Path) -> Result<State> {
    let print = fingerprint(store_root);
    let mut cache = Cache::open(cache_dir)?;

    if let Some(cache) = &cache
        && let Some(state) = cache.load(&print)
    {
        return Ok(state);
    }

    let events: Vec<Event> = store::read_all(store_root)?;
    let state = State::replay(&events);

    if let Some(cache) = &mut cache {
        let _ = cache.store(&state, &print);
    }
    Ok(state)
}
