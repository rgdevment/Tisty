use std::sync::Mutex;

use tauri::Manager;

use tisty_core::{
    Config, Event, List, Op, Paths, State, Store, Tag, Task,
    event::{LogAdd, LogEdit, StepAdd, StepRef, StepText, TaskPatch},
    view::{Filter, Scope, Window},
};

/// The CLI writes to the same store while the window is open, so this can go stale under it.
struct Session {
    paths: Paths,
    config: Config,
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
        let state = tisty_core::cache::project(&paths.store(), paths.cache())?;
        let cache = tisty_core::cache::Cache::open(paths.cache())?;
        let print = tisty_core::cache::fingerprint(&paths.store());

        Ok(Self {
            locale: config.locale.clone(),
            paths,
            config,
            state,
            store,
            cache,
            print,
        })
    }

    /// Read from disk before writing: the terminal edits the same file while
    /// the window is open, and saving a copy from startup would undo it.
    fn keep(&mut self, change: impl FnOnce(&mut Config)) -> Answer<()> {
        let mut fresh = Config::load(&self.paths.config_file())
            .ok()
            .flatten()
            .unwrap_or_else(|| self.config.clone());
        change(&mut fresh);
        fresh
            .save(&self.paths)
            .map_err(|e| Refusal::about("internal", e.to_string()))?;
        self.config = fresh;
        Ok(())
    }

    fn reload(&mut self) -> tisty_core::Result<bool> {
        let print = tisty_core::cache::fingerprint(&self.paths.store());
        if print == self.print {
            return Ok(false);
        }
        self.state = tisty_core::cache::project(&self.paths.store(), self.paths.cache())?;
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
    // Folded work must still say it is there, or putting it away is losing it.
    count(
        "folded",
        Filter {
            scope: Scope::Archived,
            hidden: true,
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
            // Folded work is not counted, or the chip promises more than the
            // list delivers.
            tasks: state.tasks_tagged(tag).filter(|t| !t.hidden).count(),
        })
        .collect()
}

