use std::sync::Mutex;

mod command;
mod herald;
mod report;
mod tray;
mod update;
mod waking;
mod wiring;

use tauri::{Emitter, Manager};

use tisty_core::{
    Config, Event, List, Op, Paths, Reading, State, Store, Tag, Task,
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
    corpus: tisty_core::docs::Corpus,
    print: String,
    /// What each open document looked like when this window last read or wrote it.
    minded: std::collections::HashMap<String, String>,
    locale: Option<String>,
    log: Option<(String, Vec<Event>)>,
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
        let clean = !paths.config_file().exists();
        let config = Config::load_or_init(&paths)?;
        let store = Store::open(paths.store(), config.device_id.clone())?;
        let state = tisty_core::cache::project(&paths.store(), paths.cache())?;
        let cache = tisty_core::cache::Cache::open(paths.cache())?;
        let print = tisty_core::cache::fingerprint(&paths.store());

        let mut session = Self {
            locale: config.locale.clone(),
            paths,
            config,
            state,
            store,
            cache,
            corpus: tisty_core::docs::Corpus::default(),
            print,
            minded: std::collections::HashMap::new(),
            log: None,
        };
        session.take_out_the_shed();
        session.take_out_the_retired();
        let gone = tisty_core::attach::empty_the_bin(
            session.paths.data(),
            jiff::Timestamp::now().as_second(),
        );
        if gone > 0 {
            witness::note(
                channel::ATTACH,
                "what waited in the bin past its time is gone",
                &[("count", Fact::Count(gone))],
            );
        }
        session.sow_if_clean(clean);
        Ok(session)
    }

    fn sow_if_clean(&mut self, clean: bool) {
        if !clean || !self.state.lists.is_empty() || !self.state.tasks.is_empty() {
            return;
        }
        let code = tisty_core::model::spoken(self.config.locale.as_deref());
        if let Err(why) = self.commit_all(tisty_core::model::sown(&code)) {
            witness::warn(
                channel::CONFIG,
                "the lists a fresh install starts with were not written",
                &[("why", Fact::Why(why.to_string()))],
            );
        }
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

    fn log(&mut self) -> tisty_core::Result<&[Event]> {
        let held = self
            .log
            .as_ref()
            .is_some_and(|(print, _)| *print == self.print);
        if !held {
            let read = tisty_core::store::read_all(self.paths.store())?;
            self.log = Some((self.print.clone(), read));
        }
        Ok(&self.log.as_ref().expect("just filled").1)
    }

    fn reproject(&mut self) -> tisty_core::Result<()> {
        self.state = tisty_core::cache::project(&self.paths.store(), self.paths.cache())?;
        self.print = tisty_core::cache::fingerprint(&self.paths.store());
        Ok(())
    }

    fn alive(&self) -> Vec<String> {
        self.state
            .docs
            .values()
            .map(|one| one.file.clone())
            .collect()
    }

    fn mind(&mut self, id: &str) {
        let now = tisty_core::docs::resolve(&self.paths.docs(), id)
            .ok()
            .and_then(|at| tisty_core::docs::print_of(&at).ok().flatten());
        match now {
            Some(print) => self.minded.insert(id.to_string(), print),
            None => self.minded.remove(id),
        };
    }

    /// The body itself, not the file: reading the disk again would mind what nobody here saw.
    fn mind_body(&mut self, id: &str, body: &str) {
        self.minded
            .insert(id.to_string(), tisty_core::attach::printed(body.as_bytes()));
    }

    fn moved(&self, id: &str) -> bool {
        let now = tisty_core::docs::resolve(&self.paths.docs(), id)
            .ok()
            .and_then(|at| tisty_core::docs::print_of(&at).ok().flatten());
        stale(self.minded.get(id).map(String::as_str), now.as_deref())
    }

    /// Here and in the shared folder both, or a machine that holds none of them sees none astray.
    /// The shared folder, but only while this machine leaves anything in it.
    fn shared_now(&self) -> Option<std::path::PathBuf> {
        match (&self.config.sync, self.config.holds()) {
            (Some(tisty_core::config::Sync::Folder(dest)), holds)
                if holds != tisty_core::config::Holds::Everywhere =>
            {
                Some(dest.clone())
            }
            _ => None,
        }
    }

    fn adrift(&self, held: &[String]) -> tisty_core::attach::Loose {
        let mut found = tisty_core::attach::loose(self.paths.data(), held);
        let Some(dest) = self.shared_now() else {
            return found;
        };
        // The same file is in both places for anyone who syncs; counting it twice doubles the bill.
        let here: std::collections::BTreeSet<String> =
            found.items.iter().map(|one| one.at.clone()).collect();
        for one in tisty_core::attach::loose(&dest, held).items {
            if here.contains(&one.at) {
                continue;
            }
            found.bytes += one.bytes;
            found.items.push(tisty_core::attach::Astray {
                shared: true,
                ..one
            });
        }
        found
    }

    fn referenced(&self) -> Vec<String> {
        let mut held: Vec<String> = self
            .state
            .tasks
            .values()
            .flat_map(|task| task.references())
            .map(|one| one.target)
            .collect();
        held.extend(tisty_core::docs::referenced(&self.paths.docs()));
        held
    }

    fn take_out_the_shed(&mut self) {
        if self.state.shed.is_empty() {
            return;
        }
        let mut gone = tisty_core::docs::sweep(&self.paths.docs(), &self.state.shed);
        if let Some(tisty_core::config::Sync::Folder(dest)) = self.config.sync.clone() {
            gone += tisty_core::docs::sweep(&dest.join("docs"), &self.state.shed);
        }
        if gone > 0 {
            witness::note(
                channel::SYNC,
                "a document deleted elsewhere is gone from here too",
                &[("count", Fact::Count(gone))],
            );
        }
    }

    fn take_out_the_retired(&mut self) {
        if self.state.retired.is_empty() {
            return;
        }
        let named = self.referenced();
        let held: std::collections::BTreeSet<&str> = named.iter().map(|one| one.as_str()).collect();
        let mut gone = tisty_core::attach::sweep(self.paths.data(), &self.state.retired, &held);
        if let Some(tisty_core::config::Sync::Folder(dest)) = self.config.sync.clone() {
            gone += tisty_core::attach::sweep(&dest, &self.state.retired, &held);
        }
        if gone > 0 {
            witness::note(
                channel::ATTACH,
                "what was retired elsewhere is gone from here too",
                &[("count", Fact::Count(gone))],
            );
        }
    }

    fn take_a_seat(&mut self) -> tisty_core::Result<()> {
        let who = self.config.device_id.clone();
        if self.state.devices.contains(&who) {
            return Ok(());
        }
        self.commit(Op::DeviceJoin {
            d: who,
            k: Some(tisty_core::DeviceKind::Machine),
        })
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
    for (key, how) in [("stories", Reading::Story), ("traces", Reading::Trace)] {
        count(
            key,
            Filter {
                scope: Scope::Archived,
                reading: Some(how),
                ..Default::default()
            },
        );
    }
    counts.insert("routines".to_string(), tisty_core::series::how_many(state));

    counts.insert("tags".to_string(), state.tags().len());
    counts.insert(
        "quadrants".to_string(),
        state
            .matching(&Filter::default(), today())
            .iter()
            .filter(|task| !task.priority.set())
            .count(),
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
            Rejected::ArchivedList(name) => Refusal::about("archivedList", name),
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
    lists: Vec<String>,
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
    #[serde(default)]
    reading: Option<String>,
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
                .into_iter()
                .chain(self.lists)
                .map(|id| id.parse().map_err(|_| Refusal::of("notAListId")))
                .collect::<Result<_, _>>()?,
            tags: self
                .tags
                .iter()
                .map(|t| Tag::new(t).map_err(|_| Refusal::about("badTag", t)))
                .collect::<Result<_, _>>()?,
            tagged: self.tagged,
            hidden: self.hidden,
            priority: None,
            repeating: self.repeating,
            reading: match self.reading.as_deref() {
                Some("story") => Some(Reading::Story),
                Some("routine") => Some(Reading::Routine),
                Some("trace") => Some(Reading::Trace),
                _ => None,
            },
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

#[derive(serde::Serialize)]
struct Left {
    kind: &'static str,
    target: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    away: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    gone: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    bytes: Option<u64>,
}

#[tauri::command]
fn task_left(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<Left>> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.reload()?;
    let Some(task) = session.state.tasks.get(&id) else {
        return Err(Refusal::of("notATaskId"));
    };

    let root = session.paths.data().to_path_buf();
    let shared = session.shared_now();
    let on_disk = tisty_core::docs::all(&session.paths.docs());
    let named: std::collections::BTreeMap<&str, &tisty_core::docs::Doc> =
        on_disk.iter().map(|one| (one.id.as_str(), one)).collect();

    let left = task
        .references()
        .into_iter()
        .map(|one| match one.kind {
            // The window writes a document as `[title](tisty:doc/ID)`, which parses as a link.
            _ if one.target.starts_with("tisty:doc/") => {
                // A reference names the document by its file or by its id, depending on who wrote it.
                let held = one.target.strip_prefix("tisty:doc/").and_then(|raw| {
                    raw.parse()
                        .ok()
                        .and_then(|id| session.state.docs.get(&id))
                        .or_else(|| session.state.docs.values().find(|doc| doc.file == raw))
                });
                let on_paper = held.and_then(|doc| named.get(doc.file.as_str()));
                Left {
                    kind: "doc",
                    label: on_paper
                        .map(|doc| doc.title.clone())
                        .filter(|title| !title.is_empty())
                        .or_else(|| one.label.clone()),
                    away: held.is_some_and(|doc| doc.archived),
                    gone: held.is_none() || on_paper.is_none(),
                    target: one.target,
                    bytes: None,
                }
            }
            tisty_core::refs::Kind::Doc => Left {
                kind: "named",
                label: one.label.clone(),
                away: false,
                gone: false,
                target: one.target,
                bytes: None,
            },
            tisty_core::refs::Kind::Link
                if tisty_core::attach::names_an_attachment(&one.target) =>
            {
                let bytes = where_it_lies(&one.target, &root, shared.as_deref())
                    .and_then(|at| std::fs::metadata(at).ok())
                    .filter(|told| told.is_file())
                    .map(|told| told.len());
                Left {
                    kind: "file",
                    label: one.label.clone(),
                    away: false,
                    gone: bytes.is_none(),
                    target: one.target,
                    bytes,
                }
            }
            tisty_core::refs::Kind::Link => Left {
                kind: if one.target.starts_with("http") {
                    "link"
                } else {
                    "named"
                },
                label: one.label.clone(),
                away: false,
                gone: false,
                target: one.target,
                bytes: None,
            },
        })
        .collect();
    Ok(left)
}

#[tauri::command]
fn task_story(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::story::Story> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.reload()?;
    let told = tisty_core::story::story(session.log()?, id);
    Ok(told)
}

#[tauri::command]
fn task_series(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<Option<tisty_core::series::Series>> {
    let id: tisty_core::TaskId = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    session.reload()?;
    Ok(tisty_core::series::series(&session.state, id))
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Agent {
    on: bool,
    called: Option<String>,
    id: Option<String>,
    filed: usize,
}

#[tauri::command]
fn agent(session: tauri::State<'_, Mutex<Session>>) -> Answer<Agent> {
    let mut session = held(&session);
    session.reload()?;
    let who = session.config.agent_id.clone();
    Ok(Agent {
        on: who.is_some(),
        called: who
            .as_ref()
            .map(|one| tisty_core::config::nicknamed(&one.0)),
        filed: match &who {
            Some(one) => session
                .state
                .tasks
                .values()
                .filter(|task| task.created_by.as_ref() == Some(one))
                .count(),
            None => 0,
        },
        id: who.map(|one| one.0),
    })
}

/// Registering is the person's act. Nothing an assistant can say over the wire reaches here,
/// which is what stops one granting itself a voice by connecting.
#[tauri::command]
fn agent_turn(session: tauri::State<'_, Mutex<Session>>, on: bool) -> Answer<Agent> {
    {
        let mut session = held(&session);
        session.reload()?;
        let paths = session.paths.clone();
        if on {
            tisty_core::agent::register(&paths)
                .map_err(|e| blamed(channel::STORE, "the agent could not be registered", e))?;
        } else {
            tisty_core::agent::retire(&paths)
                .map_err(|e| blamed(channel::STORE, "the agent could not be retired", e))?;
        }
        session.config = Config::load(&session.paths.config_file())
            .ok()
            .flatten()
            .unwrap_or_else(|| session.config.clone());
        session.reproject()?;
    }
    agent(session)
}

#[tauri::command]
fn routines(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<tisty_core::series::Series>> {
    let mut session = held(&session);
    session.reload()?;
    Ok(tisty_core::series::routines(&session.state))
}

#[tauri::command]
fn archive_shape(session: tauri::State<'_, Mutex<Session>>) -> Answer<tisty_core::shape::Shape> {
    let mut session = held(&session);
    session.reload()?;
    let now = jiff::Zoned::now();
    Ok(tisty_core::shape::shape(
        &session.state,
        18,
        &now.time_zone().clone(),
        now.date(),
    ))
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
    priority: Option<String>,
    #[serde(default)]
    take_offer: bool,
}

fn named_priority(raw: &str) -> Answer<tisty_core::Priority> {
    raw.parse().map_err(|_| Refusal::of("notAPriority"))
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
        if let Some(name) = &self.priority {
            draft.priority = Some(named_priority(name)?);
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
    priority: Option<String>,
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
        priority: change.priority.as_deref().map(named_priority).transpose()?,
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

#[tauri::command(async)]
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
    let tasks: Vec<Task> = hits.into_iter().cloned().collect();
    let listed: std::collections::BTreeMap<String, bool> = session
        .state
        .docs
        .values()
        .filter(|one| match scope {
            Scope::Open => !one.archived,
            Scope::Archived => one.archived,
            Scope::Either => true,
        })
        .map(|one| (one.file.clone(), one.archived))
        .collect();
    let root = session.paths.docs();
    let papers = session
        .corpus
        .searching(&root, &query, PAPERS_MOST, |id| listed.contains_key(id))
        .into_iter()
        .map(|one| Paper {
            archived: listed.get(&one.id).copied().unwrap_or(false),
            id: one.id,
            title: one.title,
            line: one.line,
        })
        .collect();
    Ok(Found {
        tasks,
        total,
        papers,
    })
}

const MOST: usize = 200;
const PAPERS_MOST: usize = 40;

#[derive(serde::Serialize)]
struct Paper {
    id: String,
    title: String,
    line: String,
    archived: bool,
}

#[derive(serde::Serialize)]
struct Found {
    tasks: Vec<Task>,
    papers: Vec<Paper>,
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
    heard: Option<String>,
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
fn twinned(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<tisty_core::attach::Twins>> {
    let data = held(&session).paths.data().to_path_buf();
    Ok(tisty_core::attach::twins(&data))
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
    let adrift = session.adrift(&held);

    let kept = report::attachments(session.paths.data());
    let told = tisty_core::store::read_all(session.paths.store()).unwrap_or_default();

    Ok(Reviewed {
        tasks: session.state.tasks.len(),
        lists: session.state.lists.len(),
        agrees: matches!(audit, tisty_core::cache::Audit::Agrees { .. }),
        loose: adrift.files(),
        loose_bytes: adrift.bytes,
        astray: adrift.items,
        stranded: tisty_core::docs::loose(
            &session.paths.docs(),
            &session
                .state
                .docs
                .values()
                .map(|one| one.file.clone())
                .collect::<Vec<_>>(),
        )
        .len(),
        events: told.len(),
        machines: report::machines(
            &told,
            session.config.device_id.0.as_str(),
            &session.state.dropped,
        ),
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
    stranded: usize,
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
    let adrift = session.adrift(&referenced);
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
    "noSuchColour",
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
    stuck: Option<Refusal>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    locale: Option<String>,
    holds: tisty_core::config::Holds,
    /// Whether the choice means anything here: without a shared folder there is nowhere else.
    shares: bool,
    only_shared_above: u64,
}

const HERE: &str = env!("CARGO_PKG_VERSION");

#[tauri::command]
async fn update_ready(
    session: tauri::State<'_, Mutex<Session>>,
    now_please: Option<bool>,
) -> Answer<Option<update::Ready>> {
    let (last, found) = {
        let held = held(&session);
        (held.config.checked_at, held.config.found_version.clone())
    };
    let now = jiff::Timestamp::now();
    if !now_please.unwrap_or(false) && !update::due(last, now) {
        return Ok(update::remembered(HERE, found.as_deref(), update::route()));
    }

    let manifest = tauri::async_runtime::spawn_blocking(update::fetch)
        .await
        .map_err(|_| Refusal::of("internal"))?;

    // A look that never answered says nothing about whether an update is owed, so what was found
    // before stays where it is.
    let Some(manifest) = manifest else {
        return Ok(update::remembered(HERE, found.as_deref(), update::route()));
    };

    let seen = update::newer(HERE, &manifest, update::route());
    let version = seen.as_ref().map(|one| one.version.clone());
    held(&session).keep(|c| {
        c.checked_at = Some(now);
        c.found_version = version;
    })?;
    Ok(seen)
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Underway {
    stage: &'static str,
    far: u64,
}

#[tauri::command]
async fn update_install(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Updating>,
) -> Answer<()> {
    use tauri_plugin_updater::UpdaterExt;

    let _busy = alone
        .inner()
        .0
        .claim()
        .ok_or_else(|| Refusal::of("updateBusy"))?;

    let kept = update::route();
    if !update::self_installs(kept.route) || update::from_a_mount() {
        return Err(Refusal::of("updateNotHere"));
    }

    let found = held(&session).config.found_version.clone();
    let Some(want) = update::remembered(HERE, found.as_deref(), kept).map(|one| one.version) else {
        return Err(Refusal::of("updateGone"));
    };

    let asked = want.clone();
    let update = app
        .updater_builder()
        .endpoints(vec![
            update::channel_for(&want)
                .parse()
                .map_err(|_| Refusal::of("internal"))?,
        ])
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?
        // Pinned to what the person was shown, so a feed that moves in between cannot quietly
        // hand them a different version than the one they agreed to.
        .version_comparator(move |_, release| release.version.to_string() == asked)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?
        .check()
        .await
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?;

    let Some(mut update) = update else {
        return Err(Refusal::of("updateGone"));
    };

    // The feed names the address the installer comes from, so it is checked against where our
    // releases actually live before a single byte is asked for.
    if !update::ours(update.download_url.as_str()) {
        return Err(Refusal::of("updateElsewhere"));
    }
    // The plugin builds the download with no deadline of its own, and a server that dribbles
    // bytes forever would otherwise be waited on forever.
    update.timeout = Some(std::time::Duration::from_secs(600));

    let telling = app.clone();
    let done = app.clone();
    let mut carried: u64 = 0;
    let mut said = 0;
    update
        .download_and_install(
            move |chunk, whole| {
                // The callback hands over the length of one chunk, not how much has arrived.
                carried += chunk as u64;
                let far = whole.map_or(0, |all| carried * 100 / all.max(1));
                if far != said {
                    said = far;
                    let _ = telling.emit(
                        "updating",
                        Underway {
                            stage: "getting",
                            far,
                        },
                    );
                }
            },
            // The last thing anyone sees on Windows: the installer takes the process with it and
            // nothing after the await ever runs.
            move || {
                let _ = done.emit(
                    "updating",
                    Underway {
                        stage: "installing",
                        far: 100,
                    },
                );
            },
        )
        .await
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?;

    // Only macOS gets this far, and restarting has to happen where the app loop lives.
    let handle = app.clone();
    app.run_on_main_thread(move || handle.restart())
        .map_err(|why| Refusal::about("updateFailed", why.to_string()))?;
    Ok(())
}

#[tauri::command]
fn settings(session: tauri::State<'_, Mutex<Session>>) -> Answer<Settings> {
    let session = held(&session);
    Ok(as_settings(&session))
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Freeing {
    gone: usize,
    freed: u64,
    done: bool,
}

#[derive(Default)]
struct Stopping(std::sync::atomic::AtomicBool);

/// Turning it on is the only change that moves anything, so it is asked for rather than done on
/// the way past: it can take an afternoon, and somebody may want it to stop.
#[tauri::command(async)]
async fn free_up(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    stopping: tauri::State<'_, Stopping>,
) -> Answer<Freeing> {
    let _done = alone.inner().taken()?;
    stopping
        .0
        .store(false, std::sync::atomic::Ordering::Relaxed);
    let (data, dest, above) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            session.paths.data().to_path_buf(),
            dest,
            session.config.only_shared_above(),
        )
    };

    let telling = app.clone();
    let done = tauri::async_runtime::spawn_blocking(move || {
        let mut said = 0;
        tisty_sync::let_go_telling(&data, &dest, above, &mut |far| {
            if far.gone != said {
                said = far.gone;
                let _ = telling.emit(
                    "freeing",
                    Freeing {
                        gone: far.gone,
                        freed: far.freed,
                        done: false,
                    },
                );
            }
            !telling
                .state::<Stopping>()
                .0
                .load(std::sync::atomic::Ordering::Relaxed)
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(said)?;

    witness::note(
        channel::SYNC,
        "big attachments were left to the shared folder",
        &[
            ("count", Fact::Count(done.gone)),
            ("bytes", Fact::Bytes(done.freed)),
        ],
    );
    let now = Freeing {
        gone: done.gone,
        freed: done.freed,
        done: true,
    };
    let _ = app.emit("freeing", now.clone());
    Ok(now)
}

#[tauri::command]
fn stop_freeing(stopping: tauri::State<'_, Stopping>) {
    stopping.0.store(true, std::sync::atomic::Ordering::Relaxed);
}

fn as_settings(session: &Session) -> Settings {
    Settings {
        quiet: session.config.muted().to_vec(),
        attach_up_to: session.config.copies_up_to(),
        locale: session.config.locale.clone(),
        holds: session.config.holds.unwrap_or_default(),
        shares: !session.config.backs_up(),
        only_shared_above: session.config.only_shared_above(),
    }
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
    let holds = settings.holds;
    session.keep(|config| {
        config.quiet = (!quiet.is_empty()).then_some(quiet);
        config.attach_up_to = Some(up_to);
        config.holds = Some(holds);
    })?;
    let now = as_settings(&session);
    drop(session);
    herald::respeak(&app, &now.quiet);
    Ok(now)
}

#[tauri::command]
fn icons() -> Vec<&'static str> {
    tisty_core::model::icon::ICONS.to_vec()
}

#[tauri::command]
fn families() -> Vec<(&'static str, usize)> {
    tisty_core::model::icon::FAMILIES.to_vec()
}

#[tauri::command]
fn list_add(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    icon: Option<String>,
    color: Option<String>,
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
    let painted = color.filter(|key| tisty_core::model::hue::kept(key).is_some());
    let drawn = icon.filter(|key| tisty_core::model::icon::known(key));
    if drawn.is_some() || painted.is_some() {
        session.commit(Op::ListLook {
            id,
            d: tisty_core::event::Look {
                icon: Some(drawn),
                color: Some(painted),
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
    color: Option<String>,
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
    let painted = match color {
        Some(key) => Some(
            tisty_core::model::hue::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| Refusal::about("noSuchColour", key))?,
        ),
        None => None,
    };

    let mut session = held(&session);
    session.commit(Op::ListLook {
        id,
        d: tisty_core::event::Look {
            icon: Some(kept),
            color: Some(painted),
        },
    })?;
    session
        .state
        .lists
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notAListId"))
}

#[tauri::command]
fn list_rename(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    name: String,
) -> Answer<List> {
    let id: tisty_core::ListId = id.parse().map_err(|_| Refusal::of("notAListId"))?;
    let name = tisty_core::text::plainly(&name);
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }

    let mut session = held(&session);
    if !session.state.lists.contains_key(&id) {
        return Err(Refusal::of("notAListId"));
    }
    if session
        .state
        .list_called(&name)
        .iter()
        .any(|one| one.id != id)
    {
        return Err(Refusal::about("manyLists", name));
    }

    session.commit(Op::ListRename {
        id,
        d: tisty_core::event::Name { name },
    })?;
    session
        .state
        .lists
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notAListId"))
}

#[tauri::command]
fn list_drop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id: tisty_core::ListId = id.parse().map_err(|_| Refusal::of("notAListId"))?;
    let mut session = held(&session);
    if !session.state.lists.contains_key(&id) {
        return Err(Refusal::of("notAListId"));
    }
    session.commit(Op::ListDelete { id })?;
    Ok(())
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
    color: Option<String>,
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
    gone: bool,
    page_of: Option<String>,
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
        let found = named.get(kept.file.as_str());
        docs.push(Filed {
            id: kept.id.to_string(),
            file: kept.file.clone(),
            title: found.map(|one| one.title.clone()).unwrap_or_default(),
            folder: kept.folder.map(|at| at.to_string()),
            archived: kept.archived,
            gone: found.is_none(),
            page_of: kept.page_of.map(|up| up.to_string()),
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
                color: one.color.clone(),
                holds: state.held_by(one.id),
            }];
            branch.append(&mut hanging(state, Some(one.id)));
            branch
        })
        .collect()
}

fn named_folder(said: &str) -> Answer<String> {
    let name = tisty_core::text::plainly(said);
    if name.is_empty() {
        return Err(Refusal::of("untitled"));
    }
    if name.chars().count() > tisty_core::model::FOLDER_NAME_AT_MOST {
        return Err(Refusal::of("folderNameTooLong"));
    }
    Ok(name)
}

#[tauri::command]
fn folder_add(
    session: tauri::State<'_, Mutex<Session>>,
    name: String,
    parent: Option<String>,
    icon: Option<String>,
) -> Answer<()> {
    let name = named_folder(&name)?;
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
            color: None,
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
    let name = named_folder(&name)?;
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
    color: Option<String>,
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
    let painted = match color {
        Some(key) => Some(
            tisty_core::model::hue::kept(&key)
                .map(str::to_string)
                .ok_or_else(|| Refusal::about("noSuchColour", key))?,
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
            color: Some(painted),
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
            page_of: None,
            order: None,
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
    match session.state.docs.get(&id) {
        None => return Err(Refusal::of("noSuchDoc")),
        Some(one) if one.page_of.is_some() => return Err(Refusal::of("pageStaysPut")),
        Some(_) => {}
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
            page_of: None,
            order: None,
        },
    })?;
    Ok(())
}

#[tauri::command(async)]
fn doc_read(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<String> {
    let root = held(&session).paths.docs();
    let read = tisty_core::docs::read(&root, &id);
    if let Ok(body) = &read {
        held(&session).mind_body(&id, body);
    }
    read.map_err(|e| match e {
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

/// Long enough for a file already on its way, short enough that nobody thinks the app hung.
const COMES_WITHIN: std::time::Duration = std::time::Duration::from_millis(1_500);

fn unreachable(found: Sought, reference: String) -> Refusal {
    match found {
        Sought::Coming => Refusal::about("comingDown", reference),
        Sought::Away => Refusal::of("sharedAway"),
        Sought::Torn => Refusal::about("attachmentTorn", reference),
        _ => Refusal::about("cannotRead", reference),
    }
}

enum Sought {
    At(std::path::PathBuf),
    Coming,
    Away,
    /// It is there, and it is not what its name says it is.
    Torn,
    No,
}

/// A link or a junction under somebody else's folder can point anywhere; the store's tree is ours.
fn under_root(at: &std::path::Path, root: &std::path::Path) -> bool {
    match (at.canonicalize(), root.canonicalize()) {
        (Ok(at), Ok(root)) => at.starts_with(root),
        _ => false,
    }
}

/// The sync has always made that folder answer for its bytes; opening one asks the same, once per
/// file and again only when it changes size or date.
fn vouches(at: &std::path::Path, reference: &str) -> bool {
    static SEEN: std::sync::OnceLock<Mutex<std::collections::HashMap<String, bool>>> =
        std::sync::OnceLock::new();
    let mut parts = reference.rsplit('/');
    let (Some(leaf), Some(shelf)) = (parts.next(), parts.next()) else {
        return false;
    };
    let Ok(told) = std::fs::metadata(at) else {
        return false;
    };
    let when = told
        .modified()
        .ok()
        .and_then(|one| one.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|one| one.as_secs())
        .unwrap_or(0);
    let asked = format!("{}|{}|{when}", at.display(), told.len());

    let seen = SEEN.get_or_init(Default::default);
    if let Some(held) = seen.lock().ok().and_then(|one| one.get(&asked).copied()) {
        return held;
    }
    let said = tisty_core::attach::hashed(at)
        .is_ok_and(|(sha256, _)| tisty_core::attach::vouched(shelf, leaf, &sha256));
    if let Ok(mut one) = seen.lock() {
        one.insert(asked, said);
    }
    said
}

/// Where to look, taken and let go of at once: what follows can wait on iCloud, and holding the
/// session while it does would freeze the window.
fn where_to(
    session: &tauri::State<'_, Mutex<Session>>,
) -> (std::path::PathBuf, Option<std::path::PathBuf>) {
    let session = held(session);
    (session.paths.data().to_path_buf(), session.shared_now())
}

/// The store first, then the shared folder, which is where a machine that let go of it kept it.
/// For a size or a count, where reading the whole of it to check would fetch what nobody asked to
/// open — and on a cloud folder, fetching is the one thing this setting exists to avoid.
fn where_it_lies(
    reference: &str,
    data: &std::path::Path,
    shared: Option<&std::path::Path>,
) -> Option<std::path::PathBuf> {
    for root in [Some(data), shared].into_iter().flatten() {
        let Ok(at) = tisty_core::attach::resolve(reference, root) else {
            continue;
        };
        if at.is_file() && (root == data || under_root(&at, root)) {
            return Some(at);
        }
    }
    None
}

fn found_in(reference: &str, data: &std::path::Path, shared: Option<&std::path::Path>) -> Sought {
    for root in [Some(data), shared].into_iter().flatten() {
        let Ok(at) = tisty_core::attach::resolve(reference, root) else {
            continue;
        };
        let ours = root == data;
        if at.is_file() {
            if !ours && !under_root(&at, root) {
                return Sought::No;
            }
            if !ours && !vouches(&at, reference) {
                return Sought::Torn;
            }
            return Sought::At(at);
        }
        if tisty_core::icloud::shed(&at).is_some() {
            if !tisty_core::icloud::can_ask() {
                return Sought::Away;
            }
            if !tisty_core::icloud::waited_for(&at, COMES_WITHIN) {
                return Sought::Coming;
            }
            // What comes back from a cloud answers for its name like anything else that lives there.
            return match ours || (under_root(&at, root) && vouches(&at, reference)) {
                true => Sought::At(at),
                false => Sought::Torn,
            };
        }
    }
    match shared {
        Some(dest) if !dest.is_dir() => Sought::Away,
        _ => Sought::No,
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Facts {
    made: Option<i64>,
    wrote: Option<i64>,
    bytes: u64,
    pages: usize,
}

fn seconds(at: std::io::Result<std::time::SystemTime>) -> Option<i64> {
    at.ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|gone| gone.as_secs() as i64)
}

#[tauri::command(async)]
fn keep_pdf(at: String, bytes: Vec<u8>) -> Answer<()> {
    std::fs::write(&at, bytes).map_err(|e| {
        blamed(
            channel::WINDOW,
            "a pdf could not be written",
            tisty_core::Error::Io(e),
        )
    })
}

#[tauri::command]
fn doc_facts(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Facts> {
    let session = held(&session);
    let root = session.paths.docs();
    let kept = session.state.docs.values().find(|one| one.file == id);
    let made = kept.map(|one| (one.id.timestamp_ms() / 1000) as i64);
    let pages = kept.map_or(0, |one| session.state.pages_of(one.id).len());
    let at = tisty_core::docs::resolve(&root, &id)
        .map_err(|_| Refusal::about("noSuchDoc", id.clone()))?;
    let about = std::fs::metadata(&at).map_err(|_| Refusal::about("noSuchDoc", id))?;
    Ok(Facts {
        made,
        wrote: seconds(about.modified()),
        bytes: about.len(),
        pages,
    })
}

const PICTURES: &[&str] = &[
    "captura.png",
    "prioridades.png",
    "capture.png",
    "priorities.png",
];

// The MSIX package ships the executable alone, so the guide travels inside the binary.
const GUIDE_ES: &str = include_str!("../resources/guide/es/guia.md");
const GUIDE_EN: &str = include_str!("../resources/guide/en/guide.md");

#[tauri::command]
fn guide(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
) -> Answer<tisty_core::docs::Doc> {
    let tongue = {
        let session = held(&session);
        let code = tisty_core::model::spoken(session.locale.as_deref());
        if code.starts_with("es") { "es" } else { "en" }
    };
    let called = if tongue == "es" { "Guía" } else { "Guide" };
    let told = if tongue == "es" { GUIDE_ES } else { GUIDE_EN };

    let from = app
        .path()
        .resolve(
            format!("resources/guide/{tongue}"),
            tauri::path::BaseDirectory::Resource,
        )
        .ok();

    let mut session = held(&session);

    if let Some(kept) = session.config.guide.clone() {
        let root = session.paths.docs();
        let standing = session.state.docs.values().any(|one| one.file == kept);
        if standing && let Ok(body) = tisty_core::docs::read(&root, &kept) {
            return Ok(tisty_core::docs::Doc {
                id: kept,
                title: tisty_core::docs::titled(&body),
            });
        }
    }

    let data = session.paths.data().to_path_buf();

    let mut body = told.to_string();
    for shot in PICTURES {
        let Some(at) = from
            .as_ref()
            .map(|dir| dir.join(shot))
            .filter(|at| at.is_file())
        else {
            continue;
        };
        let kept = tisty_core::attach::keep(&at, &data, tisty_core::attach::COPIED_IN_DOC)
            .map_err(|e| Refusal::about("cannotRead", e.to_string()))?;
        body = body.replace(&format!("]({shot})"), &format!("](<{}>)", kept.at));
    }

    let folder = ulid::Ulid::generate();
    let order = tisty_core::order::last_of(
        session
            .state
            .under(None)
            .iter()
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::FolderAdd {
        id: folder,
        d: tisty_core::event::FolderAdd {
            name: called.to_string(),
            order,
            parent: None,
            icon: None,
            color: None,
        },
    })?;

    let root = session.paths.docs();
    let device = session.store.device().clone();
    let made = tisty_core::docs::create(&root, &device, &body)
        .map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;

    let sorted = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.folder == Some(folder))
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            file: made.id.clone(),
            order: sorted,
            folder: Some(folder),
            page_of: None,
        },
    })?;
    let written = made.id.clone();
    session.keep(|c| c.guide = Some(written))?;

    Ok(made)
}

fn stale(mine: Option<&str>, now: Option<&str>) -> bool {
    matches!((mine, now), (Some(mine), Some(now)) if mine != now)
}

#[tauri::command(async)]
fn doc_write(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
    anyway: Option<bool>,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    if !anyway.unwrap_or(false) && session.moved(&id) {
        return Err(Refusal::about("documentMoved", id));
    }
    let root = session.paths.docs();
    tisty_core::docs::write(&root, &id, &body).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooLong", weighed(limit))
        }
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        _ => blamed(channel::WINDOW, "a document could not be written", e),
    })?;
    session.mind_body(&id, &tisty_core::docs::settled(&body));
    session.corpus.forget(&id);
    Ok(tisty_core::docs::Doc {
        title: tisty_core::docs::titled(&body),
        id,
    })
}

#[tauri::command]
fn doc_away(session: tauri::State<'_, Mutex<Session>>, id: String, away: bool) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let mut session = held(&session);
    match session.state.docs.get(&id) {
        None => return Err(Refusal::of("noSuchDoc")),
        Some(one) if one.page_of.is_some() => return Err(Refusal::of("pageStaysPut")),
        Some(_) => {}
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
            .filter(|one| {
                one.page_of == kept.page_of && (kept.page_of.is_some() || one.folder == kept.folder)
            })
            .map(|one| one.order.as_str()),
    );
    let twin = ulid::Ulid::generate();
    session.commit(Op::DocAdd {
        id: twin,
        d: tisty_core::event::DocAdd {
            file: made.id.clone(),
            order,
            folder: kept.folder,
            page_of: kept.page_of,
        },
    })?;
    if kept.archived && kept.page_of.is_none() {
        session.commit(Op::DocArchive { id: twin })?;
    }

    // A page is part of its document, so the copy is not the same document without them.
    for page in session
        .state
        .pages_of(id)
        .iter()
        .map(|one| one.file.clone())
        .collect::<Vec<_>>()
    {
        let body = tisty_core::docs::read(&root, &page).unwrap_or_default();
        let leaf = tisty_core::docs::create(&root, &session.config.device_id, &body)
            .map_err(|e| blamed(channel::WINDOW, "a page could not be copied", e))?;
        let order = tisty_core::order::last_of(
            session
                .state
                .docs
                .values()
                .filter(|one| one.page_of == Some(twin))
                .map(|one| one.order.as_str()),
        );
        session.commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: leaf.id,
                order,
                folder: kept.folder,
                page_of: Some(twin),
            },
        })?;
    }
    Ok(made)
}

#[tauri::command(async)]
fn doc_export(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    into: String,
) -> Answer<usize> {
    let session = held(&session);
    let pages: Vec<String> = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .map(|one| {
            session
                .state
                .pages_of(one.id)
                .iter()
                .map(|page| page.file.clone())
                .collect()
        })
        .unwrap_or_default();
    tisty_core::docs::with_pages(
        session.paths.data(),
        &id,
        &pages,
        std::path::Path::new(&into),
    )
    .map_err(|e| {
        witness::warn(
            channel::WINDOW,
            "a document could not be taken out",
            &[
                ("id", Fact::Id(id.clone())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        Refusal::about("cannotWrite", into)
    })
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
            page_of: None,
        },
    })?;
    Ok(made)
}

#[tauri::command]
fn doc_new(
    session: tauri::State<'_, Mutex<Session>>,
    folder: Option<String>,
    page_of: Option<String>,
) -> Answer<tisty_core::docs::Doc> {
    let folder = folder
        .map(|at| at.parse().map_err(|_| Refusal::of("noSuchFolder")))
        .transpose()?;
    let page_of = page_of
        .map(|up| up.parse().map_err(|_| Refusal::of("noSuchDoc")))
        .transpose()?;
    let mut session = held(&session);
    if let Some(at) = folder
        && !session.state.folders.contains_key(&at)
    {
        return Err(Refusal::of("noSuchFolder"));
    }
    let under = match page_of {
        Some(up) => match session.state.docs.get(&up) {
            None => return Err(Refusal::of("noSuchDoc")),
            Some(one) if one.page_of.is_some() => return Err(Refusal::of("pageOfPage")),
            Some(one) => Some(one.folder),
        },
        None => None,
    };
    let folder = under.unwrap_or(folder);
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "")
        .map_err(|e| blamed(channel::WINDOW, "a document could not be made", e))?;

    let order = tisty_core::order::last_of(
        session
            .state
            .docs
            .values()
            .filter(|one| one.page_of == page_of && (page_of.is_some() || one.folder == folder))
            .map(|one| one.order.as_str()),
    );
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            file: made.id.clone(),
            order,
            folder,
            page_of,
        },
    })?;
    Ok(made)
}

#[tauri::command]
fn doc_page(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    page_of: Option<String>,
) -> Answer<()> {
    let id: tisty_core::model::DocId = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
    let page_of = page_of
        .map(|up| up.parse().map_err(|_| Refusal::of("noSuchDoc")))
        .transpose()?;
    let mut session = held(&session);

    if !session.state.docs.contains_key(&id) {
        return Err(Refusal::of("noSuchDoc"));
    }
    if let Some(up) = page_of {
        if up == id {
            return Err(Refusal::of("pageOfPage"));
        }
        match session.state.docs.get(&up) {
            None => return Err(Refusal::of("noSuchDoc")),
            Some(one) if one.page_of.is_some() => return Err(Refusal::of("pageOfPage")),
            Some(_) => {}
        }
        if session
            .state
            .docs
            .values()
            .any(|one| one.page_of == Some(id))
        {
            return Err(Refusal::of("holdsPages"));
        }
    }

    session.commit(Op::DocMove {
        id,
        d: tisty_core::event::Filed {
            folder: None,
            page_of: Some(page_of),
            order: None,
        },
    })?;
    Ok(())
}

#[tauri::command]
fn doc_drop(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let mut session = held(&session);

    let files = match id.parse() {
        Ok(id) => {
            let kept = session
                .state
                .docs
                .get(&id)
                .ok_or_else(|| Refusal::of("noSuchDoc"))?;
            let mut files = vec![kept.file.clone()];
            files.extend(
                session
                    .state
                    .pages_of(id)
                    .iter()
                    .map(|one| one.file.clone()),
            );
            session.commit(Op::DocDelete { id })?;
            files
        }
        Err(_) => vec![id],
    };

    let root = session.paths.docs();
    let mut said = tisty_core::docs::Carried::read(session.paths.data());
    for file in &files {
        tisty_core::docs::remove(&root, file)
            .map_err(|e| blamed(channel::WINDOW, "a document could not be removed", e))?;

        if let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() {
            tisty_sync::forget_paper(&dest, file);
        }
        said.forget(file);
        tisty_core::docs::forget_carried(session.paths.data(), file);
    }
    let _ = said.save(session.paths.data());
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
        // Private selectors: asked before told, since a missing one raises an ObjC exception Rust cannot catch.
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
fn sow(app: tauri::AppHandle, priority: Option<String>) {
    tray::sow(&app, priority);
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
    let (was, dest, data, store, aside, device, alive, holds) = {
        let session = held(&session);
        let was = session.config.opened_by.clone();
        if was.as_deref() == Some(here) {
            return Ok(Settling {
                ran: false,
                brought: false,
                agrees: true,
                was,
                stuck: None,
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
            session.paths.cache().to_path_buf(),
            session.config.device_id.0.clone(),
            session.alive(),
            session.config.holds(),
        )
    };

    let mut brought = false;
    let mut stuck = None;
    let mut carried = dest.is_none();
    if let Some(dest) = dest
        && let Some(_done) = alone.inner().claim()
    {
        carried = true;
        let before = tisty_core::cache::fingerprint(&store);
        let carried = tauri::async_runtime::spawn_blocking(move || {
            tisty_sync::carry_holding(
                &data,
                Some(&aside),
                &device,
                &dest,
                tisty_sync::Way::Both,
                &alive,
                holds,
            )
        })
        .await;
        match carried {
            Ok(Err(why)) => {
                let refusal = said(why);
                witness::warn(
                    channel::SYNC,
                    "the carry on opening did not finish",
                    &[("code", Fact::Code(refusal.code))],
                );
                stuck = Some(refusal);
            }
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
        stuck,
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
fn wiring() -> Vec<wiring::Seen> {
    wiring::seen()
}

#[tauri::command]
fn wire(id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::wire(&id).map_err(stuck)
}

#[tauri::command]
fn unwire(id: String) -> Answer<Vec<wiring::Seen>> {
    wiring::unwire(&id).map_err(stuck)
}

fn stuck(why: wiring::Stuck) -> Refusal {
    match why {
        wiring::Stuck::NoSuch => Refusal::of("noSuchAgent"),
        wiring::Stuck::Puzzling(at) => Refusal::about("settingsPuzzling", at),
        wiring::Stuck::Cannot(why) => Refusal::about("cannotWrite", why),
    }
}

#[tauri::command]
fn waking() -> waking::Waking {
    waking::waking()
}

#[tauri::command]
fn wake_for(wanted: bool) -> Answer<waking::Waking> {
    waking::wake(wanted).map_err(|e| Refusal::about("cannotWrite", e.to_string()))
}

#[tauri::command]
fn keep_locale(
    session: tauri::State<'_, Mutex<Session>>,
    locale: Option<String>,
) -> Answer<Option<String>> {
    let mut session = held(&session);
    let wanted = locale.filter(|one| !one.trim().is_empty());
    session.keep(|config| config.locale = wanted.clone())?;
    Ok(wanted)
}

#[tauri::command]
fn keep_closing(session: tauri::State<'_, Mutex<Session>>, how: String) -> Answer<()> {
    let how = match how.as_str() {
        "hide" => tisty_core::config::Closing::Hide,
        "quit" => tisty_core::config::Closing::Quit,
        _ => return Err(Refusal::of("notAClosing")),
    };
    held(&session).keep(|config| config.on_close = Some(how))?;
    Ok(())
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
        heard: config.heard_at.map(|at| at.to_string()),
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
    // Leaving a folder that holds what this machine let go of takes them with it — but only while
    // it is there to bring them back from. Gone, refusing would trap somebody with nowhere to go.
    if session.config.holds() != tisty_core::config::Holds::Everywhere
        && session.config.sync != Some(chosen.clone())
        && let Some(tisty_core::config::Sync::Folder(old)) = session.config.sync.clone()
        && old.is_dir()
    {
        return Err(Refusal::about(
            "sharedAwayToLeave",
            old.display().to_string(),
        ));
    }
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
) -> Answer<Settled> {
    let Some(_done) = alone.inner().claim() else {
        return Ok(Settled {
            carried: "busy",
            undecided: Vec::new(),
            unreadable: Vec::new(),
            astray: Vec::new(),
            joined: Vec::new(),
        });
    };

    let (dest, data, store, aside, device, alive, holds) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            dest,
            session.paths.data().to_path_buf(),
            session.paths.store(),
            session.paths.cache().to_path_buf(),
            session.config.device_id.0.clone(),
            session.alive(),
            session.config.holds(),
        )
    };

    let before = tisty_core::cache::fingerprint(&store);
    let way = match way.as_deref() {
        Some("push") => tisty_sync::Way::Push,
        Some("pull") => tisty_sync::Way::Pull,
        Some("again") => tisty_sync::Way::Again,
        _ => tisty_sync::Way::Both,
    };

    let done = tauri::async_runtime::spawn_blocking(move || {
        tisty_sync::carry_holding(&data, Some(&aside), &device, &dest, way, &alive, holds)
    })
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
    session.take_out_the_shed();
    session.take_out_the_retired();
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
    let heard = done.brought > 0;
    session.keep(|c| {
        c.synced_at = Some(jiff::Timestamp::now());
        if heard {
            c.heard_at = c.synced_at;
        }
    })?;
    Ok(Settled {
        carried: match (done.sent > 0, moved || done.brought > 0) {
            (true, true) => "both",
            (true, false) => "sent",
            (false, true) => "came",
            (false, false) => "same",
        },
        undecided: done.undecided.into_iter().map(|one| one.id).collect(),
        unreadable: done.unreadable,
        astray: done.astray,
        joined: done.joined,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Settled {
    carried: &'static str,
    undecided: Vec<String>,
    unreadable: Vec<String>,
    astray: Vec<String>,
    joined: Vec<String>,
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
fn convert_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
) -> Answer<()> {
    let mut session = held(&session);
    let papers = session.paths.docs();
    let was = tisty_core::docs::read(&papers, &id)
        .map_err(|_| Refusal::about("cannotRead", id.clone()))?;

    tisty_core::docs::kept_before(session.paths.data(), &id, &was)
        .map_err(|e| blamed(channel::SYNC, "what it was could not be kept", e))?;
    tisty_core::docs::write(&papers, &id, &body).map_err(|e| match e {
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        e => blamed(
            channel::SYNC,
            "the converted document could not be written",
            e,
        ),
    })?;
    session.mind(&id);
    Ok(())
}

type Placing = (Option<ulid::Ulid>, Option<ulid::Ulid>, String);

fn placed(beside: Option<Placing>, fresh: &str) -> Placing {
    match beside {
        Some((folder, page_of, order)) => (folder, page_of, tisty_core::order::after(&order)),
        None => (None, None, fresh.to_string()),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Torn {
    rifts: Vec<tisty_core::merge::Rift>,
    print: String,
}

fn three_bodies(session: &Session, id: &str) -> Answer<Option<(String, String, String)>> {
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    let Some(base) = tisty_core::docs::read_carried(session.paths.data(), id) else {
        return Ok(None);
    };
    match tisty_sync::both_papers(session.paths.data(), &dest, id) {
        Ok((mine, theirs)) => Ok(Some((base, mine, theirs))),
        Err(_) => Ok(None),
    }
}

fn print_of_three(base: &str, mine: &str, theirs: &str) -> String {
    tisty_core::attach::printed(format!("{base}\u{0}{mine}\u{0}{theirs}").as_bytes())
}

#[tauri::command(async)]
fn paper_rifts(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Torn> {
    let session = held(&session);
    let Some((base, mine, theirs)) = three_bodies(&session, &id)? else {
        return Ok(Torn {
            rifts: Vec::new(),
            print: String::new(),
        });
    };
    Ok(Torn {
        print: print_of_three(&base, &mine, &theirs),
        rifts: tisty_core::merge::rifts(&base, &mine, &theirs),
    })
}

#[tauri::command(async)]
fn weave_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    picks: Vec<String>,
    print: String,
) -> Answer<()> {
    let mut session = held(&session);
    let Some((base, mine, theirs)) = three_bodies(&session, &id)? else {
        return Err(Refusal::of("noBase"));
    };
    if print_of_three(&base, &mine, &theirs) != print {
        return Err(Refusal::of("movedUnderfoot"));
    }
    let picked: Vec<tisty_core::merge::Pick> = picks
        .iter()
        .map(|one| match one.as_str() {
            "mine" => tisty_core::merge::Pick::Mine,
            "theirs" => tisty_core::merge::Pick::Theirs,
            _ => tisty_core::merge::Pick::Both,
        })
        .collect();
    let whole = tisty_core::merge::woven_with(&base, &mine, &theirs, &picked)
        .ok_or_else(|| Refusal::of("cannotWeave"))?;

    let papers = session.paths.docs();
    tisty_core::docs::kept_before(session.paths.data(), &id, &mine)
        .map_err(|e| blamed(channel::SYNC, "what it was could not be kept", e))?;
    tisty_core::docs::write(&papers, &id, &whole).map_err(|e| match e {
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        e => blamed(channel::SYNC, "the woven body could not be written", e),
    })?;
    session.mind(&id);
    Ok(())
}

#[tauri::command]
fn settle_paper(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    keep: String,
    marked: Option<String>,
) -> Answer<Option<String>> {
    let mut session = held(&session);
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    let keep = match keep.as_str() {
        "mine" => tisty_sync::Keep::Mine,
        "theirs" => tisty_sync::Keep::Theirs,
        _ => tisty_sync::Keep::Both,
    };

    let data = session.paths.data().to_path_buf();
    let brought = tisty_sync::settle(&data, &dest, &id, keep).map_err(said)?;
    session.mind(&id);

    let Some(body) = brought else { return Ok(None) };
    let beside = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .map(|one| (one.folder, one.page_of, one.order.clone()));
    let body = match &marked {
        Some(said) => tisty_core::docs::marked(&body, said),
        None => body,
    };
    let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, &body)
        .map_err(|e| blamed(channel::SYNC, "the other version could not be kept", e))?;
    let file = made.id.clone();
    let (folder, page_of, order) = placed(beside, &made.id);
    session
        .commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                file: file.clone(),
                folder,
                order,
                page_of,
            },
        })
        .map_err(|e| blamed(channel::SYNC, "the other version was not written down", e))?;

    tisty_sync::settle(&data, &dest, &id, tisty_sync::Keep::Mine).map_err(said)?;
    session.mind(&id);
    Ok(Some(file))
}

#[tauri::command]
fn retire_attachment(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<()> {
    let mut session = held(&session);
    let now = jiff::Timestamp::now().as_second();

    let mut held_by: Vec<String> = session
        .state
        .tasks
        .values()
        .flat_map(|task| task.references())
        .map(|one| one.target)
        .collect();
    held_by.extend(tisty_core::docs::referenced(&session.paths.docs()));
    if held_by.iter().any(|one| one == &reference) {
        return Err(Refusal::about("stillReferenced", reference));
    }

    // One never carried here has no bin to wait in; the retirement travels and the sweep acts.
    let here =
        tisty_core::attach::resolve(&reference, session.paths.data()).is_ok_and(|at| at.is_file());
    if here {
        tisty_core::attach::set_aside(session.paths.data(), &reference, now).map_err(|e| {
            witness::warn(
                channel::ATTACH,
                "an attachment could not be set aside",
                &[
                    ("at", Fact::Id(reference.clone())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            Refusal::about("cannotWrite", reference.clone())
        })?;
    }

    session
        .commit(Op::AttachRetire { d: reference })
        .map_err(|e| blamed(channel::ATTACH, "the retirement could not be written", e))?;
    session.take_out_the_shed();
    session.take_out_the_retired();
    session.reproject().map_err(|e| {
        blamed(
            channel::CACHE,
            "the store would not project after retiring",
            e,
        )
    })?;
    Ok(())
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
    session.reproject().map_err(|e| {
        blamed(
            channel::CACHE,
            "the store would not project after removing",
            e,
        )
    })?;

    witness::note(
        channel::SYNC,
        "a machine was removed and what it wrote was left where everyone can still read it",
        &[("at", Fact::Id(id))],
    );
    Ok(())
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
async fn take_over(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<u64> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (dest, aside, ours) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        let ours = tisty_core::store::identity(session.paths.store())
            .map_err(|e| blamed(channel::SYNC, "this machine has no name of its own", e))?;
        (dest, session.paths.cache().to_path_buf(), ours)
    };

    let at = std::path::PathBuf::from(&into);
    let made = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::backup::take_over(&dest, &ours, &at, &aside)
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::error(
            channel::BACKUP,
            "the folder was left alone because its backup did not land",
            &[("why", Fact::Why(e.to_string()))],
        );
        Refusal::about("cannotWrite", into)
    })?;

    Ok(made.bytes)
}

#[tauri::command(async)]
fn sync_kin(session: tauri::State<'_, Mutex<Session>>) -> Answer<&'static str> {
    let session = held(&session);
    let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
        return Err(Refusal::of("noRemote"));
    };
    Ok(match tisty_sync::kinship(&session.paths.store(), &dest) {
        tisty_sync::Kin::SameLineage => "sameLineage",
        tisty_sync::Kin::Clash(_) => "clash",
        tisty_sync::Kin::Unsure(_) => "unsure",
        tisty_sync::Kin::Strangers => "strangers",
    })
}

#[tauri::command]
async fn merge_stores(
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, OneAtATime>,
    into: String,
) -> Answer<bool> {
    let _done = alone.inner().taken()?;
    if tisty_core::paths::profile().is_some() {
        return Err(Refusal::of("sandboxCannotJoin"));
    }
    let (data, dest, aside, device) = {
        let session = held(&session);
        let Some(tisty_core::config::Sync::Folder(dest)) = session.config.sync.clone() else {
            return Err(Refusal::of("noRemote"));
        };
        (
            session.paths.data().to_path_buf(),
            dest,
            session.paths.cache().to_path_buf(),
            session.config.device_id.0.clone(),
        )
    };

    let at = std::path::PathBuf::from(&into);
    let seam = tauri::async_runtime::spawn_blocking(move || -> Answer<tisty_sync::Stitched> {
        tisty_core::backup::write(&data, &at, &aside).map_err(|e| {
            witness::error(
                channel::BACKUP,
                "nothing was joined because the backup did not land",
                &[("why", Fact::Why(e.to_string()))],
            );
            Refusal::about("cannotWrite", into)
        })?;
        tisty_sync::stitch(&data, &device, &dest).map_err(|trouble| {
            let refusal = said(trouble);
            witness::warn(
                channel::SYNC,
                "the two histories were left apart",
                &[("code", Fact::Code(refusal.code))],
            );
            refusal
        })
    })
    .await
    .map_err(|_| Refusal::of("internal"))??;

    *held(&session) = Session::open().map_err(|e| {
        blamed(
            channel::BACKUP,
            "the session would not reopen after joining",
            e,
        )
    })?;
    Ok(seam.stitch.is_some())
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
        tisty_sync::Trouble::Newer(who) => Refusal::about("syncNewer", who),
        tisty_sync::Trouble::Unreadable(why) => Refusal::about("syncUnreadable", why),
        tisty_sync::Trouble::Refused(why) => Refusal::about("syncRefused", why),
        tisty_sync::Trouble::Broke(why) => Refusal::about("syncBroke", why),
        tisty_sync::Trouble::WouldReset { theirs } => Refusal::about("wouldReset", theirs),
        tisty_sync::Trouble::NotAllowed(who) => Refusal::about("notAllowed", who),
        tisty_sync::Trouble::Emptied(at) => Refusal::about("emptiedPlace", at),
        tisty_sync::Trouble::SameName(who) => {
            Refusal::about("sameName", tisty_core::config::nicknamed(&who))
        }
    }
}

#[tauri::command(async)]
fn attached(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<Vec<u8>> {
    let (data, shared) = where_to(&session);
    let at = match found_in(&reference, &data, shared.as_deref()) {
        Sought::At(at) => at,
        other => return Err(unreachable(other, reference)),
    };
    std::fs::read(&at).map_err(|_| Refusal::about("cannotRead", reference))
}

#[tauri::command(async)]
fn served(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<String> {
    let (data, shared) = where_to(&session);
    let at = match found_in(&reference, &data, shared.as_deref()) {
        Sought::At(at) => at,
        other => return Err(unreachable(other, reference)),
    };
    Ok(at.to_string_lossy().into_owned())
}

#[tauri::command(async)]
fn attach_export(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
    into: String,
) -> Answer<()> {
    let (data, shared) = where_to(&session);
    let from = match found_in(&reference, &data, shared.as_deref()) {
        Sought::At(at) => at,
        other => return Err(unreachable(other, reference)),
    };
    std::fs::copy(&from, &into).map_err(|e| {
        witness::warn(
            channel::ATTACH,
            "an attachment could not be taken out",
            &[
                ("at", Fact::Id(reference)),
                ("why", Fact::Why(e.to_string())),
            ],
        );
        Refusal::about("cannotWrite", into)
    })?;
    Ok(())
}

#[tauri::command]
fn roomy() -> u64 {
    tisty_core::docs::BODY_ROOMY
}

#[tauri::command(async)]
fn weighs(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<u64> {
    let (data, shared) = where_to(&session);
    let at = where_it_lies(&reference, &data, shared.as_deref())
        .ok_or_else(|| Refusal::about("cannotRead", reference.clone()))?;
    let told = std::fs::metadata(&at).map_err(|_| Refusal::about("cannotRead", reference))?;
    Ok(told.len())
}

#[tauri::command(async)]
fn opened(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
) -> Answer<()> {
    let (data, shared) = where_to(&session);
    let at = match found_in(&reference, &data, shared.as_deref()) {
        Sought::At(at) => at,
        other => return Err(unreachable(other, reference)),
    };
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

    let (data, config, shared) = {
        let session = held(&session);
        (
            session.paths.data().to_path_buf(),
            session.paths.config().to_path_buf(),
            match &session.config.sync {
                Some(tisty_core::config::Sync::Folder(at)) => Some(at.clone()),
                _ => None,
            },
        )
    };
    let ours: Vec<std::path::PathBuf> = [Some(data), Some(config), shared]
        .into_iter()
        .flatten()
        .collect();
    if !within(at, &ours) {
        return Err(Refusal::about("cannotOpen", path));
    }
    show(at, &path)
}

fn within(at: &std::path::Path, ours: &[std::path::PathBuf]) -> bool {
    let Ok(real) = at.canonicalize() else {
        return false;
    };
    ours.iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| real.starts_with(&one))
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
fn owed(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<Vec<String>> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let session = held(&session);
    let today = jiff::Zoned::now().date();
    Ok(session
        .state
        .owed_since(id, today)
        .iter()
        .map(ToString::to_string)
        .collect())
}

#[tauri::command]
fn complete(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    also: Option<Vec<String>>,
) -> Answer<Task> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let also = also
        .unwrap_or_default()
        .iter()
        .map(|day| {
            day.parse::<jiff::civil::Date>()
                .map_err(|_| Refusal::about("notADate", day))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut session = held(&session);
    let ops = if also.is_empty() {
        session.state.completing(id, jiff::Zoned::now())
    } else {
        session.state.covering(id, jiff::Zoned::now(), &also)
    };
    session.commit_all(ops)?;
    let task = session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    drop(session);
    if task.status == tisty_core::Status::Done {
        let _ = herald::told(
            &app,
            tisty_core::herald::Happening::Done {
                title: task.title.clone(),
            },
        );
    }
    Ok(task)
}

#[tauri::command]
fn erase(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<()> {
    let id = id.parse().map_err(|_| Refusal::of("notATaskId"))?;
    let mut session = held(&session);
    let task = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    if !(task.is_archived() && task.folded()) {
        return Err(Refusal::of("onlyArchivedGoes"));
    }
    session.commit(Op::TaskDelete { id })?;
    Ok(())
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
struct Updating(OneAtATime);

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
    let _ = waking::wake(false);
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
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            // An update relaunches with the arguments it was started with, so a copy that opened
            // with the session comes back hidden — looking, to whoever pressed the button, like it
            // never came back at all.
            let came_back = session.config.found_version.as_deref() == Some(HERE);
            app.manage(Mutex::new(session));
            app.manage(herald::Speaking::new(app.handle(), telling, &quiet));
            herald::watch(app.handle().clone(), watched);

            app.manage(Stopping::default());
            let perched = tray::raise(app.handle(), &words).is_some();
            app.manage(Perched(perched));
            app.manage(Bound(listen_for(app.handle())));

            {
                let held = app.state::<Mutex<Session>>();
                let held = crate::held(&held);
                let seen = app.asset_protocol_scope();
                // Its attachments and no more of it: the rest of that folder is not ours to read.
                let shared = match &held.config.sync {
                    Some(tisty_core::config::Sync::Folder(dest)) => vec![dest.join("attachments")],
                    _ => Vec::new(),
                };
                for at in [held.paths.attachments(), held.paths.docs()]
                    .into_iter()
                    .chain(shared)
                {
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
                if came_back || !waking::hushed() {
                    let _ = window.show();
                }
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
        .manage(Updating::default())
        .manage(Leaving::default())
        .invoke_handler(tauri::generate_handler![
            snapshot,
            task_story,
            task_series,
            task_left,
            routines,
            agent,
            agent_turn,
            archive_shape,
            close_window,
            shortcut,
            settle_in,
            reachable,
            reach_for,
            free_up,
            stop_freeing,
            wiring,
            wire,
            unwire,
            waking,
            wake_for,
            keep_locale,
            keep_closing,
            erase,
            guide,
            capture,
            read,
            search,
            complete,
            owed,
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
            attached,
            attach_export,
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
            take_over,
            merge_stores,
            sync_kin,
            remove_machine,
            retire_attachment,
            settle_paper,
            paper_rifts,
            weave_paper,
            convert_paper,
            checked,
            twinned,
            rebuild,
            about,
            settings,
            keep_settings,
            facts,
            keep_report,
            note_trouble,
            note_break,
            update_ready,
            update_install,
            logs,
            icons,
            families,
            list_add,
            list_look,
            list_rename,
            list_drop,
            docs,
            folder_add,
            folder_rename,
            folder_look,
            folder_drop,
            doc_file,
            doc_read,
            doc_facts,
            keep_pdf,
            doc_write,
            doc_new,
            doc_page,
            doc_drop,
            doc_import,
            doc_export,
            doc_copy,
            doc_away,
            parted,
            sow,
            printed,
            folder_file
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            if matches!(_event, tauri::RunEvent::Reopen { .. }) {
                tray::surface(_app);
            }
        });
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_file_icloud_took_away_is_not_read_as_one_that_was_lost() {
        use super::{Sought, found_in};

        let here = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let reference = "attachments/cd/charla-e5f6a7b8.mp4";
        let shelf = shared.path().join("attachments/cd");
        std::fs::create_dir_all(&shelf).unwrap();
        std::fs::write(shelf.join(".charla-e5f6a7b8.mp4.icloud"), b"a few bytes").unwrap();

        // Off a Mac nothing can be asked back, but it is still told apart from what is gone.
        let told = found_in(reference, here.path(), Some(shared.path()));
        match cfg!(target_os = "macos") {
            true => assert!(matches!(told, Sought::Coming | Sought::At(_))),
            false => assert!(matches!(told, Sought::Away), "nobody here to ask iCloud"),
        }
        assert!(matches!(
            found_in(
                "attachments/ab/nope-00000000.txt",
                here.path(),
                Some(shared.path())
            ),
            Sought::No
        ));
    }

    #[test]
    fn an_attachment_is_looked_for_here_first_and_then_where_it_is_shared() {
        use super::{Sought, found_in};

        let here = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let from = tempfile::tempdir().unwrap();
        let kept = |root: &std::path::Path, called: &str, body: &[u8]| {
            let at = from.path().join(called);
            std::fs::write(&at, body).unwrap();
            tisty_core::attach::keep(&at, root, tisty_core::attach::COPIED_UP_TO)
                .unwrap()
                .at
        };
        let mine = kept(here.path(), "nota.txt", b"lo apuntado");
        let theirs = kept(shared.path(), "charla.mp4", b"lo grabado");
        let mine = mine.as_str();
        let theirs = theirs.as_str();

        assert!(matches!(
            found_in(mine, here.path(), Some(shared.path())),
            Sought::At(_)
        ));
        assert!(
            matches!(
                found_in(theirs, here.path(), Some(shared.path())),
                Sought::At(_)
            ),
            "what only the shared folder holds is still reachable"
        );
        assert!(
            matches!(found_in(theirs, here.path(), None), Sought::No),
            "without a shared folder there is nowhere else to look"
        );
        assert!(matches!(
            found_in("attachments/ab/nope-00000000.txt", here.path(), None),
            Sought::No
        ));
        let lying = shared.path().join(theirs);
        std::fs::write(&lying, b"other bytes entirely").unwrap();
        assert!(
            matches!(
                found_in(theirs, here.path(), Some(shared.path())),
                Sought::Torn
            ),
            "what does not answer for its own name is not handed over"
        );
        assert!(
            matches!(
                found_in("../outside.txt", here.path(), Some(shared.path())),
                Sought::No
            ),
            "the way out is still shut"
        );
    }

    #[test]
    fn a_body_that_changed_underneath_is_the_only_one_held_back() {
        use super::stale;

        assert!(stale(Some("aa"), Some("bb")), "somebody wrote in it");
        assert!(!stale(Some("aa"), Some("aa")), "it is as it was read");
        assert!(!stale(None, Some("bb")), "this window never read it");
        assert!(!stale(Some("aa"), None), "it is not there to compare");
    }

    #[test]
    fn a_folder_name_stops_where_the_agent_and_the_core_stop() {
        use super::named_folder;
        let most = tisty_core::model::FOLDER_NAME_AT_MOST;

        assert_eq!(named_folder("  Condominio  ").unwrap(), "Condominio");
        assert_eq!(
            named_folder(&"á".repeat(most)).unwrap().chars().count(),
            most
        );
        assert_eq!(
            named_folder(&"a".repeat(most + 1)).unwrap_err().code,
            "folderNameTooLong"
        );
        assert_eq!(named_folder("   ").unwrap_err().code, "untitled");
    }

    #[test]
    fn a_view_can_ask_for_several_lists_at_once() {
        let a = ulid::Ulid::generate();
        let b = ulid::Ulid::generate();
        let view = View {
            lists: vec![a.to_string(), b.to_string()],
            ..bare()
        };

        assert_eq!(view.resolve().unwrap().lists, vec![a, b]);
    }

    #[test]
    fn the_list_being_read_joins_the_ones_being_filtered() {
        let open = ulid::Ulid::generate();
        let also = ulid::Ulid::generate();
        let view = View {
            list: Some(open.to_string()),
            lists: vec![also.to_string()],
            ..bare()
        };

        assert_eq!(view.resolve().unwrap().lists, vec![open, also]);
    }

    #[test]
    fn a_list_that_is_not_an_id_is_refused_rather_than_ignored() {
        let view = View {
            lists: vec!["not an id".into()],
            ..bare()
        };

        assert!(view.resolve().is_err());
    }

    fn bare() -> View {
        View {
            archive: false,
            everything: false,
            inbox: false,
            list: None,
            lists: Vec::new(),
            tags: Vec::new(),
            tagged: false,
            hidden: false,
            window: None,
            repeating: false,
            reading: None,
        }
    }

    #[test]
    fn the_folder_the_person_chose_for_syncing_can_be_opened() {
        let shared = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();

        assert!(within(
            shared.path(),
            &[data.path().to_path_buf(), shared.path().to_path_buf()]
        ));
    }

    #[test]
    fn nothing_the_screen_names_reaches_a_folder_we_were_not_given() {
        let shared = tempfile::tempdir().unwrap();
        let elsewhere = tempfile::tempdir().unwrap();

        assert!(!within(elsewhere.path(), &[shared.path().to_path_buf()]));
    }

    #[test]
    fn a_neighbour_whose_name_merely_begins_the_same_is_not_inside() {
        let dir = tempfile::tempdir().unwrap();
        let ours = dir.path().join("drive");
        let theirs = dir.path().join("drive-private");
        std::fs::create_dir_all(&ours).unwrap();
        std::fs::create_dir_all(&theirs).unwrap();

        assert!(!within(&theirs, &[ours]));
    }

    #[test]
    fn the_other_version_lands_in_the_same_folder_as_the_one_it_came_from() {
        let folder = ulid::Ulid::generate();

        let (where_at, _, order) = placed(Some((Some(folder), None, "a0".into())), "dev_a-0009");

        assert_eq!(where_at, Some(folder));
        assert!(order.as_str() > "a0", "no quedo despues del original");
    }

    #[test]
    fn the_other_version_stays_loose_only_when_the_original_is_loose() {
        let (where_at, _, order) = placed(Some((None, None, "a0".into())), "dev_a-0009");

        assert_eq!(where_at, None);
        assert!(order.as_str() > "a0");
    }

    #[test]
    fn an_original_nobody_can_find_does_not_stop_the_other_version_from_landing() {
        let (where_at, _, order) = placed(None, "dev_a-0009");

        assert_eq!(where_at, None);
        assert_eq!(order, "dev_a-0009");
    }

    #[test]
    fn the_other_version_of_a_page_is_a_page_of_the_same_document() {
        let folder = ulid::Ulid::generate();
        let up = ulid::Ulid::generate();

        let (where_at, page_of, _) =
            placed(Some((Some(folder), Some(up), "a0".into())), "dev_a-0009");

        assert_eq!(
            page_of,
            Some(up),
            "it came back as a document beside the book"
        );
        assert_eq!(where_at, Some(folder));
    }
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
            tisty_nl::parse("comprar pan mañana #casa !hacer", &now(), "es").into();
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

    #[test]
    fn the_guide_travels_inside_the_binary_rather_than_beside_it() {
        assert!(GUIDE_ES.starts_with("# "), "la guia en espanol no viaja");
        assert!(GUIDE_EN.starts_with("# "), "la guia en ingles no viaja");
    }

    #[test]
    fn every_picture_the_guide_names_is_where_the_bundler_looks() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/guide");
        for (told, tongue) in [(GUIDE_ES, "es"), (GUIDE_EN, "en")] {
            for shot in PICTURES {
                if !told.contains(&format!("]({shot})")) {
                    continue;
                }
                assert!(root.join(tongue).join(shot).is_file(), "falta {shot}");
            }
        }
    }
}
