use std::sync::Mutex;

mod command;
mod herald;
mod tray;

use tauri::{Emitter, Manager};

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
            .map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
        self.config = fresh;
        Ok(())
    }

    fn reload(&mut self) -> tisty_core::Result<bool> {
        let print = tisty_core::cache::fingerprint(&self.paths.store());
        if print == self.print {
            return Ok(false);
        }
        self.reproject()?;
        Ok(true)
    }

    /// A local write during a pull already folded the arrived files into the
    /// fingerprint, so comparing it would report «nothing new» and hide them.
    fn reproject(&mut self) -> tisty_core::Result<()> {
        self.state = tisty_core::cache::project(&self.paths.store(), self.paths.cache())?;
        self.print = tisty_core::cache::fingerprint(&self.paths.store());
        Ok(())
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
        Refusal::about("internalNamed", error.to_string())
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
fn snapshot(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    view: Option<View>,
) -> Answer<Snapshot> {
    let mut session = held(&session);
    session.reload()?;
    // Reread from disk on every snapshot, so a language changed from the
    // terminal reaches the tray as well as the window.
    let spoken = Config::load(&session.paths.config_file())
        .ok()
        .flatten()
        .and_then(|c| c.locale);
    if spoken != session.locale {
        session.locale = spoken.clone();
        language(&app, &spoken);
    }

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
    no_repeat: bool,
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
        if self.no_repeat {
            draft.repeat = None;
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
            || self.no_repeat
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
            tisty_nl::Mark::Repeat => self.no_repeat,
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
    app: tauri::AppHandle,
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
    let task = session
        .state
        .tasks
        .get(&plan.task)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    drop(session);
    herald::told(
        &app,
        tisty_core::herald::Happening::Filed {
            title: task.title.clone(),
        },
    );
    Ok(task)
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
    #[serde(default)]
    repeat: Option<tisty_core::model::Repeat>,
    #[serde(default)]
    no_repeat: bool,
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
        repeat: repeated(&change)?,
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

/// A cadence of zero would make the next occurrence land on the same day for
/// ever; the parser refuses it too.
fn repeated(change: &Change) -> Result<Option<Option<tisty_core::model::Repeat>>, Refusal> {
    if change.no_repeat {
        return Ok(Some(None));
    }
    let Some(over) = change.repeat else {
        return Ok(None);
    };
    let every = over.cadence().every;
    if every == 0 || every > 999 {
        return Err(Refusal::of("notACadence"));
    }
    Ok(Some(Some(over)))
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

/// Reporting that the cache disagrees without offering to redo it leaves the
/// only screen you go to when something is wrong with nothing to press.
#[tauri::command(async)]
fn rebuild(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    let mut session = held(&session);
    tisty_core::cache::discard(session.paths.cache())
        .map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
    session.cache = tisty_core::cache::Cache::open(session.paths.cache())
        .map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
    session
        .reproject()
        .map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
    Ok(())
}

#[tauri::command(async)]
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

/// The terminal can change the language while the window is open.
fn language<R: tauri::Runtime>(app: &tauri::AppHandle<R>, locale: &Option<String>) {
    tray::reword(
        app,
        &tray::Words {
            show: worded(locale, "show"),
            capture: worded(locale, "capture"),
            quit: worded(locale, "quit"),
        },
    );
}

/// Sync before reviewing: a store about to change would be reviewed for a
/// state nobody keeps.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Settling {
    ran: bool,
    brought: bool,
    agrees: bool,
    was: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct About {
    version: String,
    repository: &'static str,
    license: &'static str,
    /// Where the log actually lives, so a report can say it without guessing.
    store: String,
}

/// Nobody could say which version they hit a problem with: it existed only in
/// `CARGO_PKG_VERSION` and was never shown anywhere in the window.
#[tauri::command]
fn about(session: tauri::State<'_, Mutex<Session>>) -> Answer<About> {
    let session = held(&session);
    Ok(About {
        version: env!("CARGO_PKG_VERSION").to_string(),
        repository: "https://github.com/rgdevment/Tisty",
        license: "AGPL-3.0-only",
        store: session.paths.store().display().to_string(),
    })
}

#[tauri::command]
async fn settle_in(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
) -> Answer<Settling> {
    let here = env!("CARGO_PKG_VERSION");
    let (was, dest, data, store, device) = {
        let session = held(&session);
        let was = session.config.opened_by.clone();
        if was.as_deref() == Some(here) {
            return Ok(Settling {
                ran: false,
                brought: false,
                agrees: true,
                was,
            });
        }
        let dest = match &session.config.sync {
            Some(tisty_core::config::Sync::Folder(at)) => Some(at.clone()),
            _ => None,
        };
        (
            was,
            dest,
            session.paths.data().to_path_buf(),
            session.paths.store(),
            session.config.device_id.0.clone(),
        )
    };

    let mut brought = false;
    let mut carried = dest.is_none();
    if let Some(dest) = dest
        && let Some(_done) = alone.inner().claim()
    {
        carried = true;
        let before = tisty_core::cache::fingerprint(&store);
        // A folder that is not there is not a reason to refuse to start.
        let _ = tauri::async_runtime::spawn_blocking(move || {
            tisty_sync::carry(
                &data,
                &device,
                &dest,
                tisty_sync::Way::Both,
                tisty_sync::Join::Ask,
            )
        })
        .await;
        brought = tisty_core::cache::fingerprint(&store) != before;
    }

    let mut session = held(&session);
    if brought {
        session
            .reproject()
            .map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
    }
    // A version that changed how the cache is built cannot be trusted with it.
    let audit = tisty_core::cache::audit(&session.paths.store(), session.paths.cache())
        .map_err(|_| Refusal::of("internal"))?;
    let agrees = matches!(audit, tisty_core::cache::Audit::Agrees { .. });
    if !agrees {
        let _ = std::fs::remove_dir_all(session.paths.cache());
        session
            .reproject()
            .map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
    }

    // Not sealed when the carry never ran — the automatic one had the lock —
    // or the next start would skip settling in for good.
    if carried {
        session.keep(|c| c.opened_by = Some(here.to_string()))?;
    }
    Ok(Settling {
        ran: true,
        brought,
        agrees,
        was,
    })
}

#[tauri::command]
fn reachable() -> command::Reach {
    command::reach()
}

/// Here and not in the installer, which read the value truncated and wiped it.
#[tauri::command]
fn reach_for(wanted: bool) -> Answer<command::Reach> {
    command::within_reach(wanted).map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    Ok(command::reach())
}

#[tauri::command]
fn shortcut(bound: tauri::State<'_, Bound>) -> Option<String> {
    bound.0.clone()
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
        Some(dest) => {
            let at = std::path::PathBuf::from(&dest);
            // The panel would count them loose, and each backup carry itself.
            let data = session.paths.data();
            let tangled = at.starts_with(data)
                || data.starts_with(&at)
                || at
                    .canonicalize()
                    .ok()
                    .zip(data.canonicalize().ok())
                    .is_some_and(|(a, b)| a.starts_with(&b) || b.starts_with(&a));
            if tangled {
                return Err(Refusal::about("remoteInsideStore", dest));
            }
            tisty_core::config::Sync::Folder(at)
        }
        None => tisty_core::config::Sync::Local,
    };
    session.keep(|c| c.sync = Some(chosen))
}

/// `None` is «not now»: unanswered, so the question comes again.
#[tauri::command]
fn close_window(
    window: tauri::Window,
    session: tauri::State<'_, Mutex<Session>>,
    how: Option<String>,
    remember: Option<bool>,
) -> Answer<()> {
    let how = match how.as_deref() {
        Some("hide") => tisty_core::config::Closing::Hide,
        Some("quit") => tisty_core::config::Closing::Quit,
        _ => return Ok(()),
    };

    if remember == Some(true) {
        held(&session).keep(|c| c.on_close = Some(how))?;
    }
    match how {
        tisty_core::config::Closing::Hide => {
            let _ = window.hide();
        }
        tisty_core::config::Closing::Quit => window.app_handle().exit(0),
    }
    Ok(())
}

#[tauri::command]
async fn sync_now(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    way: Option<String>,
    merge: Option<bool>,
) -> Answer<&'static str> {
    // Busy is not «nothing new»: reporting the second as the first tells the
    // person a sync happened when what happened was that one was already going.
    let Some(_done) = alone.inner().claim() else {
        return Ok("busy");
    };

    let (dest, data, store, device) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            dest,
            session.paths.data().to_path_buf(),
            session.paths.store(),
            session.config.device_id.0.clone(),
        )
    };

    let before = tisty_core::cache::fingerprint(&store);
    let way = match way.as_deref() {
        Some("push") => tisty_sync::Way::Push,
        Some("pull") => tisty_sync::Way::Pull,
        _ => tisty_sync::Way::Both,
    };

    let join = if merge == Some(true) {
        tisty_sync::Join::Agreed
    } else {
        tisty_sync::Join::Ask
    };
    tauri::async_runtime::spawn_blocking(move || {
        tisty_sync::carry(&data, &device, &dest, way, join)
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(said)?;

    let mut session = held(&session);
    let moved = tisty_core::cache::fingerprint(&store) != before;
    if moved {
        session
            .reproject()
            .map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
    }
    // Last: «synced a moment ago» over a store that will not project is a lie.
    session.keep(|c| c.synced_at = Some(jiff::Timestamp::now()))?;
    Ok(if moved { "came" } else { "same" })
}

/// Off the main thread: this window has no system title bar to keep dragging.
#[tauri::command]
async fn back_up(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    // The same lock as carrying: a restore renaming the store while a carry
    // writes into it is the pair that must never overlap.
    let _done = alone.inner().taken()?;
    let data = {
        let session = held(&session);
        if !session.config.backs_up() {
            return Err(Refusal::of("sharedIsTheBackup"));
        }
        session.paths.data().to_path_buf()
    };

    let at = std::path::PathBuf::from(&into);
    tauri::async_runtime::spawn_blocking(move || tisty_core::backup::write(&data, &at))
        .await
        .map_err(|_| Refusal::of("internal"))?
        .map(|made| made.bytes)
        .map_err(|_| Refusal::about("cannotWrite", into))
}

#[tauri::command]
async fn restore(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    from: String,
) -> Answer<usize> {
    let _done = alone.inner().taken()?;
    let paths = {
        let session = held(&session);
        if !session.config.backs_up() {
            return Err(Refusal::of("sharedIsTheBackup"));
        }
        session.paths.clone()
    };

    let at = std::path::PathBuf::from(&from);
    let done = tauri::async_runtime::spawn_blocking(move || tisty_core::backup::read(&paths, &at))
        .await
        .map_err(|_| Refusal::of("internal"))?
        .map_err(|e| match e {
            tisty_core::Error::OtherStore { theirs } => Refusal::about("otherStore", theirs),
            tisty_core::Error::Io(why) => Refusal::about("restoreFailed", why.to_string()),
            _ => Refusal::about("cannotRead", from.clone()),
        })?;

    // Not `reload`: the restore minted a new device id, and a Store still open
    // on the old one would write into a directory that just went back in time.
    *held(&session) =
        Session::open().map_err(|e| Refusal::about("internalNamed", e.to_string()))?;
    Ok(done.files)
}

struct Releasing<'a>(&'a std::sync::atomic::AtomicBool);

impl Drop for Releasing<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
    }
}

