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
                 CREATE TABLE IF NOT EXISTS task_body(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
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
    /// `bodies` decides whether descriptions, journals and steps come along.
    pub fn load(&self, fingerprint: &str, bodies: bool) -> Option<State> {
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
        if bodies {
            let mut q = self.db.prepare("SELECT id, doc FROM task_body").ok()?;
            let rows = q
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .ok()?
                .filter_map(|r| r.ok());
            for (id, doc) in rows {
                let (Ok(id), Ok(body)) = (id.parse(), serde_json::from_str::<Body>(&doc)) else {
                    return None;
                };
                if let Some(task) = state.tasks.get_mut(&id) {
                    task.description = body.description;
                    task.log = body.log;
                    task.steps = body.steps;
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
            tx.execute("DELETE FROM task_body", [])?;
            tx.execute("DELETE FROM list", [])?;
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

/// The body is what makes a task heavy — a journal of forty entries is fifteen
/// kilobytes against two hundred bytes of everything else. Listing never needs
/// it, so it is stored apart and left on disk until something asks.
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
    projected(store_root, cache_dir, true)
}

/// Without bodies: everything a listing needs and none of what makes it slow.
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

    let events: Vec<Event> = store::read_all(store_root)?;
    let state = State::replay(&events);

    if let Some(cache) = &mut cache {
        let _ = cache.store(&state, &print);
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{DeviceId, LogAdd, StepAdd, TaskAdd};
    use crate::{Op, Store};
    use ulid::Ulid;

    struct Fixture {
        _tmp: tempfile::TempDir,
        store_root: std::path::PathBuf,
        cache_dir: std::path::PathBuf,
        task: Ulid,
    }

    fn loaded() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let cache_dir = tmp.path().join("cache");

        let mut store = Store::open(&store_root, DeviceId("dev_a".into())).unwrap();
        let task = Ulid::generate();
        store
            .append(Op::TaskAdd {
                id: task,
                d: TaskAdd::new("write the report", "a0"),
            })
            .unwrap();
        for body in ["spoke to accounting", "still waiting"] {
            store
                .append(Op::TaskLog {
                    id: task,
                    d: LogAdd::new(Ulid::generate(), body),
                })
                .unwrap();
        }
        store
            .append(Op::StepAdd {
                id: task,
                d: StepAdd {
                    step: Ulid::generate(),
                    text: "collect the figures".into(),
                    order: "a0".into(),
                },
            })
            .unwrap();

        Fixture {
            _tmp: tmp,
            store_root,
            cache_dir,
            task,
        }
    }

    #[test]
    fn a_summary_knows_how_much_body_it_left_behind() {
        let f = loaded();
        project(&f.store_root, &f.cache_dir).unwrap();

        let light = summarised(&f.store_root, &f.cache_dir).unwrap();
        let task = &light.tasks[&f.task];

        assert!(task.log.is_empty(), "the body came along");
        assert!(task.steps.is_empty(), "the body came along");
        assert_eq!(task.journal_count(), 2);
        assert_eq!(task.steps_done(), (0, 1));
    }

    #[test]
    fn the_full_load_carries_everything_the_log_had() {
        let f = loaded();
        let cached = project(&f.store_root, &f.cache_dir).unwrap();
        let replayed = State::replay(&store::read_all(&f.store_root).unwrap());

        assert_eq!(cached, replayed, "the cache disagrees with the log");
        assert_eq!(cached.tasks[&f.task].log.len(), 2);
    }

    #[test]
    fn a_cache_built_once_is_reused_and_a_deleted_one_is_rebuilt() {
        let f = loaded();
        let first = project(&f.store_root, &f.cache_dir).unwrap();
        assert!(matches!(
            audit(&f.store_root, &f.cache_dir).unwrap(),
            Audit::Agrees { .. }
        ));

        std::fs::remove_dir_all(&f.cache_dir).unwrap();
        assert_eq!(project(&f.store_root, &f.cache_dir).unwrap(), first);
    }

    #[test]
    fn a_log_that_grew_leaves_the_cache_behind() {
        let f = loaded();
        project(&f.store_root, &f.cache_dir).unwrap();

        let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        store
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new("something later", "a1"),
            })
            .unwrap();

        assert!(matches!(
            audit(&f.store_root, &f.cache_dir).unwrap(),
            Audit::Stale { .. }
        ));
        assert_eq!(project(&f.store_root, &f.cache_dir).unwrap().tasks.len(), 2);
    }

    #[test]
    fn tombstones_survive_the_round_trip() {
        let f = loaded();
        let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        store.append(Op::TaskDelete { id: f.task }).unwrap();

        let state = project(&f.store_root, &f.cache_dir).unwrap();
        assert!(state.is_erased(f.task));

        // Straight from the cache this time, and it has to remember just the same.
        let again = project(&f.store_root, &f.cache_dir).unwrap();
        assert!(again.is_erased(f.task), "the cache forgot a deletion");
    }
}
