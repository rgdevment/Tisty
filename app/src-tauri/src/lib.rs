use std::sync::Mutex;

use tisty_core::{
    Config, Event, List, Op, Paths, State, Store, Tag, Task,
    event::TaskPatch,
    view::{Filter, Scope, Window},
};

/// The CLI writes to the same store while the window is open, so this can go stale under it.
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

/// Counted with the same filter the views use, or the sidebar would promise a number the list does not deliver.
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

/// Derived from tasks in use; the count reaches into the archive too.
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
    /// Set only when configured, or the window would speak a different language than `tisty`.
    locale: Option<String>,
}

/// `code` is untranslated; each client renders it in the language it speaks.
#[derive(Debug, serde::Serialize)]
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

/// Recovers the guard, or one panicked command would refuse every command after it.
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
    /// The tag view with nothing picked lists what carries a tag, not everything.
    #[serde(default)]
    tagged: bool,
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
            tagged: self.tagged,
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

/// Same knobs the CLI exposes as flags; the window may not store anything `tisty add` cannot.
#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Edits {
    #[serde(default)]
    no_date: bool,
    #[serde(default)]
    no_deadline: bool,
    #[serde(default)]
    no_list: bool,
    #[serde(default)]
    no_priority: bool,
    #[serde(default)]
    no_tags: Vec<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    deadline: Option<String>,
    #[serde(default)]
    priority: Option<u8>,
    /// The offered phrase was accepted, so it stops being part of the title.
    #[serde(default)]
    take_offer: bool,
}

impl Edits {
    fn apply(
        &self,
        draft: &mut tisty_core::capture::Draft,
        now: &jiff::Zoned,
        spoken: &str,
    ) -> Result<(), Refusal> {
        if self.no_date {
            draft.date = None;
        }
        if self.no_deadline {
            draft.deadline = None;
        }
        if self.no_list {
            draft.filing = None;
        }
        if self.no_priority {
            draft.priority = None;
        }
        for name in &self.no_tags {
            if let Ok(tag) = Tag::new(name) {
                draft.tags.retain(|kept| *kept != tag);
            }
        }
        if let Some(raw) = &self.date {
            draft.date = Some(dated(raw, now, spoken)?);
        }
        if let Some(raw) = &self.deadline {
            draft.deadline = Some(dated(raw, now, spoken)?);
        }
        if let Some(level) = self.priority {
            draft.priority = Some(
                tisty_core::Priority::try_from(level).map_err(|_| Refusal::of("notAPriority"))?,
            );
        }
        Ok(())
    }

    /// A reading the user unmarked goes back into the title; picking a different date is not unmarking.
    fn retitled(&self, text: &str, read: &tisty_nl::Parsed) -> Option<String> {
        let undone = self.no_date
            || self.no_deadline
            || self.no_list
            || self.no_priority
            || !self.no_tags.is_empty();
        if !undone && !self.take_offer {
            return None;
        }

        let letters: Vec<char> = text.chars().collect();
        let mut kept: Vec<tisty_nl::Span> = read
            .spans
            .iter()
            .copied()
            .filter(|span| !self.unmarked(span, &letters))
            .collect();

        if self.take_offer
            && let Some(offer) = read.offers.first()
        {
            kept.extend(offer.spans.iter().copied());
        }
        Some(tisty_nl::title_without(text, &kept))
    }

    fn unmarked(&self, span: &tisty_nl::Span, letters: &[char]) -> bool {
        match span.mark {
            tisty_nl::Mark::Date => self.no_date,
            tisty_nl::Mark::Deadline => self.no_deadline,
            tisty_nl::Mark::List => self.no_list,
            tisty_nl::Mark::Priority => self.no_priority,
            tisty_nl::Mark::Tag => {
                let written: String = letters[span.from..span.to].iter().collect();
                Tag::new(written.trim_start_matches('#')).is_ok_and(|tag| {
                    self.no_tags
                        .iter()
                        .any(|name| Tag::new(name) == Ok(tag.clone()))
                })
            }
        }
    }
}

fn dated(raw: &str, now: &jiff::Zoned, spoken: &str) -> Result<tisty_core::DateSpec, Refusal> {
    tisty_nl::parse_date(raw, now, spoken).ok_or_else(|| Refusal::about("notADate", raw))
}