fn said(trouble: tisty_sync::Trouble) -> Refusal {
    match trouble {
        tisty_sync::Trouble::NotThere(at) => Refusal::about("noMeetingPlace", at),
        tisty_sync::Trouble::OtherStore { theirs } => Refusal::about("otherStore", theirs),
        tisty_sync::Trouble::Unreadable(why) => Refusal::about("syncUnreadable", why),
        tisty_sync::Trouble::Broke(why) => Refusal::about("syncBroke", why),
        tisty_sync::Trouble::WouldMerge { theirs } => Refusal::about("wouldMerge", theirs),
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
    let ops = session.state.completing(id, jiff::Zoned::now());
    session.commit_all(ops)?;
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
    let ops = session.state.reopening(id);
    session.commit_all(ops)?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

struct Perched(bool);

/// Which combination answered, so the window can say it instead of leaving the
/// person pressing keys that belong to their editor.
struct Bound(Option<String>);

/// A shortcut another program already holds is not an error worth stopping
/// for: the tray still opens the same window. Which one answered is worth
/// saying — `Ctrl+Shift+Space` is Trigger Parameter Hints in VS Code and Smart
/// Type Completion in the JetBrains IDEs, so it does get taken.
fn listen_for<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Option<String> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let tries = [
        (
            "Ctrl+Shift+Space",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space),
        ),
        (
            "Ctrl+Alt+Space",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space),
        ),
    ];

    for (said, combo) in tries {
        let handle = app.clone();
        let taken = app
            .global_shortcut()
            .on_shortcut(combo, move |_, _, event| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    tray::quicken(&handle);
                }
            });
        if taken.is_ok() {
            return Some(said.to_string());
        }
    }
    None
}

