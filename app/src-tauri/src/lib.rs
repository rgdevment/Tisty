use std::sync::Mutex;

mod command;
mod herald;
mod report;
mod tray;
mod update;

use tauri::{Emitter, Manager};

use tisty_core::{
    Config, Event, List, Op, Paths, State, Store, Tag, Task,
    event::{LogAdd, LogEdit, StepAdd, StepRef, StepText, TaskPatch},
    view::{Filter, Scope, Window},
    witness::{self, Fact, channel},
};

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
        tisty_core::witness::keeps(
            tisty_core::witness::file(&paths),
            tisty_core::witness::wants_all(),
        );
        tisty_core::witness::catches(tisty_core::witness::channel::WINDOW);
        witness::note(
            channel::WINDOW,
            "the window opened",
            &[
                ("version", Fact::Id(env!("CARGO_PKG_VERSION").to_string())),
                (
                    "sandbox",
                    Fact::Word(if tisty_core::paths::profile().is_some() {
                        "yes"
                    } else {
                        "no"
                    }),
                ),
            ],
        );
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

    fn keep(&mut self, change: impl FnOnce(&mut Config)) -> Answer<()> {
        let mut fresh = match Config::load(&self.paths.config_file()) {
            Ok(Some(kept)) => kept,
            Ok(None) => self.config.clone(),
            Err(why) => {
                witness::warn(
                    channel::CONFIG,
                    "the settings could not be read before saving",
                    &[("why", Fact::Why(why.to_string()))],
                );
                self.config.clone()
            }
        };
        change(&mut fresh);
        fresh
            .save(&self.paths)
            .map_err(|e| blamed(channel::CONFIG, "the settings could not be saved", e))?;
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

    fn reproject(&mut self) -> tisty_core::Result<()> {
        self.state = tisty_core::cache::project(&self.paths.store(), self.paths.cache())?;
        self.print = tisty_core::cache::fingerprint(&self.paths.store());
        Ok(())
    }

    fn take_a_seat(&mut self) -> tisty_core::Result<()> {
        let who = self.config.device_id.clone();
        if tisty_core::store::ledger(self.paths.store())?
            .allowed
            .contains(&who)
        {
            return Ok(());
        }
        self.commit(Op::DeviceJoin { d: who })
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

fn tally(state: &State) -> std::collections::BTreeMap<String, usize> {
    let mut counts = std::collections::BTreeMap::new();
    let mut count = |key: &str, filter: Filter| {
        counts.insert(key.to_string(), state.matching(&filter, today()).len());
    };

    count(
        "tasks",
        Filter {
            window: Some(Window::Today),
            ..Default::default()
        },
    );
    count(
        "upcoming",
        Filter {
            window: Some(Window::After(today())),
            ..Default::default()
        },
    );
    count(
        "repeating",
        Filter {
            repeating: true,
            ..Default::default()
        },
    );
    count("all", Filter::default());
    count(
        "archive",
        Filter {
            scope: Scope::Archived,
            ..Default::default()
        },
    );
    count(
        "folded",
        Filter {
            scope: Scope::Archived,
            hidden: true,
            ..Default::default()
        },
    );

    counts.insert("tags".to_string(), state.tags().len());

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

fn tags_in_use(state: &State) -> Vec<Counted> {
    state
        .tags()
        .into_iter()
        .map(|tag| Counted {
            tag: tag.to_string(),
            tasks: state.tasks_tagged(tag).filter(|t| !t.hidden).count(),
        })
        .collect()
}

#[derive(serde::Serialize)]
struct Snapshot {
    tasks: Vec<Task>,
    lists: Vec<List>,
    tags: Vec<Counted>,
    refs: Vec<String>,
    counts: std::collections::BTreeMap<String, usize>,
    locale: Option<String>,
}

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
            Rejected::EndedAlready => Refusal::of("pastEnd"),
            Rejected::NoSuchList(name) => Refusal::about("noSuchList", name),
            Rejected::AmbiguousList(name) => Refusal::about("ambiguousList", name),
        }
    }
}

impl From<tisty_core::Error> for Refusal {
    fn from(error: tisty_core::Error) -> Self {
        blamed(channel::WINDOW, "a command could not finish", error)
    }
}

fn blamed(channel: &'static str, said: &'static str, error: tisty_core::Error) -> Refusal {
    witness::error(channel, said, &error.told());
    Refusal::about("internalNamed", error.to_string())
}

type Answer<T> = std::result::Result<T, Refusal>;

