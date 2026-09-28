use std::sync::Mutex;

mod answers;
mod command;
mod finding;
mod glimpse;
mod herald;
mod refusing;
mod report;
mod session;
mod shop;
mod summing;
mod tray;
mod update;
mod vouching;
mod waking;
mod wiring;

use tauri::{Emitter, Manager};

pub(crate) use refusing::{Refusal, blamed};
pub(crate) use refusing::{behind_buttons, behind_said, where_it_comes_from};
pub(crate) use session::Session;
pub(crate) use summing::{Coming, Counted, Habit, coming, recurring, tags_in_use, tally};

use tisty_core::{
    List, Op, Paths, Reading, State, Tag, Task,
    event::TaskPatch,
    view::{Filter, Scope, Window},
    witness::{self, Fact, channel},
};

#[derive(serde::Serialize)]
struct Snapshot {
    tasks: Vec<Task>,
    ahead: Vec<Coming>,
    routines: Vec<Habit>,
    lists: Vec<List>,
    tags: Vec<Counted>,
    refs: Vec<String>,
    counts: std::collections::BTreeMap<String, usize>,
    locale: Option<String>,
    agents: std::collections::BTreeMap<String, String>,
    agent_tag: &'static str,
    hosts: std::collections::BTreeMap<String, String>,
    machines: std::collections::BTreeMap<String, String>,
    machine_here: String,
    clients: std::collections::BTreeMap<String, String>,
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

impl answers::tasks::Edits {
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
            draft.date = Some(answers::tasks::dated(raw, now, spoken)?);
        }
        if let Some(raw) = &self.deadline {
            draft.deadline = Some(answers::tasks::dated(raw, now, spoken)?);
        }
        if let Some(name) = &self.priority {
            draft.priority = Some(answers::tasks::named_priority(name)?);
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
        let one = Tag::written(name).map_err(|_| Refusal::about("badTag", name))?;
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
        (Some(raw), _) => Ok(Some(Some(answers::tasks::dated(raw, now, spoken)?))),
        (None, true) => Ok(Some(None)),
        _ => Ok(None),
    }
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
            Scope::Open => !session.state.held_away(one),
            Scope::Archived => session.state.held_away(one),
            Scope::Either => true,
        })
        .map(|one| (one.file.clone(), session.state.held_away(one)))
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Carrying {
    chosen: Option<String>,
    keeper: Option<String>,
    kept_by: Option<String>,
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
    let mut session = held(&session);
    let _ = session.reload();
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
    let alive: Vec<String> = session
        .state
        .docs
        .values()
        .map(|one| one.file.clone())
        .collect();

    Ok(Reviewed {
        tasks: session.state.tasks.len(),
        lists: session.state.lists.len(),
        agrees: matches!(audit, tisty_core::cache::Audit::Agrees { .. }),
        loose: adrift.files(),
        loose_bytes: adrift.bytes,
        astray: adrift.items,
        stranded: tisty_core::docs::strayed(&session.paths.docs(), &alive),
        missing: tisty_core::docs::missing(&session.paths.docs(), &alive)
            .into_iter()
            .filter_map(|file| {
                let kept = session.state.docs.values().find(|one| one.file == file)?;
                let title = tisty_core::docs::read_before(session.paths.data(), &file)
                    .map(|was| tisty_core::docs::titled(&was))
                    .unwrap_or_default();
                Some(Gone {
                    file,
                    title,
                    id: kept.id.to_string(),
                })
            })
            .collect(),
        events: told.len(),
        machines: report::machines(
            &told,
            session.config.device_id.0.as_str(),
            &session.state.dropped,
            &session.state.assistants,
        ),
        log_bytes: report::weighed(&session.paths.store()),
        docs_bytes: report::weighed(&session.paths.docs()),
        held_bytes: kept.bytes,
        held_files: kept.files,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Gone {
    id: String,
    file: String,
    title: String,
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
    stranded: Vec<tisty_core::docs::Stray>,
    missing: Vec<Gone>,
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
    "folderAway",
    "folderAwayHolds",
    "folderIsAway",
    "notAParcel",
    "parcelNewer",
    "parcelLocked",
    "wrongNumber",
    "parcelTorn",
    "noRoom",
    "nothingToCarry",
    "stillPacking",
    "aliasTooLong",
    "tooBig",
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

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Logs {
    at: String,
    bytes: u64,
    lines: Vec<String>,
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

const HERE: &str = env!("CARGO_PKG_VERSION");

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

/// A document only learns its tags where its body is read, and a body is read when it is saved.
/// Everything written before this existed is caught up here, in one pass over the folder.
#[tauri::command(async)]
fn read_tags(session: tauri::State<'_, Mutex<Session>>) -> Answer<usize> {
    let root = held(&session).paths.docs();
    let owing: Vec<(tisty_core::model::DocId, String)> = held(&session)
        .state
        .docs
        .values()
        .map(|one| (one.id, one.file.clone()))
        .collect();

    let mut ops: Vec<Op> = Vec::new();
    for (id, file) in owing {
        let Ok(body) = tisty_core::docs::read(&root, &file) else {
            continue;
        };
        let session = held(&session);
        let said = tisty_core::event::Said::of(&body).by(None);
        let Some(kept) = session.state.docs.get(&id) else {
            continue;
        };
        if said.news_for(kept) {
            ops.push(Op::DocSaid { id, d: said });
        }
    }

    let told = ops.len();
    if !ops.is_empty() {
        held(&session).commit_all(ops)?;
    }
    Ok(told)
}

fn stop(found: Option<Option<usize>>) -> Option<usize> {
    found.flatten()
}

fn signing(state: &tisty_core::State) -> Option<String> {
    state.signed.alias.clone()
}

// The MSIX package ships the executable alone, so the guide travels inside the binary.
fn stale(mine: Option<&str>, now: Option<&str>) -> bool {
    matches!((mine, now), (Some(mine), Some(now)) if mine != now)
}

/// Changing a folder the archive holds, or filing anything into it, is refused everywhere.
pub(crate) fn folder_open(
    state: &State,
    at: tisty_core::model::FolderId,
    holds: bool,
) -> Answer<()> {
    match state.folder_away(at) {
        true => Err(Refusal::of(match holds {
            true => "folderAwayHolds",
            false => "folderAway",
        })),
        false => Ok(()),
    }
}

fn doc_out(state: &State, id: tisty_core::model::DocId) -> Answer<()> {
    let Some(kept) = state.docs.get(&id) else {
        return Ok(());
    };
    match (state.held_by_another(kept), kept.page_of.is_some()) {
        (true, true) => Err(Refusal::of("pageIsAway")),
        (true, false) => Err(Refusal::of("folderIsAway")),
        (false, _) => Ok(()),
    }
}

#[tauri::command(async)]
fn doc_copy(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::docs::Doc> {
    held(&session).copy_doc(&id)
}

#[tauri::command(async)]
fn doc_export(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    into: String,
) -> Answer<Taken> {
    let mut session = held(&session);
    if let Ok(body) = tisty_core::docs::read(&session.paths.docs(), &id) {
        let _ = session.retell(&id, &body, None);
    }
    let said = tisty_core::docs::read(&session.paths.docs(), &id).unwrap_or_default();
    let pages: Vec<String> = session
        .state
        .docs
        .values()
        .find(|one| one.file == id)
        .map(|one| {
            session
                .state
                .pages_read(one.id, &said)
                .iter()
                .map(|page| page.file.clone())
                .collect()
        })
        .unwrap_or_default();
    let beside = session.dest();
    tisty_core::docs::with_pages(
        session.paths.data(),
        &id,
        &pages,
        std::path::Path::new(&into),
        beside.as_deref(),
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
    .map(|took| {
        if !took.left.is_empty() {
            witness::warn(
                channel::WINDOW,
                "a document went out without everything it points at",
                &[
                    ("id", Fact::Id(id.clone())),
                    ("left", Fact::Why(took.left.join("; "))),
                ],
            );
        }
        Taken {
            files: took.files,
            missed: took.missed,
            left: took.left.len(),
        }
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Taken {
    files: usize,
    missed: usize,
    left: usize,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Afoot {
    stage: &'static str,
    far: u64,
    done: usize,
    whole: usize,
}

fn along_the_way(
    app: &tauri::AppHandle,
    stage: &'static str,
) -> impl Fn(tisty_core::parcel::Step) + use<> {
    let app = app.clone();
    let said = std::sync::atomic::AtomicU64::new(u64::MAX);
    move |step| {
        let far = match step.whole {
            0 => 0,
            whole => (step.done as u64 * 100 / whole as u64).min(100),
        };
        if said.swap(far, std::sync::atomic::Ordering::Relaxed) == far {
            return;
        }
        let _ = app.emit(
            "carrying",
            Afoot {
                stage,
                far,
                done: step.done,
                whole: step.whole,
            },
        );
    }
}

fn standing(
    session: &tauri::State<'_, Mutex<Session>>,
    which: &[String],
) -> (Paths, tisty_core::State, Option<std::path::PathBuf>) {
    let mut session = held(session);
    for one in which {
        if let Ok(body) = tisty_core::docs::read(&session.paths.docs(), one) {
            let _ = session.retell(one, &body, None);
        }
    }
    (session.paths.clone(), session.state.clone(), session.dest())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Packed {
    docs: usize,
    pages: usize,
    folders: usize,
    files: usize,
    missed: usize,
    left: usize,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Unpacked {
    docs: usize,
    pages: usize,
    folders: usize,
    joined: usize,
    files: usize,
    missed: usize,
}

#[tauri::command(async)]
async fn docs_pack(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Packing>,
    which: Vec<String>,
    into: String,
    number: Option<String>,
) -> Answer<Packed> {
    let _done = alone.inner().taken()?;
    let (paths, state, beside) = standing(&session, &which);
    let asked = which.clone();
    let at = into.clone();
    let telling = along_the_way(&app, "packing");
    let locking = along_the_way(&app, "locking");
    let sent = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::parcel::written(
            &paths,
            &state,
            &asked,
            std::path::Path::new(&at),
            &tisty_core::parcel::Along {
                also: beside.as_deref(),
                say: Some(&telling),
                then: Some(&locking),
            },
            number.as_deref(),
        )
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::warn(
            channel::WINDOW,
            "a parcel of documents could not be written",
            &[("why", Fact::Why(e.to_string()))],
        );
        match e {
            tisty_core::Error::TooBig => Refusal::of("tooBig"),
            tisty_core::Error::NothingToCarry => Refusal::of("nothingToCarry"),
            _ => Refusal::about("cannotWrite", into.clone()),
        }
    })?;

    if !sent.left.is_empty() {
        witness::warn(
            channel::WINDOW,
            "a parcel went out without everything it points at",
            &[("left", Fact::Why(sent.left.join("; ")))],
        );
    }
    Ok(Packed {
        docs: sent.docs,
        pages: sent.pages,
        folders: sent.folders,
        files: sent.files,
        missed: sent.missed,
        left: sent.left.len(),
    })
}

#[tauri::command(async)]
async fn docs_take_out(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Packing>,
    which: Vec<String>,
    into: String,
) -> Answer<Packed> {
    let _done = alone.inner().taken()?;
    let (paths, state, beside) = standing(&session, &which);
    let asked = which.clone();
    let at = into.clone();
    let telling = along_the_way(&app, "takingOut");
    let sent = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::parcel::plainly(
            paths.data(),
            &state,
            &asked,
            std::path::Path::new(&at),
            &tisty_core::parcel::Along {
                also: beside.as_deref(),
                say: Some(&telling),
                then: None,
            },
        )
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| {
        witness::warn(
            channel::WINDOW,
            "the documents could not be taken out",
            &[("why", Fact::Why(e.to_string()))],
        );
        match e {
            tisty_core::Error::NothingToCarry => Refusal::of("nothingToCarry"),
            _ => Refusal::about("cannotWrite", into.clone()),
        }
    })?;

    if !sent.left.is_empty() {
        witness::warn(
            channel::WINDOW,
            "documents went out without everything they point at",
            &[("left", Fact::Why(sent.left.join("; ")))],
        );
    }
    Ok(Packed {
        docs: sent.docs,
        pages: sent.pages,
        folders: sent.folders,
        files: sent.files,
        missed: sent.missed,
        left: sent.left.len(),
    })
}

#[tauri::command(async)]
async fn docs_unpack(
    app: tauri::AppHandle,
    session: tauri::State<'_, Mutex<Session>>,
    alone: tauri::State<'_, Packing>,
    from: String,
    number: Option<String>,
) -> Answer<Unpacked> {
    let _done = alone.inner().taken()?;
    let (paths, state, device) = {
        let session = held(&session);
        (
            session.paths.clone(),
            session.state.clone(),
            session.config.device_id.clone(),
        )
    };
    let at = from.clone();
    let telling = along_the_way(&app, "landing");
    let opening = along_the_way(&app, "opening");
    let (landed, ops) = tauri::async_runtime::spawn_blocking(move || {
        tisty_core::parcel::taken(
            &paths,
            &state,
            &device,
            std::path::Path::new(&at),
            &tisty_core::parcel::Along {
                also: None,
                say: Some(&telling),
                then: Some(&opening),
            },
            number.as_deref(),
        )
    })
    .await
    .map_err(|_| Refusal::of("internal"))?
    .map_err(|e| match e {
        tisty_core::Error::NotAParcel(_) => Refusal::about("notAParcel", from.clone()),
        tisty_core::Error::ParcelNewer(_) => Refusal::of("parcelNewer"),
        tisty_core::Error::ParcelLocked => Refusal::of("parcelLocked"),
        tisty_core::Error::WrongNumber => Refusal::of("wrongNumber"),
        tisty_core::Error::ParcelTorn => Refusal::of("parcelTorn"),
        tisty_core::Error::NoRoom { needs, .. } => Refusal::about("noRoom", weighed(needs)),
        tisty_core::Error::TooBig => Refusal::of("tooBig"),
        other => blamed(channel::WINDOW, "a parcel could not be taken in", other),
    })?;

    // What landed is only real once the log says so: if it cannot be written, the bodies go
    // rather than sit in the folder as documents nobody knows about.
    let files: Vec<String> = ops
        .iter()
        .filter_map(|one| match one {
            tisty_core::Op::DocAdd { d, .. } => Some(d.file.clone()),
            _ => None,
        })
        .collect();
    let mut held = held(&session);
    if let Err(e) = held.commit_all(ops) {
        let papers = held.paths.docs();
        for file in files {
            let _ = tisty_core::docs::remove(&papers, &file);
        }
        return Err(blamed(
            channel::WINDOW,
            "a parcel landed but was not written",
            e,
        ));
    }
    drop(held);
    Ok(Unpacked {
        docs: landed.docs,
        pages: landed.pages,
        folders: landed.folders,
        joined: landed.joined,
        files: landed.files,
        missed: landed.missed,
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
    if let Some(at) = folder {
        if !session.state.folders.contains_key(&at) {
            return Err(Refusal::of("noSuchFolder"));
        }
        folder_open(&session.state, at, true)?;
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
    let signed_as = signing(&session.state);
    session.commit(Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            made: None,
            by: signed_as.clone(),
            file: made.id.clone(),
            order,
            said: Some(tisty_core::event::Said {
                title: made.title.clone(),
                bytes: None,
                tags: Some(Vec::new()),
                by: None,
            }),
            folder,
            page_of: None,
        },
    })?;
    Ok(made)
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

/// Whether this process runs translated by Rosetta. Only an Apple Silicon Mac carries the key, so
/// an Intel one answers with an error, which reads as no.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
fn translated() -> bool {
    unsafe extern "C" {
        fn sysctlbyname(
            name: *const std::ffi::c_char,
            oldp: *mut std::ffi::c_void,
            oldlenp: *mut usize,
            newp: *mut std::ffi::c_void,
            newlen: usize,
        ) -> std::ffi::c_int;
    }
    let mut yes: std::ffi::c_int = 0;
    let mut len = size_of::<std::ffi::c_int>();
    let rc = unsafe {
        sysctlbyname(
            c"sysctl.proc_translated".as_ptr(),
            (&raw mut yes).cast(),
            &raw mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    rc == 0 && yes == 1
}

#[cfg(not(target_os = "macos"))]
fn translated() -> bool {
    false
}

#[cfg(not(target_os = "macos"))]
fn proofread(_window: &tauri::WebviewWindow) {}

fn fitted(window: &tauri::WebviewWindow) {
    let (Ok(Some(screen)), Ok(asked)) = (window.current_monitor(), window.outer_size()) else {
        return;
    };
    let room = screen.work_area().size;
    if asked.width > room.width || asked.height > room.height {
        let _ = window.maximize();
    }
}

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
        leave(&handle);
    });
}

/// The window answers in milliseconds and the timer above is only there for the window that
/// never answers, so the second of the two reaches an event loop that is already gone — and
/// tao panics rather than ignore it.
fn leave<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri::Manager;

    if app
        .state::<Departed>()
        .0
        .swap(true, std::sync::atomic::Ordering::SeqCst)
    {
        return;
    }
    app.exit(0);
}

#[derive(Default)]
struct Leaving(std::sync::atomic::AtomicBool);

#[derive(Default)]
struct Departed(std::sync::atomic::AtomicBool);

const SETTLED_IN: jiff::SignedDuration = jiff::SignedDuration::from_hours(24 * 14);
const CLOSED_ENOUGH: usize = 10;
const PAPERS_ENOUGH: usize = 10;

#[derive(Debug, PartialEq, Eq)]
enum Asking {
    Start,
    Wait,
    Now,
}

fn asking(
    asked: Option<bool>,
    since: Option<jiff::Timestamp>,
    now: jiff::Timestamp,
    papers: usize,
    closed: impl FnOnce() -> usize,
) -> Asking {
    if asked.unwrap_or(false) {
        return Asking::Wait;
    }
    let Some(since) = since.filter(|at| *at <= now) else {
        return Asking::Start;
    };
    if now.duration_since(since) < SETTLED_IN {
        return Asking::Wait;
    }
    if papers >= PAPERS_ENOUGH {
        return Asking::Now;
    }
    if closed() < CLOSED_ENOUGH {
        return Asking::Wait;
    }
    Asking::Now
}

fn offering(seen: usize, wired: usize) -> bool {
    seen > 0 && wired == 0
}

#[tauri::command]
fn reachable() -> command::Reach {
    command::reach()
}

#[tauri::command]
fn take_out_of_reach() -> Answer<command::Reach> {
    command::out_of_reach().map_err(|e| Refusal::about("cannotWrite", e.to_string()))?;
    Ok(command::reach())
}

/// A folder that already holds a store is the meeting place itself; anywhere else we hang ours
/// inside, so pointing at Documents does not scatter the store through it.
fn room(at: &std::path::Path) -> std::path::PathBuf {
    if tisty_sync::theirs(at).is_some() || at.join(tisty_sync::STORE).is_dir() {
        return at.to_path_buf();
    }
    tisty_core::keepers::suggested(at)
}

type Placing = (Option<ulid::Ulid>, Option<ulid::Ulid>, String);

fn placed(beside: Option<Placing>, fresh: &str) -> Placing {
    match beside {
        Some((folder, page_of, order)) => (folder, page_of, tisty_core::order::after(&order)),
        None => (None, None, fresh.to_string()),
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
    let (data, shared) = finding::where_to(&session);
    let at = match finding::found_in(&reference, &data, shared.as_deref()) {
        finding::Sought::At(at) => at,
        other => return Err(finding::unreachable(other, reference)),
    };
    std::fs::read(&at).map_err(|_| Refusal::about("cannotRead", reference))
}

#[tauri::command(async)]
fn served(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<String> {
    let (data, shared) = finding::where_to(&session);
    let at = match finding::found_in(&reference, &data, shared.as_deref()) {
        finding::Sought::At(at) => at,
        other => return Err(finding::unreachable(other, reference)),
    };
    Ok(at.to_string_lossy().into_owned())
}

#[tauri::command(async)]
fn attach_export(
    session: tauri::State<'_, Mutex<Session>>,
    reference: String,
    into: String,
) -> Answer<()> {
    let (data, shared) = finding::where_to(&session);
    let from = match finding::found_in(&reference, &data, shared.as_deref()) {
        finding::Sought::At(at) => at,
        other => return Err(finding::unreachable(other, reference)),
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

#[tauri::command(async)]
fn weighs(session: tauri::State<'_, Mutex<Session>>, reference: String) -> Answer<u64> {
    let (data, shared) = finding::where_to(&session);
    let at = finding::where_it_lies(&reference, &data, shared.as_deref())
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
    let (data, shared) = finding::where_to(&session);
    let at = match finding::found_in(&reference, &data, shared.as_deref()) {
        finding::Sought::At(at) => at,
        other => return Err(finding::unreachable(other, reference)),
    };
    if !safe_to_open(&at) {
        return show(&at, &reference);
    }
    handed(&at).map_err(|_| Refusal::about("cannotOpen", reference))?;
    let _ = app;
    Ok(())
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

// Erasing has no undo, so it is judged against the store as it is now — an agent or the
// terminal may have written since the window last looked.
fn erasing(session: &mut Session, id: tisty_core::TaskId) -> Answer<()> {
    session.reload()?;
    if !session.state.tasks.contains_key(&id) {
        return Err(Refusal::of("notATaskId"));
    }
    match session.state.erasable(id) {
        Ok(()) => {}
        Err(tisty_core::model::Stays::Open) => return Err(Refusal::of("onlyArchivedGoes")),
        Err(tisty_core::model::Stays::Story) => return Err(Refusal::of("storyStays")),
        Err(tisty_core::model::Stays::Routine) => return Err(Refusal::of("routineStays")),
    }
    session.commit(Op::TaskDelete { id })?;
    Ok(())
}

/// Only an open task the person wrote takes the permission: a closed one is history to an
/// assistant, and what an agent filed is the agents' already.
fn opening_to_agents(session: &mut Session, id: tisty_core::TaskId, open: bool) -> Answer<Task> {
    session.reload()?;
    let task = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    if !task.is_open() {
        return Err(Refusal::of("onlyOpenOpens"));
    }
    if session.state.filed_by_agents(task) {
        return Err(Refusal::of("alreadyTheirs"));
    }
    session.commit(Op::TaskUpdate {
        id,
        d: TaskPatch {
            open_to_agents: Some(open),
            ..Default::default()
        },
    })?;
    session
        .state
        .tasks
        .get(&id)
        .cloned()
        .ok_or_else(|| Refusal::of("notATaskId"))
}

fn reading_as(session: &mut Session, id: tisty_core::TaskId, how: Reading) -> Answer<Task> {
    session.reload()?;
    let task = session
        .state
        .tasks
        .get(&id)
        .ok_or_else(|| Refusal::of("notATaskId"))?;
    if task.is_open() {
        return Err(Refusal::of("onlyClosedConverts"));
    }
    if task.reading() == Reading::Routine {
        return Err(Refusal::of("routineReadsAsRoutine"));
    }
    session.commit(Op::TaskUpdate {
        id,
        d: TaskPatch {
            read_as: Some(Some(how)),
            ..Default::default()
        },
    })?;
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
struct Packing(OneAtATime);

impl Packing {
    fn taken(&self) -> Answer<Releasing<'_>> {
        self.0.claim().ok_or_else(|| Refusal::of("stillPacking"))
    }
}

struct Releasing<'a>(&'a std::sync::atomic::AtomicBool);

impl Drop for Releasing<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::Release);
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
    let _ = waking::wake(false);
    let reached = command::out_of_reach();
    if let Ok(paths) = tisty_core::Paths::resolve() {
        for at in paths.swept_on_leaving() {
            let _ = match at.is_dir() {
                true => std::fs::remove_dir_all(&at),
                false => std::fs::remove_file(&at),
            };
        }
    }
    for at in tisty_core::Paths::shims() {
        let _ = std::fs::remove_file(&at);
    }
    reached
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn settled(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let session = match Session::open() {
        Ok(session) => session,
        Err(why) => {
            use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
            witness::error(channel::WINDOW, "the session would not open", &why.told());
            for window in app.webview_windows().values() {
                let _ = window.close();
            }
            let behind = matches!(why, tisty_core::Error::UnsupportedVersion(_));
            let said = app
                .dialog()
                .message(if behind {
                    behind_said()
                } else {
                    why.to_string()
                })
                .kind(MessageDialogKind::Error)
                .title("Tisty");
            if behind {
                let (yes, no) = behind_buttons();
                if said
                    .buttons(MessageDialogButtons::OkCancelCustom(yes.into(), no.into()))
                    .blocking_show()
                {
                    let _ = tauri_plugin_opener::open_url(where_it_comes_from(), None::<&str>);
                }
            } else {
                said.blocking_show();
            }
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
    answers::settings::appearance(app.handle(), session.config.theme);
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
            fitted(&window);
            let _ = window.show();
        }
    }
    Ok(())
}

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
        .setup(settled)
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
                    let _ = window.emit("withdrawn", ());
                    let _ = window.hide();
                }
                None => {
                    api.prevent_close();
                    let _ = window.emit("closing", ());
                }
            }
        })
        .manage(OneAtATime::default())
        .manage(Packing::default())
        .manage(Updating::default())
        .manage(Leaving::default())
        .manage(Departed::default())
        .invoke_handler(tauri::generate_handler![
            answers::tasks::snapshot,
            answers::storing::keepers,
            answers::storing::keeper_of,
            answers::storing::strays_at,
            answers::storing::make_room,
            answers::storing::glimpse_kept,
            answers::storing::glimpse_fetch,
            answers::shelves::sow_lists,
            answers::tasks::task_story,
            answers::tasks::task_series,
            answers::tasks::task_left,
            answers::tasks::routines,
            answers::agents::agent,
            answers::agents::agent_turn,
            answers::tasks::archive_shape,
            answers::settings::close_window,
            answers::settings::shortcut,
            answers::storing::settle_in,
            reachable,
            take_out_of_reach,
            free_up,
            stop_freeing,
            answers::wired::wiring,
            answers::agents::assistants,
            answers::wired::wire,
            answers::wired::unwire,
            answers::wired::waking,
            answers::wired::wake_for,
            answers::settings::keep_locale,
            answers::settings::keep_closing,
            answers::settings::keep_theme,
            answers::tasks::erase,
            answers::papers::read_as,
            answers::tasks::open_to_agents,
            answers::settings::guide,
            answers::tasks::capture,
            answers::tasks::read,
            search,
            answers::attaching::complete,
            answers::attaching::owed,
            answers::tasks::reopen,
            answers::tasks::patch,
            answers::tasks::write_step,
            answers::tasks::mark_step,
            answers::tasks::drop_step,
            answers::tasks::write_log,
            answers::tasks::fold,
            answers::tasks::still_open,
            answers::tasks::discard,
            answers::attaching::attach,
            served,
            attached,
            attach_export,
            weighs,
            answers::storing::roomy,
            opened,
            answers::storing::revealed,
            answers::storing::sync_state,
            answers::storing::choose_sync,
            answers::storing::sync_now,
            answers::storing::back_up,
            answers::storing::restore,
            answers::storing::join_them,
            answers::storing::take_over,
            answers::storing::merge_stores,
            answers::storing::sync_kin,
            answers::storing::joining,
            answers::storing::folder_astir,
            answers::storing::remove_machine,
            answers::storing::retire_attachment,
            answers::papers::settle_paper,
            answers::storing::paper_rifts,
            answers::storing::weave_paper,
            answers::papers::convert_paper,
            checked,
            twinned,
            rebuild,
            answers::settings::about,
            answers::settings::notices,
            answers::settings::settings,
            answers::settings::keep_settings,
            facts,
            keep_report,
            answers::tasks::note_trouble,
            answers::tasks::note_break,
            answers::updating::update_ready,
            answers::updating::update_install,
            answers::updating::update_candidates,
            answers::tasks::star_due,
            answers::tasks::star_done,
            answers::tasks::door_due,
            answers::tasks::door_done,
            answers::tasks::logs,
            answers::settings::icons,
            answers::settings::families,
            answers::shelves::list_add,
            answers::shelves::list_look,
            answers::shelves::list_rename,
            answers::shelves::list_drop,
            answers::papers::docs,
            answers::papers::docs_catch_up,
            read_tags,
            answers::shelves::folder_add,
            answers::shelves::folder_rename,
            answers::shelves::folder_look,
            answers::shelves::folder_drop,
            answers::papers::doc_file,
            answers::papers::doc_read,
            answers::papers::doc_facts,
            answers::papers::keep_pdf,
            answers::papers::doc_back,
            answers::papers::doc_backable,
            answers::papers::doc_write,
            answers::papers::doc_order,
            answers::papers::doc_new,
            answers::papers::doc_page,
            answers::papers::doc_drop,
            doc_import,
            doc_export,
            answers::papers::signed,
            answers::papers::sign,
            answers::papers::sign_the_rest,
            answers::papers::spelled,
            docs_pack,
            docs_take_out,
            docs_unpack,
            doc_copy,
            answers::papers::doc_adopt,
            answers::papers::doc_let_go,
            answers::storing::retire_attachments,
            answers::papers::doc_away,
            answers::papers::doc_unflag,
            answers::shelves::folder_away,
            answers::papers::doc_lock,
            answers::papers::parted,
            answers::shelves::sow,
            answers::shelves::folder_file
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
#[path = "lib_copying.rs"]
mod copying;

#[cfg(test)]
#[path = "lib_hanging_again.rs"]
mod hanging_again;

#[cfg(test)]
#[path = "lib_going_back.rs"]
mod going_back;

#[cfg(test)]
#[path = "lib_deleting.rs"]
mod deleting;

#[cfg(test)]
#[path = "lib_test.rs"]
mod tests;

#[cfg(test)]
#[path = "lib_behind_tests.rs"]
mod behind_tests;

#[cfg(test)]
#[path = "lib_hands_of.rs"]
mod hands_of;

#[cfg(test)]
#[path = "lib_starring.rs"]
mod starring;

#[cfg(test)]
#[path = "lib_doors.rs"]
mod doors;

#[cfg(test)]
#[path = "lib_hosting.rs"]
mod hosting;

#[cfg(test)]
#[path = "lib_letting_go.rs"]
mod letting_go;

#[cfg(test)]
#[path = "lib_ordering.rs"]
mod ordering;