/// The tray menu is drawn before the window exists, so it cannot ask the
/// frontend for its words.
fn worded(locale: &Option<String>, key: &str) -> String {
    let spanish = locale
        .as_deref()
        .or(sys_locale::get_locale().as_deref())
        .is_some_and(|code| code.to_lowercase().starts_with("es"));

    match (key, spanish) {
        ("show", true) => "Abrir Tisty".into(),
        ("show", false) => "Open Tisty".into(),
        ("capture", true) => "Capturar…".into(),
        ("capture", false) => "Capture…".into(),
        ("due", true) => "Recordatorio".into(),
        ("due", false) => "Reminder".into(),
        (_, true) => "Salir de Tisty".into(),
        (_, false) => "Quit Tisty".into(),
    }
}

/// Only one at a time: the automatic carrier and the panel's button cannot see
/// each other, and two carries would copy the same files over one another.
#[derive(Default)]
struct OneAtATime(std::sync::atomic::AtomicBool);

impl OneAtATime {
    /// `None` while another one holds it: the caller decides whether that is
    /// «already going» or a reason to refuse.
    fn claim(&self) -> Option<Releasing<'_>> {
        use std::sync::atomic::Ordering;
        self.0
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then(|| Releasing(&self.0))
    }

    fn taken(&self) -> Answer<Releasing<'_>> {
        self.claim().ok_or_else(|| Refusal::of("stillCarrying"))
    }
}