fn held<'a>(session: &'a tauri::State<'_, Mutex<Session>>) -> std::sync::MutexGuard<'a, Session> {
    session
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

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
    #[serde(default)]
    everything: bool,
    #[serde(default)]
    inbox: bool,
    #[serde(default)]
    list: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    tagged: bool,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    window: Option<String>,
    #[serde(default)]
    repeating: bool,
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
            repeating: self.repeating,
            window: match self.window.as_deref() {
                Some("today") => Some(Window::Today),
                Some("upcoming") => Some(Window::After(today())),
                Some("overdue") => Some(Window::Overdue),
                Some("undated") => Some(Window::Undated),
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

    fn retitled(&self, text: &str, read: &tisty_nl::Parsed, spoken: &str) -> Option<String> {
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
        Some(tisty_nl::title_without(text, &kept, spoken))
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

fn dated(raw: &str, now: &jiff::Zoned, spoken: &str) -> Result<tisty_core::DateSpec, Refusal> {
    if let Ok(day) = raw.parse::<jiff::civil::Date>() {
        return Ok(tisty_core::DateSpec::all_day(day, zone()));
    }
    if let Ok(when) = raw.parse::<jiff::civil::DateTime>() {
        return Ok(tisty_core::DateSpec::floating(when, zone()));
    }
    tisty_nl::parse_date(raw, now, spoken).ok_or_else(|| Refusal::about("notADate", raw))
}

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

    let edits = edits.unwrap_or_default();
    edits.apply(&mut draft, &now, &spoken)?;
    if let Some(spec) = &draft.deadline {
        ahead(spec, &now, "pastDeadline")?;
    }
    if let Some(title) = edits.retitled(&text, &read, &spoken) {
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
    let _ = herald::told(
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
    list_named: Option<String>,
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
        repeat: repeated(&change, &now)?,
    };

    let mut ops = Vec::new();
    let named = match change.list_named.as_deref().map(str::trim) {
        Some(name) if !name.is_empty() => Some(match session.state.list_called(name).as_slice() {
            [one] => one.id,
            [_, _, ..] => return Err(Refusal::about("manyLists", name)),
            [] => {
                let made = ulid::Ulid::generate();
                ops.push(Op::ListAdd {
                    id: made,
                    d: tisty_core::event::ListAdd {
                        name: name.to_string(),
                        order: session.state.next_list_order(),
                        color: None,
                    },
                });
                made
            }
        }),
        _ => None,
    };

    let filed = match (named, &change.list, change.inbox) {
        (Some(id), _, _) => Some(Some(id)),
        (None, Some(raw), _) => Some(Some(raw.parse().map_err(|_| Refusal::of("notAListId"))?)),
        (None, None, true) => Some(None),
        _ => None,
    };

    if d != TaskPatch::default() {
        ops.push(Op::TaskUpdate { id, d });
    }
    if let Some(body) = &change.description {
        let kept = body.trim().to_string();
        tisty_core::state::short_enough(&kept).map_err(|e| match e {
            tisty_core::Error::TextTooLong { limit, .. } => {
                Refusal::about("textTooLong", weighed(limit))
            }
            _ => Refusal::of("internal"),
        })?;
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

fn repeated(
    change: &Change,
    now: &jiff::Zoned,
) -> Result<Option<Option<tisty_core::model::Repeat>>, Refusal> {
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
    if over.ended(now.date()) {
        return Err(Refusal::of("pastEnd"));
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
    tisty_core::state::short_enough(&body).map_err(|e| match e {
        tisty_core::Error::TextTooLong { limit, .. } => {
            Refusal::about("textTooLong", weighed(limit))
        }
        _ => Refusal::of("internal"),
    })?;
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

#[tauri::command]
fn search(
    session: tauri::State<'_, Mutex<Session>>,
    query: String,
    scope: Option<String>,
) -> Answer<Found> {
    let mut session = held(&session);
    session.reload()?;

    let scope = match scope.as_deref() {
        Some("open") => Scope::Open,
        Some("archived") => Scope::Archived,
        _ => Scope::Either,
    };
    let (hits, total) = session.state.searching(&query, scope, MOST);
    Ok(Found {
        tasks: hits.into_iter().cloned().collect(),
        total,
    })
}

const MOST: usize = 200;

#[derive(serde::Serialize)]
struct Found {
    tasks: Vec<Task>,
    total: usize,
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

#[tauri::command]
fn note_break(kind: String, frames: String) {
    let cut = |text: String, most: usize| text.chars().take(most).collect::<String>();
    witness::error(
        channel::WINDOW,
        "the window broke and stopped drawing",
        &[
            ("kind", Fact::Why(cut(kind, 40))),
            ("frames", Fact::Why(cut(frames, 400))),
        ],
    );
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Carrying {
    chosen: Option<String>,
    asked: bool,
    backs_up: bool,
    last: Option<String>,
    loose: usize,
    open: usize,
    archived: usize,
    lists: usize,
    attachments: usize,
    weight: u64,
    backed_up_at: Option<String>,
}

#[tauri::command(async)]
fn rebuild(session: tauri::State<'_, Mutex<Session>>) -> Answer<()> {
    let mut session = held(&session);
    tisty_core::cache::discard(session.paths.cache())
        .map_err(|e| blamed(channel::CACHE, "the cache could not be thrown away", e))?;
    session.cache = tisty_core::cache::Cache::open(session.paths.cache())
        .map_err(|e| blamed(channel::CACHE, "the cache could not be opened", e))?;
    session
        .reproject()
        .map_err(|e| blamed(channel::CACHE, "the store would not project", e))?;
    Ok(())
}

#[tauri::command(async)]
fn checked(session: tauri::State<'_, Mutex<Session>>) -> Answer<Reviewed> {
    let session = held(&session);
    let audit =
        tisty_core::cache::audit(&session.paths.store(), session.paths.cache()).map_err(|e| {
            witness::error(
                channel::CACHE,
                "the cache could not be audited",
                &[("why", Fact::Why(e.to_string()))],
            );
            Refusal::of("internal")
        })?;

    let mut held: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    held.extend(tisty_core::docs::referenced(&session.paths.docs()));
    let adrift = tisty_core::attach::loose(session.paths.data(), &held);

    let kept = report::attachments(session.paths.data());

    Ok(Reviewed {
        tasks: session.state.tasks.len(),
        lists: session.state.lists.len(),
        agrees: matches!(audit, tisty_core::cache::Audit::Agrees { .. }),
        loose: adrift.files(),
        loose_bytes: adrift.bytes,
        astray: adrift.items,
        events: tisty_core::store::read_all(session.paths.store())
            .map(|all| all.len())
            .unwrap_or(0),
        machines: report::machines(&session.paths.store(), session.config.device_id.0.as_str()),
        log_bytes: report::weighed(&session.paths.store()),
        docs_bytes: report::weighed(&session.paths.docs()),
        held_bytes: kept.bytes,
        held_files: kept.files,
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
    astray: Vec<tisty_core::attach::Astray>,
    events: usize,
    machines: Vec<report::Machine>,
    log_bytes: u64,
    docs_bytes: u64,
    held_bytes: u64,
    held_files: usize,
}

#[tauri::command(async)]
fn facts(
    session: tauri::State<'_, Mutex<Session>>,
    bound: tauri::State<'_, Bound>,
    names: bool,
    paths: bool,
) -> Answer<report::Facts> {
    let session = held(&session);
    let store = session.paths.store();
    let audit = tisty_core::cache::audit(&store, session.paths.cache());

    let mut referenced: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    referenced.extend(tisty_core::docs::referenced(&session.paths.docs()));
    let adrift = tisty_core::attach::loose(session.paths.data(), &referenced);
    let kept = report::attachments(session.paths.data());

    let shown = |raw: String| if paths { raw } else { report::hidden(&raw) };
    let lists = session.state.ordered_lists();
    let tags = session.state.tags();

    Ok(report::Facts {
        version: env!("CARGO_PKG_VERSION").to_string(),
        dev: cfg!(debug_assertions),
        sandbox: tisty_core::paths::profile(),
        locale: session
            .locale
            .clone()
            .or_else(sys_locale::get_locale)
            .unwrap_or_else(|| "?".into()),
        zone: jiff::tz::TimeZone::system()
            .iana_name()
            .unwrap_or("?")
            .to_string(),
        os: report::os(),
        arch: std::env::consts::ARCH,
        webview: tauri::webview_version().ok(),
        store: shown(store.display().to_string()),
        devices: report::devices(&store),
        events: tisty_core::store::read_all(&store)
            .map(|all| all.len())
            .unwrap_or(0),
        open: session.state.matching(&Filter::default(), today()).len(),
        archived: session
            .state
            .matching(
                &Filter {
                    scope: Scope::Archived,
                    ..Default::default()
                },
                today(),
            )
            .len(),
        lists: lists.len(),
        tags: tags.len(),
        list_names: if names {
            lists.iter().map(|one| one.name.clone()).collect()
        } else {
            Vec::new()
        },
        tag_names: if names {
            tags.iter().map(|one| one.to_string()).collect()
        } else {
            Vec::new()
        },
        cache: match audit {
            Ok(tisty_core::cache::Audit::Agrees { .. }) => "agrees",
            Ok(tisty_core::cache::Audit::Stale { .. }) => "stale",
            Ok(tisty_core::cache::Audit::Diverged { .. }) => "diverged",
            _ => "none",
        },
        attachments: kept.files,
        attachment_bytes: kept.bytes,
        loose: adrift.files(),
        loose_bytes: adrift.bytes,
        weight: report::weighed(session.paths.data()),
        syncs: session.config.sync.is_some(),
        shared: !session.config.backs_up(),
        backed_up_at: session.config.backed_up_at.map(|at| at.to_string()),
        quiet: session.config.muted().to_vec(),
        attach_up_to: session.config.copies_up_to(),
        in_path: command::reach().within_reach,
        shortcut: bound.0.clone(),
    })
}

#[tauri::command(async)]
fn keep_report(
    session: tauri::State<'_, Mutex<Session>>,
    at: String,
    text: String,
    logs: bool,
) -> Answer<()> {
    let path = std::path::PathBuf::from(&at);
    if path.extension().is_none_or(|kind| kind != "zip") {
        return Err(Refusal::about("cannotWrite", at));
    }

    let carried: Vec<(String, Vec<u8>)> = if logs {
        let session = held(&session);
        let live = witness::file(&session.paths);
        [live.clone(), live.with_extension("log.1")]
            .into_iter()
            .filter_map(|one| {
                let named = one.file_name()?.to_string_lossy().into_owned();
                Some((named, std::fs::read(&one).ok()?))
            })
            .collect()
    } else {
        Vec::new()
    };

    bundled(&path, &text, &carried)
        .map_err(|e| blamed(channel::WINDOW, "the report would not be written", e))
}

fn bundled(
    at: &std::path::Path,
    text: &str,
    carried: &[(String, Vec<u8>)],
) -> tisty_core::Result<()> {
    use std::io::Write;
    let file = std::fs::File::create(at)?;
    let _ = tisty_core::paths::ours_alone(at);
    let mut zip = zip::ZipWriter::new(file);
    let plain = zip::write::SimpleFileOptions::default();

    let mut put = |named: &str, body: &[u8]| -> tisty_core::Result<()> {
        zip.start_file(named, plain)
            .map_err(|e| tisty_core::Error::Io(std::io::Error::other(e)))?;
        zip.write_all(body)?;
        Ok(())
    };

    put("report.txt", text.as_bytes())?;
    for (named, body) in carried {
        put(named, body)?;
    }
    zip.finish()
        .map_err(|e| tisty_core::Error::Io(std::io::Error::other(e)))?;
    Ok(())
}

const REFUSALS: &[&str] = &[
    "untitled",
    "noSuchList",
    "ambiguousList",
    "badTag",
    "notATaskId",
    "notAListId",
    "notAStepId",
    "notAnEntry",
    "notADate",
    "notAPriority",
    "notACadence",
    "emptyStep",
    "emptyEntry",
    "pastDeadline",
    "pastReminder",
    "cannotRead",
    "cannotOpen",
    "cannotWrite",
    "attachmentTooBig",
    "noRemote",
    "noMeetingPlace",
    "syncUnreadable",
    "syncRefused",
    "syncBroke",
    "wouldMerge",
    "remoteInsideStore",
    "sharedIsTheBackup",
    "otherStore",
    "restoreFailed",
    "stillCarrying",
    "sandboxCannotMerge",
    "noSuchDoc",
    "noSuchIcon",
    "noSuchFolder",
    "manyLists",
    "internal",
    "internalNamed",
];

fn refusal_code(said: &str) -> Option<&'static str> {
    REFUSALS.iter().copied().find(|one| *one == said)
}

#[tauri::command]
fn note_trouble(code: String) {
    let Some(code) = refusal_code(&code) else {
        return;
    };
    witness::warn(
        channel::WINDOW,
        "the window showed a refusal",
        &[("code", Fact::Code(code))],
    );
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Logs {
    at: String,
    bytes: u64,
    lines: Vec<String>,
}

#[tauri::command(async)]
fn logs(session: tauri::State<'_, Mutex<Session>>, most: usize) -> Answer<Logs> {
    let session = held(&session);
    Ok(Logs {
        at: witness::file(&session.paths).display().to_string(),
        bytes: witness::weighs(&session.paths),
        lines: if most == 0 {
            Vec::new()
        } else {
            witness::recent(&session.paths, most)
        },
    })
}

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
    sandbox: Option<String>,
    repository: &'static str,
    license: &'static str,
    store: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    quiet: Vec<String>,
    attach_up_to: u64,
}

#[tauri::command]
async fn update_ready(session: tauri::State<'_, Mutex<Session>>) -> Answer<Option<update::Ready>> {
    let last = held(&session).config.checked_at;
    let now = jiff::Timestamp::now();
    if !update::due(last, now) {
        return Ok(None);
    }

    let manifest = tauri::async_runtime::spawn_blocking(update::fetch)
        .await
        .map_err(|_| Refusal::of("internal"))?;

    let mut session = held(&session);
    session.keep(|c| c.checked_at = Some(now))?;
    Ok(manifest.and_then(|said| update::newer(env!("CARGO_PKG_VERSION"), &said, update::route())))
}

#[tauri::command]
fn settings(session: tauri::State<'_, Mutex<Session>>) -> Answer<Settings> {
    let session = held(&session);
    Ok(Settings {
        quiet: session.config.muted().to_vec(),
        attach_up_to: session.config.copies_up_to(),
    })
}

#[tauri::command]
fn keep_settings(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    settings: Settings,
) -> Answer<Settings> {
    let mut session = held(&session);
    let quiet = settings.quiet.clone();
    let up_to = settings.attach_up_to.clamp(
        tisty_core::attach::COPIED_LEAST,
        tisty_core::attach::COPIED_MOST,
    );
    session.keep(|config| {
        config.quiet = (!quiet.is_empty()).then_some(quiet);
        config.attach_up_to = Some(up_to);
    })?;
    let now = Settings {
        quiet: session.config.muted().to_vec(),
        attach_up_to: session.config.copies_up_to(),
    };
    drop(session);
    herald::respeak(&app, &now.quiet);
    Ok(now)
}

#[tauri::command]
fn icons() -> Vec<(&'static str, &'static str)> {
    tisty_core::model::icon::ICONS.to_vec()
}

#[tauri::command]
fn list_add(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    icon: Option<String>,
) -> Answer<List> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }
    let mut session = held(&session);
    if !session.state.list_called(&name).is_empty() {
        return Err(Refusal::about("manyLists", name));
    }

    let id = ulid::Ulid::generate();
    let order = session.state.next_list_order();
    session.commit(Op::ListAdd {
        id,
        d: tisty_core::event::ListAdd {
            name,
            order,
            color: None,
        },
    })?;
    if let Some(icon) = icon.filter(|key| tisty_core::model::icon::known(key)) {
        session.commit(Op::ListLook {
            id,
            d: tisty_core::event::Look {
                icon: Some(Some(icon)),
                color: None,
            },
        })?;
    }
    session
        .state
        .lists
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notAListId"))
}

#[tauri::command]
fn list_look(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    icon: Option<String>,
) -> Answer<List> {
    let id: tisty_core::ListId = id.parse().map_err(|_| Refusal::of("notAListId"))?;
    let kept = match icon {
        Some(key) => Some(
            tisty_core::model::icon::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| Refusal::about("noSuchIcon", key))?,
        ),
        None => None,
    };

    let mut session = held(&session);
    session.commit(Op::ListLook {
        id,
        d: tisty_core::event::Look {
            icon: Some(kept),
            color: None,
        },
    })?;
    session
        .state
        .lists
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notAListId"))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Papers {
    folders: Vec<Folded>,
    docs: Vec<Filed>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Folded {
    id: String,
    name: String,
    parent: Option<String>,
    icon: Option<String>,
    holds: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Filed {
    id: String,
    file: String,
    title: String,
    folder: Option<String>,
    archived: bool,
}

#[tauri::command(async)]
fn docs(session: tauri::State<'_, Mutex<Session>>) -> Answer<Papers> {
    let session = held(&session);
    let root = session.paths.docs();
    let on_disk = tisty_core::docs::all(&root);

    let mut docs: Vec<Filed> = Vec::new();
    let named: std::collections::BTreeMap<&str, &tisty_core::docs::Doc> =
        on_disk.iter().map(|one| (one.id.as_str(), one)).collect();

    let mut kept_in_order: Vec<&tisty_core::model::Kept> = session.state.docs.values().collect();
    kept_in_order.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));

    for kept in kept_in_order {
        let Some(found) = named.get(kept.file.as_str()) else {
            continue;
        };
        docs.push(Filed {
            id: kept.id.to_string(),
            file: kept.file.clone(),
            title: found.title.clone(),
            folder: kept.folder.map(|at| at.to_string()),
            archived: kept.archived,
        });
    }
    Ok(Papers {
        folders: hanging(&session.state, None),
        docs,
    })
}

fn hanging(state: &State, parent: Option<tisty_core::model::FolderId>) -> Vec<Folded> {
    state
        .under(parent)
        .into_iter()
        .flat_map(|one| {
            let mut branch = vec![Folded {
                id: one.id.to_string(),
                name: one.name.clone(),
                parent: one.parent.map(|at| at.to_string()),
                icon: one.icon.clone(),
                holds: state.held_by(one.id),
            }];
            branch.append(&mut hanging(state, Some(one.id)));
            branch
        })
        .collect()
}

#[tauri::command]
fn folder_add(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    parent: Option<String>,
    icon: Option<String>,
) -> Answer<()> {
    let name = tisty_core::text::plainly(&name);
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }
    let parent = parent
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;

    let mut session = held(&session);
    if let Some(at) = parent {
        if !session.state.folders.contains_key(&at) {
            return Err(Refusal::of("noSuchFolder"));
        }
        if session.state.depth(Some(at)) >= tisty_core::model::DEEPEST {
            return Err(Refusal::of("tooDeep"));
        }
    }
    let order = tisty_core::order::last_of(
        session
            .state
            .under(parent)
            .iter()
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::FolderAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::FolderAdd {
            name,
            order,
            parent,
            icon: icon.filter(|key| tisty_core::model::icon::known(key)),
        },
    })?;
    Ok(())
}

#[tauri::command]
fn folder_rename(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    name: String,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let name = tisty_core::text::plainly(&name);
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }
    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    session.commit(Op::FolderRename {
        id,
        d: tisty_core::event::Name { name },
    })?;
    Ok(())
}

#[tauri::command]
fn folder_look(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    icon: Option<String>,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let kept = match icon {
        Some(key) => Some(
            tisty_core::model::icon::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| Refusal::about("noSuchIcon", key))?,
        ),
        None => None,
    };
    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    session.commit(Op::FolderLook {
        id,
        d: tisty_core::event::Look {
            icon: Some(kept),
            color: None,
        },
    })?;
    Ok(())
}

#[tauri::command]
fn folder_drop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    session.commit(Op::FolderDelete { id })?;
    Ok(())
}

#[tauri::command]
fn folder_file(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    parent: Option<String>,
) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchFolder"))?;
    let parent = parent
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;

    let mut session = held(&session);
    if !session.state.folders.contains_key(&id) {
        return Err(Refusal::of("noSuchFolder"));
    }
    if let Some(at) = parent {
        if !session.state.folders.contains_key(&at) {
            return Err(Refusal::of("noSuchFolder"));
        }
        if session.state.would_swallow(id, at) {
            return Err(Refusal::of("intoItself"));
        }
        if session.state.depth(Some(at)) + session.state.tall_under(id) > tisty_core::model::DEEPEST
        {
            return Err(Refusal::of("tooDeep"));
        }
    }
    session.commit(Op::FolderMove {
        id,
        d: tisty_core::event::Filed {
            folder: Some(parent),
        },
    })?;
    Ok(())
}

#[tauri::command]
fn doc_file(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    folder: Option<String>,
) -> Answer<()> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let mut session = held(&session);

    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    if !session.state.docs.contains_key(&id) {
        return Err(Refusal::of("noSuchDoc"));
    }
    if let Some(at) = folder
        && !session.state.folders.contains_key(&at)
    {
        return Err(Refusal::of("noSuchFolder"));
    }

    session.commit(Op::DocMove {
        id,
        d: tisty_core::event::Filed {
            folder: Some(folder),
        },
    })?;
    Ok(())
}

