use std::path::Path;

use rusqlite::Connection;

use crate::{
    Result, State,
    event::Event,
    store,
    witness::{self, Fact, channel},
};

/// Tied to the event schema: an older build then misses the cache and meets the version guard.
const SCHEMA: i64 = crate::event::SCHEMA_VERSION as i64 + 1;

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
        if let Err(e) = db.execute_batch(
            "PRAGMA journal_mode=WAL;
                 PRAGMA synchronous=NORMAL;
                 CREATE TABLE IF NOT EXISTS meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS task(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS task_body(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS list(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS folder(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS doc(id TEXT PRIMARY KEY, doc TEXT NOT NULL);
                 CREATE TABLE IF NOT EXISTS tombstone(id TEXT PRIMARY KEY);",
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
                    witness::warn(channel::CACHE, "cached body unreadable", &[]);
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
                let mut gone = tx.prepare("INSERT INTO tombstone VALUES (?)")?;
                for id in state.erased() {
                    gone.execute([id.to_string()])?;
                }
            }
            tx.execute(
                "INSERT OR REPLACE INTO meta VALUES ('schema', ?), ('fingerprint', ?), ('devices', ?), ('dropped', ?), ('retired', ?), ('shed', ?)",
                rusqlite::params![
                    SCHEMA.to_string(),
                    fingerprint,
                    serde_json::to_string(&state.devices).unwrap_or_default(),
                    serde_json::to_string(&state.dropped).unwrap_or_default(),
                    serde_json::to_string(&state.retired).unwrap_or_default(),
                    serde_json::to_string(&state.shed).unwrap_or_default(),
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

pub fn advance(
    cache: Option<&mut Cache>,
    state: &State,
    events: &[crate::Event],
    store_root: &Path,
    overtaken: bool,
) -> String {
    let print = fingerprint(store_root);
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
                | crate::Op::DeviceRemove { .. }
                | crate::Op::AttachRetire { .. }
        )
    }) {
        cache.invalidate();
        return print;
    }

    for event in events {
        if let Some(id) = event.entity_id() {
            let _ = cache.touch(state, id, &print);
        }
    }
    print
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

    let events: Vec<Event> = store::read_all(store_root)?;
    let state = State::replay(&events);

    if let Some(cache) = &mut cache {
        let _ = cache.store(&state, &print);
        if !bodies && let Some(light) = cache.load(&print, false) {
            return Ok(light);
        }
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
    fn a_word_about_a_machine_never_leaves_the_cache_claiming_to_be_current() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::open(dir.path()).unwrap().expect("a cache opens");
        let state = State::default();
        cache.store(&state, "before").unwrap();

        let said = crate::Event::new(
            crate::DeviceId("mac0".into()),
            jiff::Timestamp::from_second(1).unwrap(),
            crate::Op::DeviceRemove {
                d: crate::DeviceId("win1".into()),
            },
        );
        advance(Some(&mut cache), &state, &[said], dir.path(), false);

        assert!(
            cache.load("before", true).is_none(),
            "the cache still says it holds what it never saw"
        );
    }

    #[test]
    fn what_the_cache_gives_back_still_knows_which_machines_write() {
        let dir = tempfile::tempdir().unwrap();
        let mut cache = Cache::open(dir.path()).unwrap().expect("a cache opens");
        let mut state = State::default();
        state.devices.insert(crate::DeviceId("mac0".into()));
        state.shed.insert("a-0009".into());
        state
            .retired
            .insert("attachments/ab/charla-a3f9.mp4".into());

        cache.store(&state, "print").unwrap();
        let back = cache.load("print", true).expect("the cache had it");

        assert_eq!(back.devices, state.devices, "the list of machines was lost");
        assert_eq!(back.retired, state.retired, "the retirements were lost");
        assert_eq!(
            back.shed, state.shed,
            "sin esto el barrido de documentos borrados no correria nunca con la cache caliente"
        );
    }

    #[test]
    fn a_second_launch_still_has_its_folders_and_documents() {
        let f = loaded();
        let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        let folder = Ulid::generate();
        store
            .append(Op::FolderAdd {
                id: folder,
                d: crate::event::FolderAdd {
                    name: "trabajo".into(),
                    order: "a0".into(),
                    parent: None,
                    icon: None,
                    color: None,
                },
            })
            .unwrap();
        store
            .append(Op::DocAdd {
                id: Ulid::generate(),
                d: crate::event::DocAdd {
                    file: "a3f1-0001".into(),
                    order: "a0".into(),
                    folder: Some(folder),
                },
            })
            .unwrap();

        let first = project(&f.store_root, &f.cache_dir).unwrap();
        assert_eq!(first.folders.len(), 1, "the log itself lost them");

        let second = project(&f.store_root, &f.cache_dir).unwrap();

        assert_eq!(second.folders.len(), 1, "the tree emptied itself");
        assert_eq!(second.docs.len(), 1, "every document came back unfiled");
        assert_eq!(second.inside(folder).len(), 1);
    }

    #[test]
    fn a_summary_asked_for_on_a_cold_cache_is_as_light_as_on_a_warm_one() {
        let f = loaded();
        let cold = summarised(&f.store_root, &f.cache_dir).unwrap();
        let warm = summarised(&f.store_root, &f.cache_dir).unwrap();

        assert!(
            !cold.has_bodies(),
            "el primer resumen tras vaciar la cache traia el cuerpo entero"
        );
        assert!(
            cold.tasks[&f.task].steps.is_empty() && cold.tasks[&f.task].log.is_empty(),
            "el cuerpo viajo en el resumen"
        );
        assert_eq!(
            cold, warm,
            "el resumen depende de si la cache estaba caliente"
        );
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

        let again = project(&f.store_root, &f.cache_dir).unwrap();
        assert!(again.is_erased(f.task), "the cache forgot a deletion");
    }

    #[test]
    fn advancing_leaves_the_cache_current_after_a_write() {
        let f = loaded();
        let mut state = project(&f.store_root, &f.cache_dir).unwrap();
        let mut cache = Cache::open(&f.cache_dir).unwrap();

        let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        let event = store.append(Op::TaskDone { id: f.task }).unwrap();
        state.apply(&event);

        let print = advance(
            cache.as_mut(),
            &state,
            std::slice::from_ref(&event),
            &f.store_root,
            false,
        );

        assert_eq!(print, fingerprint(&f.store_root));
        assert!(
            matches!(
                audit(&f.store_root, &f.cache_dir).unwrap(),
                Audit::Agrees { .. }
            ),
            "the cache fell behind the log"
        );
    }

    #[test]
    fn a_summary_that_writes_never_erases_the_bodies_it_left_behind() {
        let f = loaded();
        project(&f.store_root, &f.cache_dir).unwrap();

        let mut light = summarised(&f.store_root, &f.cache_dir).unwrap();
        let mut cache = Cache::open(&f.cache_dir).unwrap();

        let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        let event = store.append(Op::TaskDone { id: f.task }).unwrap();
        light.apply(&event);
        advance(
            cache.as_mut(),
            &light,
            std::slice::from_ref(&event),
            &f.store_root,
            false,
        );

        let whole = project(&f.store_root, &f.cache_dir).unwrap();
        assert_eq!(
            whole,
            State::replay(&store::read_all(&f.store_root).unwrap()),
            "the cache kept a body-less state and called it fresh"
        );
        assert_eq!(whole.tasks[&f.task].log.len(), 2, "the journal was erased");
        assert_eq!(whole.tasks[&f.task].steps.len(), 1, "the steps were erased");
    }

    #[test]
    fn a_summary_keeps_the_volume_it_was_handed() {
        let f = loaded();
        project(&f.store_root, &f.cache_dir).unwrap();

        let mut light = summarised(&f.store_root, &f.cache_dir).unwrap();
        let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        let event = store.append(Op::TaskDone { id: f.task }).unwrap();
        light.apply(&event);

        let task = &light.tasks[&f.task];
        assert_eq!(task.journal_count(), 2, "the volume was recounted from air");
        assert_eq!(task.steps_done(), (0, 1));
    }

    #[test]
    fn a_write_that_is_not_carried_leaves_the_cache_behind() {
        let f = loaded();
        project(&f.store_root, &f.cache_dir).unwrap();

        let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        store.append(Op::TaskDone { id: f.task }).unwrap();

        assert!(matches!(
            audit(&f.store_root, &f.cache_dir).unwrap(),
            Audit::Stale { .. }
        ));
    }

    #[test]
    fn a_state_that_was_overtaken_is_thrown_away_instead_of_carried() {
        let f = loaded();
        let state = project(&f.store_root, &f.cache_dir).unwrap();

        let mut theirs = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
        let event = theirs.append(Op::TaskDone { id: f.task }).unwrap();

        let mut cache = Cache::open(&f.cache_dir).unwrap();
        advance(
            cache.as_mut(),
            &state,
            std::slice::from_ref(&event),
            &f.store_root,
            true,
        );

        let fresh = Cache::open(&f.cache_dir).unwrap().unwrap();
        assert!(
            fresh.load(&fingerprint(&f.store_root), true).is_none(),
            "a cache written from an overtaken state must not answer as fresh"
        );
    }
}
