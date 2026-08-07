use std::sync::Mutex;

use tisty_core::{
    Config, Event, List, Op, Paths, State, Store, Tag, Task,
    view::{Filter, Scope, Window},
};

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
            self.store.overtaken(),
        )
    }
}

/// Counted with the same filter the views use, or the sidebar would promise a
/// number the list does not deliver.
fn tally(state: &State) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    let mut count = |key: &str, filter: Filter| {
        counts.insert(key.to_string(), state.matching(&filter, today()).len());
    };

    count(
        "inbox",
        Filter {
            inbox: true,
            ..Default::default()
        },
    );
    count(
        "today",
        Filter {
            window: Some(Window::Today),
            ..Default::default()
        },
    );

    for list in state.ordered_lists() {
        counts.insert(list.id.to_string(), state.tasks_in(list.id).count());
    }
    counts
}

#[derive(serde::Serialize)]
struct Counted {
    tag: String,
    tasks: usize,
}

/// No catalogue to administer: a tag exists because some task mentions it, and
/// the count reaches into the archive because that is where a tag earns its keep.
fn tags_in_use(state: &State) -> Vec<Counted> {
    state
        .tags()
        .into_iter()
        .map(|tag| Counted {
            tag: tag.to_string(),
            tasks: state.tasks_tagged(tag).count(),
        })
        .collect()
}

#[derive(serde::Serialize)]
struct Snapshot {
    tasks: Vec<Task>,
    lists: Vec<List>,
    tags: Vec<Counted>,
    counts: std::collections::BTreeMap<String, usize>,
    /// Set only when configured: the window would otherwise speak a different
    /// language than `tisty` on the same machine.
    locale: Option<String>,
}

/// The core states its reasons in English on purpose. What travels is the
/// reason itself, so each client says it in the language it speaks.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Refusal {
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

impl Refusal {
    fn of(code: &'static str) -> Self {
        Self { code, name: None }
    }

    fn about(code: &'static str, name: impl Into<String>) -> Self {
        Self {
            code,
            name: Some(name.into()),
        }
    }
}

impl From<tisty_core::capture::Rejected> for Refusal {
    fn from(rejected: tisty_core::capture::Rejected) -> Self {
        use tisty_core::capture::Rejected;
        match rejected {
            Rejected::Untitled => Refusal::of("untitled"),
            Rejected::NoSuchList(name) => Refusal::about("noSuchList", name),
            Rejected::AmbiguousList(name) => Refusal::about("ambiguousList", name),
        }
    }
}

impl From<tisty_core::Error> for Refusal {
    fn from(error: tisty_core::Error) -> Self {
        Refusal::about("internal", error.to_string())
    }
}

type Answer<T> = std::result::Result<T, Refusal>;

/// Recovers the guard: one panicked command would otherwise refuse every
/// command after it for the life of the window.
fn held<'a>(session: &'a tauri::State<'_, Mutex<Session>>) -> std::sync::MutexGuard<'a, Session> {
    session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The day is decided where the user is, never in UTC.
fn today() -> jiff::civil::Date {
    jiff::Zoned::now().date()
}

fn zone() -> String {
    jiff::Zoned::now()
        .time_zone()
        .iana_name()
        .unwrap_or("UTC")
        .to_string()
}

#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct View {
    #[serde(default)]
    archive: bool,
    /// A tag reaches across the archive: «everything I did with #istio».
    #[serde(default)]
    everything: bool,
    #[serde(default)]
    inbox: bool,
    #[serde(default)]
    list: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    window: Option<String>,
}