#[tauri::command(async)]
fn doc_read(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let root = held(&session).paths.docs();
    tisty_core::docs::read(&root, &id).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { bytes, limit } => {
            witness::warn(
                channel::WINDOW,
                "a document too big to hold was not opened",
                &[
                    ("id", witness::Fact::Id(id)),
                    ("bytes", witness::Fact::Bytes(bytes)),
                ],
            );
            Refusal::about("documentTooBig", weighed(limit))
        }
        _ => Refusal::about("noSuchDoc", id),
    })
}

#[tauri::command(async)]
fn doc_write(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
) -> Answer<tisty_core::docs::Doc> {
    let root = held(&session).paths.docs();
    tisty_core::docs::write(&root, &id, &body)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be written", e))?;
    Ok(tisty_core::docs::Doc {
        title: tisty_core::docs::titled(&body),
        id,
    })
}

#[tauri::command]
fn doc_away(session: tauri::State<'_, Mutex<Session>>, id: String, away: bool) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let mut session = held(&session);
    if !session.state.docs.contains_key(&id) {
        return Err(Refusal::of("noSuchDoc"));
    }
    session.commit(if away {
        Op::DocArchive { id }
    } else {
        Op::DocUnarchive { id }
    })?;
    Ok(())
}

#[tauri::command(async)]
fn doc_copy(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::docs::Doc> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let mut session = held(&session);
    let kept = session
        .state
        .docs
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("noSuchDoc"))?;

    let root = session.paths.docs();
    let body = tisty_core::docs::read(&root, &kept.file).map_err(|_| Refusal::of("noSuchDoc"))?;
    let body = match body.split_once('\n') {
        Some((first, rest)) if !first.trim().is_empty() => {
            format!("{first}{}\n{rest}", worded(&session.locale, "copy"))
        }
        _ if !body.trim().is_empty() => format!("{body}{}", worded(&session.locale, "copy")),
        _ => body,
    };
    let made = tisty_core::docs::create(&root, &session.config.device_id, &body)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be copied", e))?;

    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.folder == kept.folder)
            .map(|one| one.order.as_str()),
    );
    let twin = ulid::Ulid::generate();
    session.commit(Op::DocAdd {
        id: twin,
        d: tisty_core::event::DocAdd {
            file: made.id.clone(),
            order,
            folder: kept.folder,
        },
    })?;
    if kept.archived {
        session.commit(Op::DocArchive { id: twin })?;
    }
    Ok(made)
}