#[derive(serde::Serialize)]
struct Snapshot {
    tasks: Vec<Task>,
    lists: Vec<List>,
    tags: Vec<Counted>,
    /// Internal references already in use, for the `/` menu to offer back.
    refs: Vec<String>,
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
    hidden: bool,
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
            hidden: self.hidden,
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
        refs: session.state.references(),
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

/// A bare day first, or `civil::DateTime` reads it as midnight and every date
/// picked in the calendar comes back stamped «00:00».
fn dated(raw: &str, now: &jiff::Zoned, spoken: &str) -> Result<tisty_core::DateSpec, Refusal> {
    if let Ok(day) = raw.parse::<jiff::civil::Date>() {
        return Ok(tisty_core::DateSpec::all_day(day, zone()));
    }
    if let Ok(when) = raw.parse::<jiff::civil::DateTime>() {
        return Ok(tisty_core::DateSpec::floating(when, zone()));
    }
    tisty_nl::parse_date(raw, now, spoken).ok_or_else(|| Refusal::about("notADate", raw))
}

/// A deadline or a reminder in the past is not a mistake to store and explain
/// later: it can never fire and it can never be met.
fn ahead(
    spec: &tisty_core::DateSpec,
    now: &jiff::Zoned,
    code: &'static str,
) -> Result<(), Refusal> {
    let passed = if spec.has_time {
        spec.at < now.datetime()
    } else {
        spec.date() < now.date()
    };
    if passed {
        return Err(Refusal::of(code));
    }
    Ok(())
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
            && let Some(list) = &view.list
        {
            let id = list.parse().map_err(|_| Refusal::of("notAListId"))?;
            draft.filing = Some(tisty_core::capture::Filing::Kept(id));
        }
        for name in &view.tags {
            if let Ok(tag) = Tag::new(name)
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
    if let Some(spec) = &draft.deadline {
        ahead(spec, &now, "pastDeadline")?;
    }
    if let Some(title) = edits.retitled(&text, &read) {
        draft.title = title;
    }

    let plan = tisty_core::capture::plan(&session.state, draft)?;
    session.commit_all(plan.ops)?;
    session
        .state
        .tasks
        .get(&plan.task)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
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
    add_tag: Option<String>,
    #[serde(default)]
    untag: Option<String>,
    #[serde(default)]
    list: Option<String>,
    #[serde(default)]
    inbox: bool,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    remind: Option<String>,
    #[serde(default)]
    unremind: Option<String>,
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
    let task = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?
        .clone();

    let d = TaskPatch {
        title: change
            .title
            .as_deref()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty()),
        date: dated_field(change.date.as_deref(), change.no_date, &now, &spoken)?,
        deadline: {
            let field = dated_field(
                change.deadline.as_deref(),
                change.no_deadline,
                &now,
                &spoken,
            )?;
            if let Some(Some(spec)) = &field {
                ahead(spec, &now, "pastDeadline")?;
            }
            field
        },
        priority: change
            .priority
            .map(tisty_core::Priority::try_from)
            .transpose()
            .map_err(|_| Refusal::of("notAPriority"))?,
        tags: tagged(&task, &change)?,
        reminders: recalled(&task, &change, &now)?,
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
    if let Some(body) = &change.description {
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
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

/// Built from the task as it is now, not from a vector the window sent: two
/// quick clicks would otherwise undo each other.
fn tagged(task: &Task, change: &Change) -> Result<Option<Vec<Tag>>, Refusal> {
    if change.add_tag.is_none() && change.untag.is_none() {
        return Ok(None);
    }
    let mut tags = task.tags.clone();
    if let Some(name) = &change.untag {
        let gone = Tag::new(name).map_err(|_| Refusal::about("badTag", name))?;
        tags.retain(|kept| *kept != gone);
    }
    if let Some(name) = &change.add_tag {
        let one = Tag::new(name).map_err(|_| Refusal::about("badTag", name))?;
        if !tags.contains(&one) {
            tags.push(one);
        }
    }
    Ok(Some(tags))
}

fn recalled(
    task: &Task,
    change: &Change,
    now: &jiff::Zoned,
) -> Result<Option<Vec<tisty_core::DateSpec>>, Refusal> {
    if change.remind.is_none() && change.unremind.is_none() {
        return Ok(None);
    }
    let civil = |raw: &String| {
        raw.parse::<jiff::civil::DateTime>()
            .map_err(|_| Refusal::about("notADate", raw))
    };
    let mut at = task.reminders.clone();
    if let Some(raw) = &change.unremind {
        let gone = civil(raw)?;
        at.retain(|kept| kept.at != gone);
    }
    if let Some(raw) = &change.remind {
        let when = civil(raw)?;
        if when < now.datetime() {
            return Err(Refusal::of("pastReminder"));
        }
        if !at.iter().any(|kept| kept.at == when) {
            at.push(tisty_core::DateSpec::floating(when, zone()));
        }
    }
    at.sort_by_key(|one| one.at);
    Ok(Some(at))
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

#[tauri::command]
fn move_step(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: String,
    after: Option<String>,
    before: Option<String>,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let step = step.parse().map_err(|_| Refusal::of("notAStepId"))?;
    let neighbour = |raw: Option<String>| raw.and_then(|one| one.parse().ok());

    let mut session = held(&session);
    let order = session
        .state
        .step_order_between(id, neighbour(after), neighbour(before));

    session.commit(Op::StepReorder {
        id,
        d: tisty_core::event::StepReorder { step, order },
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
fn write_step(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: Option<String>,
    text: String,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(Refusal::of("emptyStep"));
    }
    let mut session = held(&session);

    let op = match step {
        Some(raw) => Op::StepText {
            id,
            d: StepText {
                step: raw.parse().map_err(|_| Refusal::of("notAStepId"))?,
                text,
            },
        },
        None => Op::StepAdd {
            id,
            d: StepAdd {
                step: ulid::Ulid::generate(),
                text,
                order: tisty_core::order::last_of(
                    session
                        .state
                        .tasks
                        .get(&id)
                        .ok_or_else(|| Refusal::of("notATaskId"))?
                        .steps
                        .iter()
                        .map(|s| s.order.as_str()),
                ),
            },
        },
    };
    session.commit(op)?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
fn mark_step(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    step: String,
    done: bool,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let d = StepRef {
        step: step.parse().map_err(|_| Refusal::of("notAStepId"))?,
    };
    let mut session = held(&session);
    session.commit(if done {
        Op::StepDone { id, d }
    } else {
        Op::StepUndone { id, d }
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
fn drop_step(session: tauri::State<'_, Mutex<Session>>, id: String, step: String) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let d = StepRef {
        step: step.parse().map_err(|_| Refusal::of("notAStepId"))?,
    };
    let mut session = held(&session);
    session.commit(Op::StepRemove { id, d })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
fn write_log(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    entry: Option<String>,
    body: String,
) -> Answer<Task> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(Refusal::of("emptyEntry"));
    }
    let mut session = held(&session);

    session.commit(match entry {
        Some(raw) => Op::TaskLogEdit {
            id,
            d: LogEdit {
                entry: raw.parse().map_err(|_| Refusal::of("notAnEntry"))?,
                body,
            },
        },
        None => Op::TaskLog {
            id,
            d: LogAdd::new(ulid::Ulid::generate(), body).in_zone(Some(zone())),
        },
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
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
fn discard(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.commit(Op::TaskDrop { id })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
fn fold(session: tauri::State<'_, Mutex<Session>>, id: String, away: bool) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.commit(if away {
        Op::TaskHide { id }
    } else {
        Op::TaskShow { id }
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Carrying {
    chosen: Option<String>,
    asked: bool,
    backs_up: bool,
    last: Option<String>,
    loose: usize,
}

#[tauri::command]
fn checked(session: tauri::State<'_, Mutex<Session>>) -> Answer<Reviewed> {
    let session = held(&session);
    let audit = tisty_core::cache::audit(&session.paths.store(), session.paths.cache())
        .map_err(|_| Refusal::of("internal"))?;

    let held: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    let adrift = tisty_core::attach::loose(session.paths.data(), &held);

    Ok(Reviewed {
        tasks: session.state.tasks.len(),
        lists: session.state.lists.len(),
        agrees: matches!(audit, tisty_core::cache::Audit::Agrees { .. }),
        loose: adrift.files,
        loose_bytes: adrift.bytes,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Reviewed {
    tasks: usize,
    lists: usize,
    agrees: bool,
    loose: usize,
    loose_bytes: u64,
}

#[tauri::command]
fn sync_state(session: tauri::State<'_, Mutex<Session>>) -> Answer<Carrying> {
    let session = held(&session);
    let config = &session.config;

    let held: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();

    Ok(Carrying {
        chosen: match &config.sync {
            Some(tisty_core::config::Sync::Folder(at)) => Some(at.display().to_string()),
            _ => None,
        },
        asked: config.sync.is_some(),
        backs_up: config.backs_up(),
        last: config.synced_at.map(|at| at.to_string()),
        loose: tisty_core::attach::loose(session.paths.data(), &held).files,
    })
}

#[tauri::command]
fn choose_sync(session: tauri::State<'_, Mutex<Session>>, dest: Option<String>) -> Answer<()> {
    let mut session = held(&session);
    let chosen = match dest
        .map(|one| one.trim().to_string())
        .filter(|one| !one.is_empty())
    {
        Some(dest) => tisty_core::config::Sync::Folder(dest.into()),
        None => tisty_core::config::Sync::Local,
    };
    session.keep(|c| c.sync = Some(chosen))
}

#[tauri::command]
async fn sync_now(session: tauri::State<'_, Mutex<Session>>, way: Option<String>) -> Answer<bool> {
    let (dest, root, device) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            dest,
            session.paths.store(),
            session.config.device_id.0.clone(),
        )
    };

    let before = tisty_core::cache::fingerprint(&root);
    let way = match way.as_deref() {
        Some("push") => tisty_sync::Way::Push,
        Some("pull") => tisty_sync::Way::Pull,
        _ => tisty_sync::Way::Both,
    };

    let here = root.clone();
    tauri::async_runtime::spawn_blocking(move || tisty_sync::carry(&here, &device, &dest, way))
        .await
        .map_err(|_| Refusal::of("internal"))?
        .map_err(said)?;

    let mut session = held(&session);
    session.keep(|c| c.synced_at = Some(jiff::Timestamp::now()))?;
    let moved = tisty_core::cache::fingerprint(&root) != before;
    if moved {
        session.reload()?;
    }
    Ok(moved)
}

#[tauri::command]
fn back_up(session: tauri::State<'_, Mutex<Session>>, into: String) -> Answer<u64> {
    let session = held(&session);
    if !session.config.backs_up() {
        return Err(Refusal::of("sharedIsTheBackup"));
    }
    tisty_core::backup::write(session.paths.data(), std::path::Path::new(&into))
        .map(|made| made.bytes)
        .map_err(|_| Refusal::about("cannotWrite", into))
}

#[tauri::command]
fn restore(session: tauri::State<'_, Mutex<Session>>, from: String) -> Answer<usize> {
    let mut session = held(&session);
    if !session.config.backs_up() {
        return Err(Refusal::of("sharedIsTheBackup"));
    }
    let done = tisty_core::backup::read(&session.paths.clone(), std::path::Path::new(&from))
        .map_err(|e| match e {
            tisty_core::Error::OtherStore { theirs } => Refusal::about("otherStore", theirs),
            _ => Refusal::about("cannotRead", from.clone()),
        })?;
    session.reload()?;
    Ok(done.files)
}

fn said(trouble: tisty_sync::Trouble) -> Refusal {
    match trouble {
        tisty_sync::Trouble::NotThere(at) => Refusal::about("noMeetingPlace", at),
        tisty_sync::Trouble::OtherStore { theirs } => Refusal::about("otherStore", theirs),
        tisty_sync::Trouble::Unreadable(why) => Refusal::about("syncUnreadable", why),
    }
}

/// Guarded by `attach::resolve`: without it a description could name any file
/// on the disk and the window would show it.
#[tauri::command]
fn served(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<String> {
    let root = held(&session).paths.data().to_path_buf();
    let at = tisty_core::attach::resolve(&reference, &root)
        .map_err(|_| Refusal::about("cannotRead", reference.clone()))?;
    if !at.is_file() {
        return Err(Refusal::about("cannotRead", reference));
    }
    Ok(at.to_string_lossy().into_owned())
}

/// Uses the free function, not the plugin command: its scope is empty and would
/// refuse every path. `attach::resolve` is the guard, and a tighter one.
#[tauri::command]
fn opened(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<()> {
    let root = held(&session).paths.data().to_path_buf();
    let at = tisty_core::attach::resolve(&reference, &root)
        .map_err(|_| Refusal::about("cannotRead", reference.clone()))?;
    if !at.is_file() {
        return Err(Refusal::about("cannotRead", reference));
    }
    // Never launched, only shown: a description can arrive from another machine.
    if runnable(&at) {
        return show(&at, &reference);
    }
    tauri_plugin_opener::open_path(at, None::<&str>)
        .map_err(|_| Refusal::about("cannotOpen", reference))?;
    let _ = app;
    Ok(())
}

/// A file the store does not hold: over the threshold only its path was kept,
/// so it is shown in its folder rather than opened from a path we cannot vouch for.
#[tauri::command]
fn revealed(path: String) -> Answer<()> {
    show(std::path::Path::new(&path), &path)
}

fn show(at: &std::path::Path, said: &str) -> Answer<()> {
    tauri_plugin_opener::reveal_item_in_dir(at)
        .map_err(|_| Refusal::about("cannotOpen", said.to_string()))
}

fn runnable(at: &std::path::Path) -> bool {
    let ext = at
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_lowercase();
    matches!(
        ext.as_str(),
        "exe"
            | "bat"
            | "cmd"
            | "com"
            | "msi"
            | "scr"
            | "ps1"
            | "vbs"
            | "js"
            | "jar"
            | "sh"
            | "app"
            | "lnk"
            | "reg"
            | "hta"
    )
}

/// Does not touch the task: what makes it an attachment is the reference in the prose.
#[tauri::command]
fn attach(
    session: tauri::State<'_, Mutex<Session>>,
    path: String,
    label: Option<String>,
) -> Answer<String> {
    let source = std::path::PathBuf::from(&path);
    let name = label
        .or_else(|| {
            source
                .file_name()
                .and_then(|n| n.to_str())
                .map(str::to_string)
        })
        .unwrap_or_default();

    // The root is read and the lock released before any file work: copying
    // megabytes with the session held would freeze every other command.
    let root = held(&session).paths.data().to_path_buf();
    let kept = tisty_core::attach::keep(&source, &root, tisty_core::attach::COPIED_UP_TO)
        .map_err(|_| Refusal::about("cannotRead", name.clone()))?;

    Ok(kept.written(&name))
}

/// `after` and `before` are the tasks it was dropped between; `list` is only
/// sent when the drop crossed into another one, so a reorder never refiles.
#[tauri::command]
fn reorder(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    after: Option<String>,
    before: Option<String>,
    list: Option<String>,
    inbox: Option<bool>,
) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let neighbour = |raw: Option<String>| raw.and_then(|one| one.parse().ok());

    let filed = match (list, inbox) {
        (Some(one), _) => Some(Some(one.parse().map_err(|_| Refusal::of("notAListId"))?)),
        (None, Some(true)) => Some(None),
        _ => None,
    };

    let mut session = held(&session);
    // With no neighbours the midpoint of nothing is always the same key — the
    // first position that ever existed — so every filed task would pile there.
    let order = match (neighbour(after), neighbour(before)) {
        (None, None) => session.state.order_last_in(filed.flatten()),
        (a, b) => session.state.order_between(a, b),
    };

    session.commit(Op::TaskMove {
        id,
        d: tisty_core::event::TaskMove {
            list: filed,
            order: Some(order),
        },
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
fn complete(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.commit(Op::TaskDone { id })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[tauri::command]
fn reopen(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.commit(Op::TaskReopen { id })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let session = Session::open().expect("could not open the store");

    // The store lives outside the bundle, so the asset scope has to be opened
    // at runtime: the path is only known once the data directory is resolved.
    let attachments = session.paths.attachments();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            let _ = std::fs::create_dir_all(&attachments);
            app.handle()
                .asset_protocol_scope()
                .allow_directory(&attachments, true)?;
            Ok(())
        })
        .manage(Mutex::new(session))
        .invoke_handler(tauri::generate_handler![
            snapshot,
            capture,
            read,
            search,
            complete,
            reopen,
            patch,
            write_step,
            mark_step,
            drop_step,
            write_log,
            fold,
            discard,
            reorder,
            attach,
            served,
            opened,
            revealed,
            move_step,
            sync_state,
            choose_sync,
            sync_now,
            back_up,
            restore,
            checked
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

    /// The window sends the list it is looking at as an id; routing that through
    /// the name lookup refused every capture inside a list.
    #[test]
    fn a_capture_inside_a_list_is_filed_by_id() {
        let mut state = State::default();
        let list = ulid::Ulid::generate();
        state.apply(&tisty_core::Event::new(
            tisty_core::event::DeviceId("dev".into()),
            jiff::Timestamp::now(),
            Op::ListAdd {
                id: list,
                d: tisty_core::event::ListAdd {
                    name: "unificación de login".into(),
                    color: None,
                    order: "a0".into(),
                },
            },
        ));

        let mut draft: tisty_core::capture::Draft =
            tisty_nl::parse("revisar el deploy", &now(), "es").into();
        draft.filing = Some(tisty_core::capture::Filing::Kept(list));

        let plan = tisty_core::capture::plan(&state, draft).expect("filed");
        assert!(matches!(plan.ops.first(), Some(Op::TaskAdd { d, .. }) if d.list == Some(list)));
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