impl View {
    fn resolve(self) -> Result<Filter, Refusal> {
        Ok(Filter {
            scope: match (self.everything, self.archive) {
                (true, _) => Scope::Either,
                (_, true) => Scope::Archived,
                _ => Scope::Open,
            },
            inbox: self.inbox,
            lists: self
                .list
                .map(|id| id.parse().map_err(|_| Refusal::of("notAListId")))
                .transpose()?
                .into_iter()
                .collect(),
            tags: self
                .tags
                .iter()
                .map(|t| Tag::new(t).map_err(|_| Refusal::about("badTag", t)))
                .collect::<Result<_, _>>()?,
            priority: None,
            window: match self.window.as_deref() {
                Some("today") => Some(Window::Today),
                Some("upcoming") => Some(Window::After(today())),
                Some("overdue") => Some(Window::Overdue),
                _ => None,
            },
        })
    }
}

#[tauri::command]
fn snapshot(session: tauri::State<'_, Mutex<Session>>, view: Option<View>) -> Answer<Snapshot> {
    let mut session = held(&session);
    session.reload()?;

    let filter = match view {
        Some(view) => view.resolve()?,
        None => Filter::default(),
    };

    Ok(Snapshot {
        tasks: session
            .state
            .matching(&filter, today())
            .into_iter()
            .cloned()
            .collect(),
        lists: session.state.ordered_lists().into_iter().cloned().collect(),
        tags: tags_in_use(&session.state),
        counts: tally(&session.state),
        locale: session.locale.clone(),
    })
}

/// `locale` is the system's, via the webview; the configured one still wins.
#[tauri::command]
fn capture(
    session: tauri::State<'_, Mutex<Session>>,
    text: String,
    locale: String,
    view: Option<View>,
) -> Answer<Task> {
    let mut session = held(&session);
    let spoken = session.locale.clone().unwrap_or(locale);
    let mut draft: tisty_core::capture::Draft =
        tisty_nl::parse(&text, &jiff::Zoned::now(), &spoken).into();

    // The view is where the task is born; what the text says wins over it,
    // because typing `@work` is asking for something explicitly.
    if let Some(view) = view {
        if draft.filing.is_none()
            && let Some(list) = view.list
        {
            draft.filing = Some(tisty_core::capture::Filing::Named(list));
        }
        for name in view.tags {
            if let Ok(tag) = Tag::new(&name)
                && !draft.tags.contains(&tag)
            {
                draft.tags.push(tag);
            }
        }
        if draft.date.is_none() && view.window.as_deref() == Some("today") {
            draft.date = Some(tisty_core::DateSpec::all_day(today(), zone()));
        }
    }

    let plan = tisty_core::capture::plan(&session.state, draft)?;
    session.commit_all(plan.ops)?;
    Ok(session.state.tasks[&plan.task].clone())
}

#[tauri::command]
fn read(
    session: tauri::State<'_, Mutex<Session>>,
    text: String,
    locale: String,
) -> Answer<tisty_nl::Parsed> {
    let spoken = held(&session).locale.clone().unwrap_or(locale);
    Ok(tisty_nl::parse(&text, &jiff::Zoned::now(), &spoken))
}

/// The window searches through the core, or it would find different things
/// than `tisty search` does with the same words.
#[tauri::command]
fn search(
    session: tauri::State<'_, Mutex<Session>>,
    query: String,
    scope: Option<String>,
) -> Answer<Vec<Task>> {
    let mut session = held(&session);
    session.reload()?;

    let scope = match scope.as_deref() {
        Some("open") => Scope::Open,
        Some("archived") => Scope::Archived,
        _ => Scope::Either,
    };
    Ok(session
        .state
        .search(&query, scope)
        .into_iter()
        .cloned()
        .collect())
}

#[tauri::command]
fn complete(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    held(&session)
        .commit(Op::TaskDone { id })
        .map_err(Refusal::from)
}

#[tauri::command]
fn reopen(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    held(&session)
        .commit(Op::TaskReopen { id })
        .map_err(Refusal::from)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let session = Session::open().expect("could not open the store");

    tauri::Builder::default()
        .manage(Mutex::new(session))
        .invoke_handler(tauri::generate_handler![
            snapshot, capture, read, search, complete, reopen
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