#[tauri::command(async)]
fn doc_import(
    session: tauri::State<'_, Mutex<Session>>,
    from: String,
    folder: Option<String>,
) -> Answer<tisty_core::docs::Doc> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let body =
        tisty_core::docs::read_outside(std::path::Path::new(&from)).map_err(|e| match e {
            tisty_core::Error::DocumentTooBig { limit, .. } => {
                Refusal::about("documentTooBig", weighed(limit))
            }
            _ => Refusal::about("cannotRead", from),
        })?;

    let mut session = held(&session);
    if let Some(at) = folder
        && !session.state.folders.contains_key(&at)
    {
        return Err(Refusal::of("noSuchFolder"));
    }
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, &body)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be imported", e))?;

    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.folder == folder)
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            file: made.id.clone(),
            order,
            folder,
        },
    })?;
    Ok(made)
}

#[tauri::command]
fn doc_new(
    session: tauri::State<'_, Mutex<Session>>,
    folder: Option<String>,
) -> Answer<tisty_core::docs::Doc> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let mut session = held(&session);
    if let Some(at) = folder
        && !session.state.folders.contains_key(&at)
    {
        return Err(Refusal::of("noSuchFolder"));
    }
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "")
        .map_err(|e| blamed(channel::WINDOW, "a document could not be made", e))?;

    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.folder == folder)
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            file: made.id.clone(),
            order,
            folder,
        },
    })?;
    Ok(made)
}

