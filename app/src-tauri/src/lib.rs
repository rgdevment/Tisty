use std::sync::Mutex;

use tisty_core::{Config, Event, List, Op, Paths, State, Store, Task};

/// The CLI writes to the same store while the window is open, so what the
/// session holds can go stale under it.
struct Session {
    paths: Paths,
    state: State,
    store: Store,
    cache: Option<tisty_core::cache::Cache>,
    print: String,
    locale: Option<String>,
}

impl Session {
    fn open() -> tisty_core::Result<Self> {
        let paths = Paths::resolve()?;
        let config = Config::load_or_init(&paths)?;
        let store = Store::open(paths.store(), config.device_id.clone())?;
        let state = tisty_core::cache::summarised(&paths.store(), paths.cache())?;
        let cache = tisty_core::cache::Cache::open(paths.cache())?;
        let print = tisty_core::cache::fingerprint(&paths.store());

        Ok(Self {
            paths,
            state,
            store,
            cache,
            print,
            locale: config.locale,
        })
    }

    fn reload(&mut self) -> tisty_core::Result<bool> {
        let print = tisty_core::cache::fingerprint(&self.paths.store());
        if print == self.print {
            return Ok(false);
        }
        self.state = tisty_core::cache::summarised(&self.paths.store(), self.paths.cache())?;
        self.print = print;
        Ok(true)
    }

    fn commit(&mut self, op: Op) -> tisty_core::Result<()> {
        let event = self.store.append(op)?;
        self.state.apply(&event);
        self.print = self.carry(std::slice::from_ref(&event));
        Ok(())
    }

    fn commit_all(&mut self, ops: Vec<Op>) -> tisty_core::Result<()> {
        let events = self.store.append_batch(ops)?;
        for event in &events {
            self.state.apply(event);
        }
        self.print = self.carry(&events);
        Ok(())
    }

    fn carry(&mut self, events: &[Event]) -> String {
        tisty_core::cache::advance(
            self.cache.as_mut(),
            &self.state,
            events,
            &self.paths.store(),
        )
    }
}

#[derive(serde::Serialize)]
struct Snapshot {
    tasks: Vec<Task>,
    lists: Vec<List>,
    /// Set only when configured: the window would otherwise speak a different
    /// language than `tisty` on the same machine.
    locale: Option<String>,
}

type Answer<T> = std::result::Result<T, String>;

/// Recovers the guard: one panicked command would otherwise refuse every
/// command after it for the life of the window.
fn held<'a>(session: &'a tauri::State<'_, Mutex<Session>>) -> std::sync::MutexGuard<'a, Session> {
    session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[tauri::command]
fn snapshot(session: tauri::State<'_, Mutex<Session>>) -> Answer<Snapshot> {
    let mut session = held(&session);
    session.reload().map_err(|e| e.to_string())?;

    Ok(Snapshot {
        tasks: session.state.ordered_open().into_iter().cloned().collect(),
        lists: session.state.ordered_lists().into_iter().cloned().collect(),
        locale: session.locale.clone(),
    })
}

/// `locale` is the system's, via the webview; the configured one still wins.
#[tauri::command]
fn capture(
    session: tauri::State<'_, Mutex<Session>>,
    text: String,
    locale: String,
) -> Answer<Task> {
    let mut session = held(&session);
    let spoken = session.locale.clone().unwrap_or(locale);
    let draft = tisty_nl::parse(&text, &jiff::Zoned::now(), &spoken).into();

    let plan = tisty_core::capture::plan(&session.state, draft).map_err(|e| e.to_string())?;
    session.commit_all(plan.ops).map_err(|e| e.to_string())?;
    Ok(session.state.tasks[&plan.task].clone())
}

/// Reads without writing, so the field can show what it understood as it is typed.
#[tauri::command]
fn read(
    session: tauri::State<'_, Mutex<Session>>,
    text: String,
    locale: String,
) -> Answer<tisty_nl::Parsed> {
    let spoken = held(&session).locale.clone().unwrap_or(locale);
    Ok(tisty_nl::parse(&text, &jiff::Zoned::now(), &spoken))
}

#[tauri::command]
fn complete(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| "not a task id".to_string())?;
    held(&session)
        .commit(Op::TaskDone { id })
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn reopen(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| "not a task id".to_string())?;
    held(&session)
        .commit(Op::TaskReopen { id })
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let session = Session::open().expect("could not open the store");

    tauri::Builder::default()
        .manage(Mutex::new(session))
        .invoke_handler(tauri::generate_handler![
            snapshot, capture, read, complete, reopen
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