pub fn unreach() -> std::io::Result<bool> {
    command::within_reach(false)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            // Opened here, not before the builder: with syncing on, other
            // machines write this store, so failing to read it stopped being
            // impossible — and a window that dies without a word leaves
            // nothing to act on, while the path and the reason do.
            let session = match Session::open() {
                Ok(session) => session,
                Err(why) => {
                    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
                    for window in app.webview_windows().values() {
                        let _ = window.close();
                    }
                    app.dialog()
                        .message(why.to_string())
                        .kind(MessageDialogKind::Error)
                        .title("Tisty")
                        .blocking_show();
                    std::process::exit(1);
                }
            };

            // The store lives outside the bundle, so the asset scope has to be
            // opened at runtime: the path is only known once data is resolved.
            // A restore that died left the only copy of what it swapped out;
            // once the store reads, it is over and can go.
            for at in tisty_core::backup::leftovers(session.paths.data()) {
                let _ = std::fs::remove_dir_all(at);
            }

            let attachments = session.paths.attachments();
            let _ = std::fs::create_dir_all(&attachments);
            app.handle()
                .asset_protocol_scope()
                .allow_directory(&attachments, true)?;
            let words = tray::Words {
                show: worded(&session.locale, "show"),
                capture: worded(&session.locale, "capture"),
                quit: worded(&session.locale, "quit"),
            };
            let telling = herald::Words {
                due: worded(&session.locale, "due"),
            };
            app.manage(Mutex::new(session));
            app.manage(herald::heralds(app.handle(), telling));
            herald::watch(app.handle().clone());

            // No tray on this desktop means closing keeps its plain meaning,
            // and the preference is ignored rather than hiding the app away.
            let perched = tray::raise(app.handle(), &words).is_some();
            app.manage(Perched(perched));
            app.manage(Bound(listen_for(app.handle())));

            // Shown only now: the window is created before `setup` runs, and
            // projecting a long log would leave a blank frame on screen.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            // The quick window is hidden, never closed, and asking there would
            // put the question in front of a capture nobody finished.
            if window.label() != "main" {
                return;
            }
            // Windows 11 switches the taskbar theme while running, so the
            // colour art has to be chosen again, not only at startup.
            if let tauri::WindowEvent::ThemeChanged(_) = event {
                tray::repaint(window.app_handle());
                return;
            }

            let tauri::WindowEvent::CloseRequested { api, .. } = event else {
                return;
            };

            let app = window.app_handle();
            // The quick window is created at startup and never closed, so the
            // process outlives the main window: letting the close through
            // would leave an invisible Tisty holding the global shortcut.
            if !app.state::<Perched>().0 {
                app.exit(0);
                return;
            }

            let asked = held(&app.state::<Mutex<Session>>()).config.on_close;
            match asked {
                Some(tisty_core::config::Closing::Quit) => app.exit(0),
                Some(tisty_core::config::Closing::Hide) => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                // Asked once, where the answer matters, instead of buried in a
                // settings screen nobody opens.
                None => {
                    api.prevent_close();
                    let _ = window.emit("closing", ());
                }
            }
        })
        .manage(OneAtATime::default())
        .invoke_handler(tauri::generate_handler![
            snapshot,
            close_window,
            shortcut,
            settle_in,
            reachable,
            reach_for,
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
            checked,
            rebuild,
            about
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