#[tauri::command]
fn doc_drop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let mut session = held(&session);

    let file = match id.parse() {
        Ok(id) => {
            let file = session
                .state
                .docs
                .get(&id)
                .map(|one| one.file.clone())
                .ok_or_else(|| Refusal::of("noSuchDoc"))?;
            session.commit(Op::DocDelete { id })?;
            file
        }
        Err(_) => id,
    };

    let root = session.paths.docs();
    tisty_core::docs::remove(&root, &file)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be removed", e))?;
    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn proofread(window: &tauri::WebviewWindow) {
    let done = window.with_webview(|webview| {
        use objc2::runtime::AnyObject;
        use objc2::{msg_send, sel};

        let wk = webview.inner().cast::<AnyObject>();
        if wk.is_null() {
            return;
        }
        // Asked before told: these are private selectors, and a missing one
        // raises an Objective-C exception that Rust cannot catch.
        unsafe {
            let spelling: bool =
                msg_send![wk, respondsToSelector: sel!(setContinuousSpellCheckingEnabled:)];
            if spelling {
                let _: () = msg_send![wk, setContinuousSpellCheckingEnabled: true];
            }
            let grammar: bool = msg_send![wk, respondsToSelector: sel!(setGrammarCheckingEnabled:)];
            if grammar {
                let _: () = msg_send![wk, setGrammarCheckingEnabled: true];
            }
        }
    });
    if let Err(e) = done {
        witness::warn(
            channel::WINDOW,
            "spell checking stayed off",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
}

#[cfg(not(target_os = "macos"))]
fn proofread(_window: &tauri::WebviewWindow) {}

#[cfg(target_os = "macos")]
fn menued(
    app: &tauri::AppHandle,
    locale: &Option<String>,
) -> tauri::Result<tauri::menu::Menu<tauri::Wry>> {
    use tauri::menu::{AboutMetadata, MenuItem, PredefinedMenuItem, Submenu};

    let leave = MenuItem::with_id(app, "leave", worded(locale, "quit"), true, Some("Cmd+Q"))?;
    let app_menu = Submenu::with_items(
        app,
        "Tisty",
        true,
        &[
            &PredefinedMenuItem::about(app, None, Some(AboutMetadata::default()))?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::hide(app, None)?,
            &PredefinedMenuItem::hide_others(app, None)?,
            &PredefinedMenuItem::show_all(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &leave,
        ],
    )?;
    let edit = Submenu::with_items(
        app,
        worded(locale, "edit"),
        true,
        &[
            &PredefinedMenuItem::undo(app, None)?,
            &PredefinedMenuItem::redo(app, None)?,
            &PredefinedMenuItem::separator(app)?,
            &PredefinedMenuItem::cut(app, None)?,
            &PredefinedMenuItem::copy(app, None)?,
            &PredefinedMenuItem::paste(app, None)?,
            &PredefinedMenuItem::select_all(app, None)?,
        ],
    )?;
    let window = Submenu::with_items(
        app,
        worded(locale, "windowMenu"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, None)?,
            &PredefinedMenuItem::fullscreen(app, None)?,
            &PredefinedMenuItem::close_window(app, None)?,
        ],
    )?;
    tauri::menu::Menu::with_items(app, &[&app_menu, &edit, &window])
}

pub fn parting<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::{Emitter, Manager};

    if app
        .state::<Leaving>()
        .0
        .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
        return;
    }
    let _ = app.emit("parting", ());

    let handle = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(1500));
        handle.exit(0);
    });
}

#[derive(Default)]
struct Leaving(std::sync::atomic::AtomicBool);

#[tauri::command]
fn printed(window: tauri::WebviewWindow) -> Answer<()> {
    window.print().map_err(|e| {
        witness::warn(
            channel::WINDOW,
            "the document would not print",
            &[("why", Fact::Why(e.to_string()))],
        );
        Refusal::of("cannotOpen")
    })
}