/// `locale` is the system's, via the webview; the configured one still wins.
#[tauri::command]
fn capture(
    session: tauri::State<'_, Mutex<Session>>,
    text: String,
    locale: String,
    view: Option<View>,
    edits: Option<Edits>,
) -> Answer<Task> {
    let mut session = held(&session);
    let spoken = session.locale.clone().unwrap_or(locale);
    let now = jiff::Zoned::now();
    let read = tisty_nl::parse(&text, &now, &spoken);
    let mut draft: tisty_core::capture::Draft = read.clone().into();

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

    // Last, or removing the date inside «Hoy» would hand it straight back.
    let edits = edits.unwrap_or_default();
    edits.apply(&mut draft, &now, &spoken)?;
    if let Some(title) = edits.retitled(&text, &read) {
        draft.title = title;
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

#[derive(serde::Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct Change {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    no_date: bool,
    #[serde(default)]
    deadline: Option<String>,
    #[serde(default)]
    no_deadline: bool,
    #[serde(default)]
    priority: Option<u8>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    list: Option<String>,
    #[serde(default)]
    inbox: bool,
    #[serde(default)]
    description: Option<String>,
}

/// One batch: an undo has to take back the whole edit, not half of it.
#[tauri::command]
fn patch(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    change: Change,
    locale: String,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    let spoken = session.locale.clone().unwrap_or(locale);
    let now = jiff::Zoned::now();

    let d = TaskPatch {
        title: change
            .title
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty()),
        date: dated_field(change.date.as_deref(), change.no_date, &now, &spoken)?,
        deadline: dated_field(
            change.deadline.as_deref(),
            change.no_deadline,
            &now,
            &spoken,
        )?,
        priority: change
            .priority
            .map(tisty_core::Priority::try_from)
            .transpose()
            .map_err(|_| Refusal::of("notAPriority"))?,
        tags: change
            .tags
            .map(|names| {
                names
                    .iter()
                    .map(|name| Tag::new(name).map_err(|_| Refusal::about("badTag", name)))
                    .collect::<Result<_, _>>()
            })
            .transpose()?,
        reminders: None,
    };

    let filed = match (&change.list, change.inbox) {
        (Some(raw), _) => Some(Some(raw.parse().map_err(|_| Refusal::of("notAListId"))?)),
        (None, true) => Some(None),
        _ => None,
    };

    let mut ops = Vec::new();
    if d != TaskPatch::default() {
        ops.push(Op::TaskUpdate { id, d });
    }
    if let Some(body) = change.description {
        let kept = body.trim().to_string();
        ops.push(Op::TaskDescribe {
            id,
            d: tisty_core::event::Body {
                body: (!kept.is_empty()).then_some(kept),
            },
        });
    }
    if let Some(list) = filed {
        ops.push(Op::TaskMove {
            id,
            d: tisty_core::event::TaskMove {
                list: Some(list),
                order: None,
            },
        });
    }
    if !ops.is_empty() {
        session.commit_all(ops)?;
    }
    Ok(session.state.tasks[&id].clone())
}

fn dated_field(
    raw: Option<&str>,
    cleared: bool,
    now: &jiff::Zoned,
    spoken: &str,
) -> Result<Option<Option<tisty_core::DateSpec>>, Refusal> {
    match (raw, cleared) {
        (Some(raw), _) => Ok(Some(Some(dated(raw, now, spoken)?))),
        (None, true) => Ok(Some(None)),
        _ => Ok(None),
    }
}

/// Searches through the core, or it would find different things than `tisty search` does.
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
            snapshot, capture, read, search, complete, reopen, patch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> jiff::Zoned {
        "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
    }

    #[test]
    fn an_accepted_offer_sets_the_date_and_trims_the_title() {
        let text = "revisar el informe del lunes";
        let read = tisty_nl::parse(text, &now(), "es");
        let offer = read.offers.first().cloned().expect("an offer");
        let mut draft: tisty_core::capture::Draft = read.clone().into();
        assert!(draft.date.is_none());

        let edits = Edits {
            date: Some(offer.date.date().to_string()),
            take_offer: true,
            ..Default::default()
        };
        edits.apply(&mut draft, &now(), "es").unwrap();
        draft.title = edits.retitled(text, &read).expect("a new title");

        assert_eq!(draft.title, "revisar el informe");
        assert_eq!(draft.date.unwrap().date().to_string(), "2026-08-10");
    }

    #[test]
    fn a_removal_leaves_nothing_behind() {
        let mut draft: tisty_core::capture::Draft =
            tisty_nl::parse("comprar pan mañana #casa !1", &now(), "es").into();
        assert!(draft.date.is_some());

        Edits {
            no_date: true,
            no_priority: true,
            no_tags: vec!["casa".to_string()],
            ..Default::default()
        }
        .apply(&mut draft, &now(), "es")
        .unwrap();

        assert!(draft.date.is_none());
        assert!(draft.priority.is_none());
        assert!(draft.tags.is_empty());
    }

    #[test]
    fn an_unmarked_reading_returns_to_the_title() {
        let text = "comprar pan el proximo lunes #casa";
        let read = tisty_nl::parse(text, &now(), "es");
        assert_eq!(read.title, "comprar pan");

        let edits = Edits {
            no_tags: vec!["casa".to_string()],
            ..Default::default()
        };
        assert_eq!(
            edits.retitled(text, &read).as_deref(),
            Some("comprar pan #casa")
        );
    }

    #[test]
    fn choosing_a_different_date_leaves_the_title_alone() {
        let text = "comprar pan mañana";
        let read = tisty_nl::parse(text, &now(), "es");
        let edits = Edits {
            date: Some("2026-08-20".to_string()),
            ..Default::default()
        };
        assert_eq!(edits.retitled(text, &read), None);
    }
}