#[tauri::command]
fn parted(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn about(session: tauri::State<'_, Mutex<Session>>) -> Answer<About> {
    let session = held(&session);
    Ok(About {
        version: env!("CARGO_PKG_VERSION").to_string(),
        sandbox: tisty_core::paths::profile(),
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
        let carried = tauri::async_runtime::spawn_blocking(move || {
            tisty_sync::carry(&data, &device, &dest, tisty_sync::Way::Both)
        })
        .await;
        match carried {
            Ok(Err(why)) => witness::warn(
                channel::SYNC,
                "the carry on opening did not finish",
                &[("code", Fact::Code(said(why).code))],
            ),
            Err(_) => witness::warn(channel::SYNC, "the carry on opening never ran", &[]),
            Ok(Ok(_)) => {}
        }
        brought = tisty_core::cache::fingerprint(&store) != before;
    }

    let mut session = held(&session);
    if brought {
        session.reproject().map_err(|e| {
            blamed(
                channel::SYNC,
                "the store would not project after carrying",
                e,
            )
        })?;
    }
    let audit =
        tisty_core::cache::audit(&session.paths.store(), session.paths.cache()).map_err(|e| {
            witness::error(
                channel::CACHE,
                "the cache could not be audited on settling in",
                &[("why", Fact::Why(e.to_string()))],
            );
            Refusal::of("internal")
        })?;
    let agrees = matches!(audit, tisty_core::cache::Audit::Agrees { .. });
    if !agrees {
        let _ = std::fs::remove_dir_all(session.paths.cache());
        session.reproject().map_err(|e| {
            blamed(
                channel::CACHE,
                "the store would not project without a cache",
                e,
            )
        })?;
    }

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

    let mut held: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    held.extend(tisty_core::docs::referenced(&session.paths.docs()));

    Ok(Carrying {
        chosen: match &config.sync {
            Some(tisty_core::config::Sync::Folder(at)) => Some(at.display().to_string()),
            _ => None,
        },
        asked: config.sync.is_some(),
        backs_up: config.backs_up(),
        last: config.synced_at.map(|at| at.to_string()),
        loose: tisty_core::attach::loose(session.paths.data(), &held).files(),
        open: session.state.matching(&Filter::default(), today()).len(),
        archived: session
            .state
            .matching(
                &Filter {
                    scope: Scope::Archived,
                    ..Default::default()
                },
                today(),
            )
            .len(),
        lists: session.state.lists.len(),
        attachments: report::attachments(session.paths.data()).files,
        weight: report::weighed(session.paths.data()),
        backed_up_at: config.backed_up_at.map(|at| at.to_string()),
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
        tisty_core::config::Closing::Quit => parting(window.app_handle()),
    }
    Ok(())
}

#[tauri::command]
async fn sync_now(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    way: Option<String>,
) -> Answer<&'static str> {
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

    tauri::async_runtime::spawn_blocking(move || tisty_sync::carry(&data, &device, &dest, way))
        .await
        .map_err(|_| Refusal::of("internal"))?
        .map_err(said)?;

    let mut session = held(&session);
    let moved = tisty_core::cache::fingerprint(&store) != before;
    if moved {
        session.reproject().map_err(|e| {
            blamed(
                channel::SYNC,
                "the store would not project after syncing",
                e,
            )
        })?;
    }
    if let Err(e) = session.take_a_seat() {
        witness::warn(
            channel::SYNC,
            "this machine could not put itself on the list",
            &[("why", Fact::Why(e.to_string()))],
        );
    }
    witness::note(
        channel::SYNC,
        "a carry finished",
        &[("moved", Fact::Word(if moved { "yes" } else { "no" }))],
    );
    session.keep(|c| c.synced_at = Some(jiff::Timestamp::now()))?;
    Ok(if moved { "came" } else { "same" })
}

#[tauri::command]
async fn back_up(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    let (data, aside) = {
        let session = held(&session);
        if !session.config.backs_up() {
            return Err(Refusal::of("sharedIsTheBackup"));
        }
        (
            session.paths.data().to_path_buf(),
            session.paths.cache().to_path_buf(),
        )
    };

    let at = std::path::PathBuf::from(&into);
    let made =
        tauri::async_runtime::spawn_blocking(move || tisty_core::backup::write(&data, &at, &aside))
            .await
            .map_err(|_| Refusal::of("internal"))?
            .map_err(|e| {
                witness::error(
                    channel::BACKUP,
                    "the backup could not be written",
                    &[("why", Fact::Why(e.to_string()))],
                );
                Refusal::about("cannotWrite", into)
            })?;

    let now = jiff::Timestamp::now();
    held(&session).keep(|config| config.backed_up_at = Some(now))?;
    Ok(made.bytes)
}

#[tauri::command]
fn remove_machine(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let mut session = held(&session);
    let who = tisty_core::event::DeviceId(id.clone());
    if who == session.config.device_id {
        return Err(Refusal::of("notThisMachine"));
    }

    session
        .commit(Op::DeviceRemove { d: who })
        .map_err(|e| blamed(channel::STORE, "the machine could not be removed", e))?;

    if let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone()
        && let Some(theirs) = named_in(&dest.join("store"), &id)
        && let Err(e) = std::fs::remove_dir_all(&theirs)
    {
        witness::warn(
            channel::SYNC,
            "what the removed machine left in the shared folder is still there",
            &[
                ("at", Fact::Path(theirs)),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
    Ok(())
}

fn named_in(store: &std::path::Path, id: &str) -> Option<std::path::PathBuf> {
    std::fs::read_dir(store)
        .ok()?
        .filter_map(|one| one.ok())
        .find(|one| one.file_name().to_str() == Some(id) && one.path().is_dir())
        .map(|one| one.path())
}

#[tauri::command]
async fn join_them(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (paths, aside) = {
        let session = held(&session);
        (session.paths.clone(), session.paths.cache().to_path_buf())
    };

    let at = std::path::PathBuf::from(&into);
    let made = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::backup::reset(&paths, &at, &aside)
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::error(
            channel::BACKUP,
            "nothing was reset because the backup did not land",
            &[("why", Fact::Why(e.to_string()))],
        );
        Refusal::about("cannotWrite", into)
    })?;

    *held(&session) = Session::open().map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after being reset",
            e,
        )
    })?;
    Ok(made.bytes)
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

    *held(&session) = Session::open().map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after a restore",
            e,
        )
    })?;
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
        tisty_sync::Trouble::Refused(why) => Refusal::about("syncRefused", why),
        tisty_sync::Trouble::Broke(why) => Refusal::about("syncBroke", why),
        tisty_sync::Trouble::WouldReset { theirs } => Refusal::about("wouldReset", theirs),
        tisty_sync::Trouble::NotAllowed(who) => Refusal::about("notAllowed", who),
    }
}

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

#[tauri::command]
fn roomy() -> u64 {
    tisty_core::docs::BODY_ROOMY
}

#[tauri::command]
fn weighs(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<u64> {
    let root = held(&session).paths.data().to_path_buf();
    let at = tisty_core::attach::resolve(&reference, &root)
        .map_err(|_| Refusal::about("cannotRead", reference.clone()))?;
    let told =
        std::fs::metadata(&at).map_err(|_| Refusal::about("cannotRead", reference.clone()))?;
    if !told.is_file() {
        return Err(Refusal::about("cannotRead", reference));
    }
    Ok(told.len())
}

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
    if !safe_to_open(&at) {
        return show(&at, &reference);
    }
    handed(&at).map_err(|_| Refusal::about("cannotOpen", reference))?;
    let _ = app;
    Ok(())
}

#[tauri::command]
fn revealed(session: tauri::State<'_, Mutex<Session>>, path: String) -> Answer<()> {
    let at = std::path::Path::new(&path);

    let plain = at.components().next().is_none_or(|first| {
        !matches!(first, std::path::Component::Prefix(at) if !matches!(at.kind(), std::path::Prefix::Disk(_)))
    });
    if !plain || !at.is_absolute() {
        return Err(Refusal::about("cannotOpen", path));
    }

    let (data, config) = {
        let session = held(&session);
        (
            session.paths.data().to_path_buf(),
            session.paths.config().to_path_buf(),
        )
    };
    let real = at
        .canonicalize()
        .map_err(|_| Refusal::about("cannotOpen", path.clone()))?;
    let ours = [data, config]
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| real.starts_with(&one));
    if !ours {
        return Err(Refusal::about("cannotOpen", path));
    }
    show(at, &path)
}

fn handed(at: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    tauri_plugin_opener::open_path(at, None::<&str>)?;
    Ok(())
}

fn show(at: &std::path::Path, said: &str) -> Answer<()> {
    tauri_plugin_opener::reveal_item_in_dir(at)
        .map_err(|_| Refusal::about("cannotOpen", said.to_string()))
}

fn safe_to_open(at: &std::path::Path) -> bool {
    let name = at
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
        .trim_end_matches(['.', ' '])
        .to_lowercase();
    let ext = name.rsplit_once('.').map(|(_, e)| e).unwrap_or_default();
    matches!(
        ext,
        "pdf"
            | "txt"
            | "md"
            | "markdown"
            | "rtf"
            | "csv"
            | "tsv"
            | "json"
            | "xml"
            | "yaml"
            | "yml"
            | "toml"
            | "log"
            | "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "avif"
            | "bmp"
            | "tiff"
            | "tif"
            | "heic"
            | "svg"
            | "ico"
            | "mp3"
            | "wav"
            | "flac"
            | "aac"
            | "ogg"
            | "opus"
            | "m4a"
            | "mp4"
            | "m4v"
            | "mov"
            | "webm"
            | "mkv"
            | "avi"
            | "doc"
            | "docx"
            | "xls"
            | "xlsx"
            | "ppt"
            | "pptx"
            | "odt"
            | "ods"
            | "odp"
            | "pages"
            | "numbers"
            | "key"
            | "epub"
            | "zip"
            | "gz"
            | "tar"
            | "bz2"
            | "xz"
            | "7z"
    )
}

#[tauri::command]
fn attach(
    session: tauri::State<'_, Mutex<Session>>,
    path: String,
    label: Option<String>,
    roomy: Option<bool>,
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

    let (root, ceiling) = {
        let session = held(&session);
        let ceiling = if roomy.unwrap_or(false) {
            tisty_core::attach::COPIED_IN_DOC
        } else {
            session.config.copies_up_to()
        };
        (session.paths.data().to_path_buf(), ceiling)
    };
    let kept = tisty_core::attach::keep(&source, &root, ceiling).map_err(|e| {
        witness::warn(channel::ATTACH, "the file could not be kept", &e.told());
        match e {
            tisty_core::Error::AttachmentTooBig { limit, .. } => Refusal::about(
                if roomy.unwrap_or(false) {
                    "attachmentTooBigHere"
                } else {
                    "attachmentTooBig"
                },
                weighed(limit),
            ),
            _ => Refusal::about("cannotRead", name.clone()),
        }
    })?;

    Ok(kept.written(&name))
}

fn weighed(bytes: u64) -> String {
    let units = ["B", "kB", "MB", "GB"];
    let mut step = 0;
    let mut left = bytes as f64;
    while left >= 1000.0 && step < units.len() - 1 {
        left /= 1000.0;
        step += 1;
    }
    if step == 0 {
        format!("{left:.0} {}", units[step])
    } else {
        format!("{left:.1} {}", units[step])
    }
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

struct Bound(Option<String>);

fn listen_for<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Option<String> {
    use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

    let tries = [
        (
            "Ctrl+Shift+Space",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Space),
        ),
        #[cfg(not(target_os = "macos"))]
        (
            "Ctrl+Alt+Space",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space),
        ),
        (
            "Ctrl+Shift+T",
            Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyT),
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
    witness::warn(channel::WINDOW, "no shortcut was free", &[]);
    None
}

fn worded(locale: &Option<String>, key: &str) -> String {
    let spanish = locale
        .as_deref()
        .or(sys_locale::get_locale().as_deref())
        .is_some_and(|code| code.to_lowercase().starts_with("es"));

    match (key, spanish) {
        ("quit", true) => "Salir de Tisty".into(),
        ("quit", false) => "Quit Tisty".into(),
        ("edit", true) => "Edición".into(),
        ("edit", false) => "Edit".into(),
        ("windowMenu", true) => "Ventana".into(),
        ("windowMenu", false) => "Window".into(),
        ("copy", true) => " (copia)".into(),
        ("copy", false) => " (copy)".into(),
        ("show", true) => "Abrir Tisty".into(),
        ("show", false) => "Open Tisty".into(),
        ("capture", true) => "Capturar…".into(),
        ("capture", false) => "Capture…".into(),
        ("due", true) => "Recordatorio".into(),
        ("due", false) => "Reminder".into(),
        ("missed", true) => "{n} recordatorios mientras no estabas".into(),
        ("missed", false) => "{n} reminders while you were away".into(),
        (_, true) => "Salir de Tisty".into(),
        (_, false) => "Quit Tisty".into(),
    }
}

#[derive(Default)]
struct OneAtATime(std::sync::atomic::AtomicBool);

impl OneAtATime {
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
    let mut building = tauri::Builder::default();

    if tisty_core::paths::profile().is_none() {
        building = building.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            tray::surface(app);
        }));
    }

    building
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .setup(move |app| {
            let session = match Session::open() {
                Ok(session) => session,
                Err(why) => {
                    use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
                    witness::error(channel::WINDOW, "the session would not open", &why.told());
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

            for at in tisty_core::backup::leftovers(session.paths.data()) {
                if let Err(why) = std::fs::remove_dir_all(&at) {
                    witness::warn(
                        channel::BACKUP,
                        "what a restore left behind could not be swept up",
                        &[("at", Fact::Path(at)), ("why", Fact::Why(why.to_string()))],
                    );
                }
            }

            let attachments = session.paths.attachments();
            if let Err(why) = std::fs::create_dir_all(&attachments) {
                witness::error(
                    channel::ATTACH,
                    "the attachments folder could not be made",
                    &[
                        ("at", Fact::Path(attachments.clone())),
                        ("why", Fact::Why(why.to_string())),
                    ],
                );
            }
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
                missed: worded(&session.locale, "missed"),
            };
            let watched = session.paths.clone();
            let quiet = session.config.muted().to_vec();
            app.manage(Mutex::new(session));
            app.manage(herald::Speaking::new(app.handle(), telling, &quiet));
            herald::watch(app.handle().clone(), watched);

            let perched = tray::raise(app.handle(), &words).is_some();
            app.manage(Perched(perched));
            app.manage(Bound(listen_for(app.handle())));

            {
                let held = app.state::<Mutex<Session>>();
                let held = crate::held(&held);
                let seen = app.asset_protocol_scope();
                for at in [held.paths.attachments(), held.paths.docs()] {
                    if let Err(e) = seen.allow_directory(&at, true) {
                        witness::warn(
                            channel::WINDOW,
                            "attachments will not show",
                            &[("at", Fact::Path(at)), ("why", Fact::Why(e.to_string()))],
                        );
                    }
                }
            }

            #[cfg(target_os = "macos")]
            {
                let locale = held(&app.state::<Mutex<Session>>()).locale.clone();
                match menued(app.handle(), &locale) {
                    Ok(menu) => {
                        let _ = app.set_menu(menu);
                        app.on_menu_event(|app, event| {
                            if event.id() == "leave" {
                                parting(app);
                            }
                        });
                    }
                    Err(e) => witness::warn(
                        channel::WINDOW,
                        "the menu would not build",
                        &[("why", Fact::Why(e.to_string()))],
                    ),
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                proofread(&window);
                let _ = window.show();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let tauri::WindowEvent::ThemeChanged(_) = event {
                tray::repaint(window.app_handle());
                return;
            }

            let tauri::WindowEvent::CloseRequested { api, .. } = event else {
                return;
            };

            let app = window.app_handle();
            if !app.state::<Perched>().0 {
                api.prevent_close();
                parting(app);
                return;
            }

            let asked = held(&app.state::<Mutex<Session>>()).config.on_close;
            match asked {
                Some(tisty_core::config::Closing::Quit) => {
                    api.prevent_close();
                    parting(app);
                }
                Some(tisty_core::config::Closing::Hide) => {
                    api.prevent_close();
                    let _ = window.hide();
                }
                None => {
                    api.prevent_close();
                    let _ = window.emit("closing", ());
                }
            }
        })
        .manage(OneAtATime::default())
        .manage(Leaving::default())
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
            attach,
            served,
            weighs,
            roomy,
            opened,
            revealed,
            sync_state,
            choose_sync,
            sync_now,
            back_up,
            restore,
            join_them,
            remove_machine,
            checked,
            rebuild,
            about,
            settings,
            keep_settings,
            facts,
            keep_report,
            note_trouble,
            note_break,
            update_ready,
            logs,
            icons,
            list_add,
            list_look,
            docs,
            folder_add,
            folder_rename,
            folder_look,
            folder_drop,
            doc_file,
            doc_read,
            doc_write,
            doc_new,
            doc_drop,
            doc_import,
            doc_copy,
            doc_away,
            parted,
            printed,
            folder_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_that_only_windows_reads_as_a_program_is_never_opened() {
        for name in [
            ".exe",
            ".bat",
            ".cmd",
            "pay.exe.",
            "pay.exe ",
            "pay.exe...",
            "x.exe",
            "x.msi",
            "x.settingcontent-ms",
            "x.appref-ms",
            "x.jnlp",
            "x.py",
            "x.inf",
            "x.scpt",
            "x.mobileconfig",
            "x.inetloc",
            "x.command",
            "x.desktop",
            "x.EXE",
            "x.Bat",
        ] {
            assert!(
                !safe_to_open(std::path::Path::new(name)),
                "{name} would be opened"
            );
        }
    }

    #[test]
    fn the_files_a_person_actually_attaches_still_open() {
        for name in [
            "informe.pdf",
            "foto.png",
            "hoja.xlsx",
            "notas.md",
            "musica.mp3",
            "video.mp4",
            "datos.csv",
            "archivo.zip",
            "diagrama.svg",
            "carta.docx",
            "FOTO.JPEG",
        ] {
            assert!(
                safe_to_open(std::path::Path::new(name)),
                "{name} was refused"
            );
        }
    }

    #[test]
    fn only_paths_inside_the_store_can_be_shown() {
        let home = tempfile::tempdir().unwrap();
        let data = home.path().join("data");
        std::fs::create_dir_all(data.join("attachments")).unwrap();
        let mine = data.join("attachments/kept.pdf");
        std::fs::write(&mine, b"x").unwrap();

        let outside = home.path().join("id_rsa");
        std::fs::write(&outside, b"x").unwrap();

        let ours = |at: &std::path::Path| {
            let real = at.canonicalize().unwrap();
            real.starts_with(data.canonicalize().unwrap())
        };

        assert!(ours(&mine));
        assert!(!ours(&outside), "a path outside the store was shown");
    }

    #[test]
    fn nothing_else_in_the_project_is_allowed_to_be_unsafe() {
        fn rust(at: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
            let Ok(entries) = std::fs::read_dir(at) else {
                return;
            };
            for one in entries.filter_map(|e| e.ok()) {
                let path = one.path();
                if path.is_dir() {
                    if path.file_name().is_some_and(|n| n == "target") {
                        continue;
                    }
                    rust(&path, found);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    found.push(path);
                }
            }
        }

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("the repository");
        let mut files = Vec::new();
        rust(&root.join("crates"), &mut files);
        rust(&root.join("app/src-tauri/src"), &mut files);

        let allowed: Vec<String> = files
            .iter()
            .filter(|at| {
                std::fs::read_to_string(at)
                    .map(|body| body.contains("allow(unsafe_code)"))
                    .unwrap_or(false)
            })
            .map(|at| at.display().to_string())
            .collect();

        assert_eq!(
            allowed.len(),
            1,
            "unsafe is allowed in more than the one audited place: {allowed:?}"
        );
        let mine = std::path::Path::new(&allowed[0]);
        assert!(mine.ends_with("src-tauri/src/lib.rs"), "{allowed:?}");
    }

    fn now() -> jiff::Zoned {
        "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
    }

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
        draft.title = edits.retitled(text, &read, "es").expect("a new title");

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
            edits.retitled(text, &read, "es").as_deref(),
            Some("comprar pan #casa")
        );
    }

    #[test]
    fn only_a_code_we_ship_is_written_down() {
        assert_eq!(refusal_code("pastDeadline"), Some("pastDeadline"));
        assert_eq!(refusal_code("internalNamed"), Some("internalNamed"));
        assert_eq!(refusal_code("comprar pan"), None);
    }

    #[test]
    fn choosing_a_different_date_leaves_the_title_alone() {
        let text = "comprar pan mañana";
        let read = tisty_nl::parse(text, &now(), "es");
        let edits = Edits {
            date: Some("2026-08-20".to_string()),
            ..Default::default()
        };
        assert_eq!(edits.retitled(text, &read, "es"), None);
    }

    #[test]
    fn a_report_is_one_zip_that_carries_what_was_ticked() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("tisty-report.zip");
        let log = (
            "tisty.log".to_string(),
            b"WARN sync folder unreachable
"
            .to_vec(),
        );

        bundled(
            &at,
            "# report
version 0.1.0
",
            std::slice::from_ref(&log),
        )
        .unwrap();

        let mut zip = zip::ZipArchive::new(std::fs::File::open(&at).unwrap()).unwrap();
        let named: Vec<String> = zip.file_names().map(str::to_owned).collect();
        assert!(named.contains(&"report.txt".to_string()), "{named:?}");
        assert!(named.contains(&"tisty.log".to_string()), "{named:?}");

        use std::io::Read;
        let mut said = String::new();
        zip.by_name("report.txt")
            .unwrap()
            .read_to_string(&mut said)
            .unwrap();
        assert!(said.contains("version 0.1.0"), "{said}");
    }

    #[test]
    fn a_report_without_the_log_carries_only_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("tisty-report.zip");

        bundled(&at, "# report", &[]).unwrap();

        let zip = zip::ZipArchive::new(std::fs::File::open(&at).unwrap()).unwrap();
        assert_eq!(zip.file_names().count(), 1);
    }

    fn every_day(until: Option<jiff::civil::Date>) -> Change {
        Change {
            repeat: Some(tisty_core::model::Repeat {
                from: tisty_core::model::From::Due,
                each: tisty_core::model::Cadence {
                    every: 1,
                    unit: tisty_core::model::Unit::Day,
                },
                until,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn a_series_cannot_be_told_to_have_ended_already() {
        let past = repeated(&every_day(Some(jiff::civil::date(2026, 8, 4))), &now());

        assert!(
            matches!(past, Err(ref why) if why.code == "pastEnd"),
            "{past:?}"
        );
    }

    #[test]
    fn today_is_late_enough_to_end_on() {
        assert!(repeated(&every_day(Some(jiff::civil::date(2026, 8, 5))), &now()).is_ok());
        assert!(repeated(&every_day(Some(jiff::civil::date(2027, 1, 1))), &now()).is_ok());
        assert!(repeated(&every_day(None), &now()).is_ok());
    }
}
