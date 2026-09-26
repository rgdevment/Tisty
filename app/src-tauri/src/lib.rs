use std::sync::Mutex;

mod answers;
mod command;
mod finding;
mod glimpse;
mod herald;
mod report;
mod shop;
mod tray;
mod update;
mod vouching;
mod waking;
mod wiring;

use tauri::{Emitter, Manager};

use tisty_core::{
    Config, Event, List, Op, Paths, Reading, State, Store, Tag, Task,
    event::TaskPatch,
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

/// What the toolkit says goes where everything else does. Without this its own refusals — an
/// asset it would not serve, a window it could not draw — are written to a logger nobody set up,
/// so they leave no trace at all and the window simply shows nothing.
struct Relayed;

impl log::Log for Relayed {
    fn enabled(&self, said: &log::Metadata<'_>) -> bool {
        said.level() <= log::Level::Warn
    }

    fn log(&self, said: &log::Record<'_>) {
        if !self.enabled(said.metadata()) {
            return;
        }
        let facts = [
            ("from", Fact::Why(said.target().to_string())),
            ("why", Fact::Why(said.args().to_string())),
        ];
        match said.level() {
            log::Level::Error => witness::error(channel::WINDOW, "the toolkit refused", &facts),
            _ => witness::warn(channel::WINDOW, "the toolkit complained", &facts),
        }
    }

    fn flush(&self) {}
}

impl Session {
    fn open() -> tisty_core::Result<Self> {
        let paths = Paths::resolve()?;
        tisty_core::witness::keeps(
            tisty_core::witness::file(&paths),
            tisty_core::witness::wants_all(),
        );
        static RELAY: Relayed = Relayed;
        if log::set_logger(&RELAY).is_ok() {
            log::set_max_level(log::LevelFilter::Warn);
        }
        crate::vouching::vouching_kept_at(paths.cache().join("vouched.json"));
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
        Self::at(paths)
    }

    fn at(paths: Paths) -> tisty_core::Result<Self> {
        let config = Config::load_or_init(&paths)?;
        tisty_core::store::brought_home(&paths);
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
        session.tidy_up(true);
        if let Some(host) = tisty_core::agent::unhosted(&session.config, &session.state)
            && let Err(why) = session.commit(host)
        {
            witness::warn(
                channel::WINDOW,
                "the agent could not say which machine hosts it",
                &[("why", Fact::Why(why.to_string()))],
            );
        }
        if session.config.sync.is_some() {
            session.sow_if_due();
        }
        Ok(session)
    }

    /// A first run holds its lists back until the welcome has said where the copies go: sown, this
    /// store stops looking new, and a folder that already holds another machine could not be
    /// adopted without asking somebody to throw one of the two away.
    fn sow_if_due(&mut self) {
        if self.config.sown == Some(true)
            || !self.state.lists.is_empty()
            || !self.state.tasks.is_empty()
        {
            return;
        }
        let code = tisty_core::model::spoken(self.config.locale.as_deref());
        if let Err(why) = self.commit_all(tisty_core::model::sown(&code)) {
            witness::warn(
                channel::CONFIG,
                "the lists a fresh install starts with were not written",
                &[("why", Fact::Why(why.to_string()))],
            );
            return;
        }
        let _ = self.keep(|config| config.sown = Some(true));
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

    /// `hand` is the alias to seal on the note, and only a write by the person has one: reading
    /// a body back to keep the state honest is not writing into it.
    fn retell(&mut self, file: &str, body: &str, hand: Option<String>) -> bool {
        let mut told = self.state.settling(file, body);
        if let Some(kept) = self.state.docs.values().find(|one| one.file == file) {
            let said = tisty_core::event::Said::of(body).by(hand);
            if said.news_for(kept) {
                told.push(Op::DocSaid {
                    id: kept.id,
                    d: said,
                });
            }
        }
        if told.is_empty() {
            return false;
        }
        if let Err(e) = self.commit_all(told) {
            witness::warn(
                channel::WINDOW,
                "where a document's pages sit could not be settled",
                &[
                    ("file", Fact::Id(file.to_string())),
                    ("why", Fact::Why(e.to_string())),
                ],
            );
            return false;
        }
        true
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

    fn retire(&mut self, references: &[String]) -> Answer<usize> {
        if references.is_empty() {
            return Ok(0);
        }
        let now = jiff::Timestamp::now().as_second();
        let mut held_by: Vec<String> = self
            .state
            .tasks
            .values()
            .flat_map(|task| task.references())
            .map(|one| one.target)
            .collect();
        held_by.extend(tisty_core::docs::referenced(&self.paths.docs()));

        let mut told = Vec::new();
        for reference in references {
            if held_by.iter().any(|one| one == reference) {
                if references.len() == 1 {
                    return Err(Refusal::about("stillReferenced", reference.clone()));
                }
                continue;
            }
            let here = tisty_core::attach::resolve(reference, self.paths.data())
                .is_ok_and(|at| at.is_file());
            if here && let Err(e) = tisty_core::attach::set_aside(self.paths.data(), reference, now)
            {
                witness::warn(
                    channel::ATTACH,
                    "an attachment could not be set aside",
                    &[
                        ("at", Fact::Id(reference.clone())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
                if references.len() == 1 {
                    return Err(Refusal::about("cannotWrite", reference.clone()));
                }
                continue;
            }
            told.push(Op::AttachRetire {
                d: reference.clone(),
            });
        }
        if told.is_empty() {
            return Ok(0);
        }
        let many = told.len();
        self.commit_all(told)
            .map_err(|e| blamed(channel::ATTACH, "the retirement could not be written", e))?;
        self.tidy_up(false);
        self.reproject().map_err(|e| {
            blamed(
                channel::CACHE,
                "the store would not project after retiring",
                e,
            )
        })?;
        Ok(many)
    }

    fn take_in(&mut self, file: &str) -> Answer<tisty_core::docs::Doc> {
        let _ = self.reload();
        if self.state.docs.values().any(|one| one.file == file) {
            return Err(Refusal::of("alreadyKept"));
        }
        if self.state.shed.contains(file) {
            return Err(Refusal::of("shedAlready"));
        }
        let body = tisty_core::docs::read(&self.paths.docs(), file)
            .map_err(|_| Refusal::of("noSuchDoc"))?;

        let order = tisty_core::order::last_of(
            self.state
                .docs
                .values()
                .filter(|one| one.page_of.is_none() && one.folder.is_none())
                .map(|one| one.order.as_str()),
        );
        self.commit(Op::DocAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: signing(&self.state),
                file: file.to_string(),
                order,
                said: Some(tisty_core::event::Said::of(&body)),
                folder: None,
                page_of: None,
            },
        })?;
        Ok(tisty_core::docs::Doc {
            title: tisty_core::docs::titled(&body),
            id: file.to_string(),
        })
    }

    fn let_go_of(&mut self, file: &str) -> Answer<()> {
        let _ = self.reload();
        if self.state.docs.values().any(|one| one.file == file) {
            return Err(Refusal::of("stillKept"));
        }
        tisty_core::docs::remove(&self.paths.docs(), file)
            .map_err(|e| blamed(channel::WINDOW, "a stray document file would not go", e))?;
        tisty_core::docs::forget_carried(self.paths.data(), file);
        self.corpus.forget(file);
        Ok(())
    }

    fn copy_doc(&mut self, id: &str) -> Answer<tisty_core::docs::Doc> {
        let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
        let kept = self
            .state
            .docs
            .get(&id)
            .cloned()
            .ok_or_else(|| Refusal::of("noSuchDoc"))?;
        if kept.page_of.is_some_and(|up| self.state.shut(up)) {
            return Err(Refusal::of("pageOfLocked"));
        }
        if kept
            .page_of
            .and_then(|up| self.state.docs.get(&up))
            .is_some_and(|up| self.state.held_away(up))
        {
            return Err(Refusal::of("pageOfAway"));
        }
        if let Some(at) = kept.folder {
            folder_open(&self.state, at, true)?;
        }

        let root = self.paths.docs();
        let body =
            tisty_core::docs::read(&root, &kept.file).map_err(|_| Refusal::of("noSuchDoc"))?;
        let body = match body.split_once('\n') {
            Some((first, rest)) if !first.trim().is_empty() => {
                format!("{first}{}\n{rest}", worded(&self.locale, "copy"))
            }
            _ if !body.trim().is_empty() => format!("{body}{}", worded(&self.locale, "copy")),
            _ => body,
        };
        let made = tisty_core::docs::create(&root, &self.config.device_id, &body)
            .map_err(|e| blamed(channel::WINDOW, "a document could not be copied", e))?;

        let order = tisty_core::order::last_of(
            self.state
                .docs
                .values()
                .filter(|one| {
                    one.page_of == kept.page_of
                        && (kept.page_of.is_some() || one.folder == kept.folder)
                })
                .map(|one| one.order.as_str()),
        );
        let twin = ulid::Ulid::generate();
        let signed_as = signing(&self.state);
        self.commit(Op::DocAdd {
            id: twin,
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
                    tags: Some(kept.tags.clone()),
                    by: None,
                }),
                folder: kept.folder,
                page_of: kept.page_of,
            },
        })?;
        if kept.archived {
            self.commit(Op::DocArchive { id: twin })?;
        }

        let mut renamed: Vec<(String, String)> = Vec::new();
        for (page, was_away) in self
            .state
            .pages_of(id)
            .iter()
            .map(|one| (one.file.clone(), one.archived))
            .collect::<Vec<_>>()
        {
            let body = tisty_core::docs::read(&root, &page).unwrap_or_default();
            let leaf = tisty_core::docs::create(&root, &self.config.device_id, &body)
                .map_err(|e| blamed(channel::WINDOW, "a page could not be copied", e))?;
            renamed.push((page.clone(), leaf.id.clone()));
            let order = tisty_core::order::last_of(
                self.state
                    .docs
                    .values()
                    .filter(|one| one.page_of == Some(twin))
                    .map(|one| one.order.as_str()),
            );
            let signed_as = signing(&self.state);
            let leaf_id = ulid::Ulid::generate();
            self.commit(Op::DocAdd {
                id: leaf_id,
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: signed_as.clone(),
                    file: leaf.id,
                    order,
                    said: Some(tisty_core::event::Said {
                        title: leaf.title,
                        bytes: None,
                        tags: Some(Vec::new()),
                        by: None,
                    }),
                    folder: kept.folder,
                    page_of: Some(twin),
                },
            })?;
            if was_away {
                self.commit(Op::DocArchive { id: leaf_id })?;
            }
        }

        if !renamed.is_empty() {
            let mine = |text: String| {
                renamed.iter().fold(text, |text, (was, now)| {
                    text.replace(
                        &format!("{}{was}", tisty_core::refs::DOC),
                        &format!("{}{now}", tisty_core::refs::DOC),
                    )
                })
            };
            // A copy whose links were not rewritten points at what it was copied from, which reads
            // as if the pages belonged to the other one.
            let mut astray = Vec::new();
            if let Err(why) = tisty_core::docs::write(&root, &made.id, &mine(body)) {
                astray.push(why.to_string());
            }
            for (_, now) in &renamed {
                let Ok(body) = tisty_core::docs::read(&root, now) else {
                    continue;
                };
                let told = mine(body.clone());
                if told != body
                    && let Err(why) = tisty_core::docs::write(&root, now, &told)
                {
                    astray.push(why.to_string());
                }
            }
            if !astray.is_empty() {
                witness::warn(
                    channel::WINDOW,
                    "a copy was made but some of its links still point at the original",
                    &[
                        ("doc", Fact::Id(made.id.clone())),
                        ("why", Fact::Why(astray.join("; "))),
                    ],
                );
            }
        }
        Ok(made)
    }

    fn drop_doc(&mut self, id: &str) -> Answer<()> {
        let id = id.parse().map_err(|_| Refusal::of("noSuchDoc"))?;
        let kept = self
            .state
            .docs
            .get(&id)
            .ok_or_else(|| Refusal::of("noSuchDoc"))?;
        if self.state.shut(id) {
            return Err(Refusal::of("documentLocked"));
        }
        // Deleting has no undo, and the archive is meant to keep what it holds.
        if self.state.held_by_another(kept) {
            return Err(Refusal::of(match kept.page_of.is_some() {
                true => "pageIsAway",
                false => "folderIsAway",
            }));
        }
        let mut files = vec![kept.file.clone()];
        files.extend(self.state.pages_of(id).iter().map(|one| one.file.clone()));
        self.commit(Op::DocDelete { id })?;
        if self.state.docs.contains_key(&id) {
            return Err(Refusal::of("deleteRefused"));
        }

        let root = self.paths.docs();
        let mut said = tisty_core::docs::Carried::read(self.paths.data());
        for file in &files {
            if let Err(e) = tisty_core::docs::remove(&root, file) {
                witness::warn(
                    channel::WINDOW,
                    "a deleted document left its file behind, and it is swept at the next opening",
                    &[
                        ("file", Fact::Id(file.clone())),
                        ("why", Fact::Why(e.to_string())),
                    ],
                );
            }
            if let Some(tisty_core::config::Sync::Folder(dest)) = self.config.sync.clone() {
                tisty_sync::forget_paper(&dest, file);
            }
            said.forget(file);
            tisty_core::docs::forget_carried(self.paths.data(), file);
        }
        let _ = said.save(self.paths.data());
        Ok(())
    }

    fn unhang(
        &mut self,
        id: tisty_core::model::DocId,
    ) -> tisty_core::Result<tisty_core::event::Filed> {
        self.log()?;
        let told = &self.log.as_ref().expect("just read").1;
        Ok(tisty_core::undo::unhung(told, &self.state, id))
    }

    fn settle_what_arrived(&mut self, files: &[String]) {
        let told = tisty_core::tidy::settling_what_arrived(&self.paths, &self.state, files);
        if told.is_empty() {
            return;
        }
        let many = told.len();
        if let Err(e) = self.commit_all(told) {
            witness::warn(
                channel::SYNC,
                "where a document's pages sit could not be settled",
                &[("why", Fact::Why(e.to_string()))],
            );
            return;
        }
        witness::note(
            channel::SYNC,
            "a document that arrived names its pages in another order, and now the log agrees",
            &[("count", Fact::Count(many))],
        );
    }

    fn dest(&self) -> Option<std::path::PathBuf> {
        match self.config.sync.clone() {
            Some(tisty_core::config::Sync::Folder(at)) => Some(at),
            _ => None,
        }
    }

    fn tidy_up(&mut self, bin: bool) {
        tisty_core::parcel::swept(self.paths.data());
        tisty_core::attach::swept(self.paths.data());
        let dest = self.dest();
        tisty_core::tidy::all_of_it(
            &self.paths,
            &self.state,
            self.cache.as_ref(),
            dest.as_deref(),
            bin,
        );
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
    // The empty trace layer says where the trace went: hidden traces, not every hidden task.
    count(
        "tracesHidden",
        Filter {
            scope: Scope::Archived,
            hidden: true,
            reading: Some(Reading::Trace),
            ..Default::default()
        },
    );
    count(
        "overdue",
        Filter {
            window: Some(Window::Overdue),
            ..Default::default()
        },
    );
    count(
        "dueToday",
        Filter {
            window: Some(Window::On(today())),
            ..Default::default()
        },
    );
    count(
        "undated",
        Filter {
            window: Some(Window::Undated),
            ..Default::default()
        },
    );
    count(
        "inbox",
        Filter {
            inbox: true,
            ..Default::default()
        },
    );
    for (key, wanted) in [
        ("do", tisty_core::model::Priority::Do),
        ("decide", tisty_core::model::Priority::Decide),
        ("delegate", tisty_core::model::Priority::Delegate),
        ("minor", tisty_core::model::Priority::Minor),
    ] {
        count(
            key,
            Filter {
                priority: Some(wanted),
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
    docs: usize,
}

fn tags_in_use(state: &State) -> Vec<Counted> {
    state
        .tags()
        .into_iter()
        .map(|tag| Counted {
            tag: tag.to_string(),
            tasks: state.tasks_tagged(tag).filter(|t| !t.hidden).count(),
            docs: state.docs_tagged(tag).count(),
        })
        .collect()
}

#[derive(serde::Serialize)]
struct Coming {
    task: Task,
    on: jiff::civil::Date,
    due: bool,
}

const AHEAD: i64 = 7;
const BEADS: usize = 5;

fn horizon(from: jiff::civil::Date) -> Option<jiff::civil::Date> {
    jiff::Span::new()
        .try_days(AHEAD)
        .ok()
        .and_then(|span| from.checked_add(span).ok())
}

fn coming(state: &State, from: jiff::civil::Date) -> Vec<Coming> {
    let Some(until) = horizon(from) else {
        return Vec::new();
    };

    let within = |on: jiff::civil::Date| on > from && on <= until;
    let mut out: Vec<Coming> = state
        .matching(&Filter::default(), from)
        .into_iter()
        .filter(|task| task.repeat.is_none())
        .flat_map(|task| {
            let held = task
                .date
                .as_ref()
                .map(|d| d.date())
                .filter(|on| within(*on))
                .map(|on| Coming {
                    task: task.clone(),
                    on,
                    due: false,
                });
            let own = task.date.as_ref().map(|d| d.date());
            let owed = task
                .deadline
                .as_ref()
                .map(|d| d.date())
                .filter(|on| within(*on) && Some(*on) != own)
                .map(|on| Coming {
                    task: task.clone(),
                    on,
                    due: true,
                });
            held.into_iter().chain(owed)
        })
        .collect();
    out.sort_by_key(|one| one.on);
    out
}

#[derive(serde::Serialize)]
struct Habit {
    task: Task,
    #[serde(skip_serializing_if = "Option::is_none")]
    series: Option<tisty_core::series::Series>,
    #[serde(skip_serializing_if = "Option::is_none")]
    on: Option<jiff::civil::Date>,
}

fn recurring(state: &State, from: jiff::civil::Date) -> Vec<Habit> {
    let Some(until) = horizon(from) else {
        return Vec::new();
    };

    state
        .matching(&Filter::default(), from)
        .into_iter()
        .filter_map(|task| {
            let (Some(repeat), Some(spec)) = (task.repeat, task.date.as_ref()) else {
                return None;
            };

            let mut turns: Vec<jiff::civil::Date> = Vec::new();
            let own = spec.date();
            if own > from && own <= until {
                turns.push(own);
            }
            if repeat.cadence().every > 0 {
                let mut walked = repeat.cadence().beyond(spec.at, from);
                for _ in 0..AHEAD {
                    let Some(at) = walked else {
                        break;
                    };
                    let on = at.date();
                    if on > until || repeat.ended(on) {
                        break;
                    }
                    if !turns.contains(&on) {
                        turns.push(on);
                    }
                    walked = repeat.cadence().after(at);
                }
            }

            (!turns.is_empty()).then(|| Habit {
                task: task.clone(),
                series: tisty_core::series::series(state, task.id).map(|mut told| {
                    told.turns.drain(..told.turns.len().saturating_sub(BEADS));
                    told
                }),
                on: (turns.len() == 1).then(|| turns[0]),
            })
        })
        .collect()
}

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

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Refusal {
    code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

/// A refusal the person could not have caused by asking for something ordinary: it says the store
/// and the window disagree, and that is worth a line in a log somebody will send us.
const TELLS_OF_TROUBLE: &[&str] = &[
    "noSuchDoc",
    "noSuchTask",
    "deleteRefused",
    "internal",
    "internalNamed",
    "storeNewer",
    "otherStore",
    "wouldReset",
];

impl Refusal {
    fn of(code: &'static str) -> Self {
        Self::told(code, None)
    }

    fn about(code: &'static str, name: impl Into<String>) -> Self {
        Self::told(code, Some(name.into()))
    }

    /// The name is left out on purpose: it carries what the person wrote, and this file is meant
    /// to be shared.
    fn told(code: &'static str, name: Option<String>) -> Self {
        let facts = [("code", Fact::Code(code))];
        if TELLS_OF_TROUBLE.contains(&code) {
            witness::warn(channel::WINDOW, "the window was refused", &facts);
        } else {
            witness::trace(channel::WINDOW, "the window was refused", &facts);
        }
        Self { code, name }
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

const RELEASES: &str = "https://github.com/rgdevment/Tisty/releases/latest";
/// The releases page hands a copy kept by the Store an installer that would settle beside the
/// package instead of replacing it, and leave the person with two Tistys.
const IN_THE_STORE: &str = "https://apps.microsoft.com/detail/9PGVWXD8X93N";

fn where_it_comes_from() -> &'static str {
    match update::route().route {
        update::Route::Store => IN_THE_STORE,
        _ => RELEASES,
    }
}

fn speaks_spanish() -> bool {
    let chosen = tisty_core::Paths::resolve()
        .ok()
        .and_then(|paths| tisty_core::config::Config::load_or_init(&paths).ok())
        .and_then(|config| config.locale);
    tisty_core::model::spoken(chosen.as_deref())
        .to_lowercase()
        .starts_with("es")
}

fn behind_words(spanish: bool, itself: bool) -> (String, &'static str, &'static str) {
    let (said, how, yes, no) = if spanish {
        (
            "Una versión más nueva de Tisty actualizó tus datos.

Actualiza este Tisty para que los dos vuelvan a entenderse: abrirlos con esta versión perdería trabajo.",
            "

Esta copia la actualiza quien la instaló, no Tisty.",
            "Actualizar",
            "Cerrar",
        )
    } else {
        (
            "A newer Tisty updated your data.

Update this one so the two agree again: opening it with this version would lose work.",
            "

This copy is updated by whoever installed it, not by Tisty.",
            "Update",
            "Close",
        )
    };
    match itself {
        true => (said.to_string(), yes, no),
        false => (format!("{said}{how}"), yes, no),
    }
}

fn takes_itself_there() -> bool {
    update::self_installs(update::route().route) && !update::from_a_mount()
}

fn behind_said() -> String {
    behind_words(speaks_spanish(), takes_itself_there()).0
}

fn behind_buttons() -> (&'static str, &'static str) {
    let (_, yes, no) = behind_words(speaks_spanish(), takes_itself_there());
    (yes, no)
}

fn blamed(channel: &'static str, said: &'static str, error: tisty_core::Error) -> Refusal {
    witness::error(channel, said, &error.told());
    match error {
        tisty_core::Error::UnsupportedVersion(_) => Refusal::of("storeNewer"),
        other => Refusal::about("internalNamed", other.to_string()),
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

#[cfg(not(windows))]
fn owner(_app: &tauri::AppHandle) -> Option<isize> {
    None
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
    /// The folder's own mark, so only the one that was shelved offers to come back.
    archived: bool,
    /// What the archive holds, the folders above it counted in.
    away: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Filed {
    id: String,
    file: String,
    tags: Vec<String>,
    title: String,
    told: bool,
    bytes: Option<u64>,
    wrote: Option<String>,
    folder: Option<String>,
    /// The document's own mark, so the menu offers what the document itself can answer for.
    archived: bool,
    /// What the archive holds, the folder above it counted in.
    away: bool,
    locked: bool,
    gone: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    guest: Option<String>,
    page_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flagged: Option<Marked>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_was: Option<Vec<String>>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Marked {
    at: String,
    said: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    via: Option<String>,
}

const CATCHING_UP_AT_ONCE: usize = 500;

#[tauri::command(async)]
fn docs_catch_up(session: tauri::State<'_, Mutex<Session>>) -> Answer<Vec<Filed>> {
    let (root, owing) = {
        let held = held(&session);
        let owing: std::collections::BTreeSet<String> = held
            .state
            .docs
            .values()
            .filter(|one| one.title.is_none())
            .map(|one| one.file.clone())
            .collect();
        (held.paths.docs(), owing)
    };

    let found: Vec<tisty_core::docs::Doc> = match owing.is_empty() {
        true => Vec::new(),
        false => tisty_core::docs::all(&root)
            .into_iter()
            .filter(|one| owing.contains(&one.id))
            .collect(),
    };

    for some in found.chunks(CATCHING_UP_AT_ONCE) {
        // Read the bodies before taking the session: five hundred files off a synced folder is a
        // second of disk, and nothing else can be asked of Tisty while the guard is held.
        let read: Vec<(&tisty_core::docs::Doc, Option<tisty_core::event::Said>)> = some
            .iter()
            .map(|one| {
                let said = tisty_core::docs::read(&root, &one.id)
                    .ok()
                    .map(|body| tisty_core::event::Said::of(&body));
                (one, said)
            })
            .collect();

        let mut held = held(&session);
        let ops: Vec<Op> = read
            .into_iter()
            .filter_map(|(one, said)| {
                let kept = held.state.docs.values().find(|kept| kept.file == one.id)?;
                let said = said
                    .unwrap_or_else(|| tisty_core::event::Said {
                        title: one.title.clone(),
                        bytes: None,
                        tags: Some(kept.tags.clone()),
                        by: None,
                    })
                    .by(None);
                said.news_for(kept).then_some(Op::DocSaid {
                    id: kept.id,
                    d: said,
                })
            })
            .collect();
        if !ops.is_empty() {
            held.commit_all(ops)
                .map_err(|e| blamed(channel::WINDOW, "the titles could not be written down", e))?;
        }
    }
    Ok(gathered(&held(&session)))
}

#[tauri::command(async)]
fn docs(session: tauri::State<'_, Mutex<Session>>) -> Answer<Papers> {
    let mut session = held(&session);
    session.reload()?;
    Ok(Papers {
        folders: hanging(&session.state, None),
        docs: gathered(&session),
    })
}

fn gathered(session: &Session) -> Vec<Filed> {
    let on_disk = tisty_core::docs::names(&session.paths.docs());
    let mut kept_in_order: Vec<&tisty_core::model::Kept> = session.state.docs.values().collect();
    kept_in_order.sort_by(|a, b| a.order.cmp(&b.order).then(a.id.cmp(&b.id)));

    kept_in_order
        .into_iter()
        .map(|kept| Filed {
            id: kept.id.to_string(),
            file: kept.file.clone(),
            title: kept.title.clone().unwrap_or_default(),
            told: kept.title.is_some(),
            bytes: kept.bytes,
            wrote: kept.wrote.map(|at| at.to_string()),
            folder: kept.folder.map(|at| at.to_string()),
            archived: kept.archived,
            away: session.state.held_away(kept),
            locked: session.state.shut(kept.id),
            gone: !on_disk.contains(&kept.file),
            // Empty means it came from elsewhere under nobody's name: the list still has to
            // say so before anybody signs it as their own.
            guest: kept.guest.then(|| kept.by.clone().unwrap_or_default()),
            page_of: kept.page_of.map(|up| up.to_string()),
            tags: kept.tags.iter().map(|one| one.to_string()).collect(),
            flagged: kept.flagged.as_ref().map(|mark| Marked {
                at: mark.at.to_string(),
                said: mark.body.clone(),
                via: mark.via.clone(),
            }),
            folder_was: kept.folder_was.clone(),
        })
        .collect()
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
                archived: one.archived,
                away: state.folder_away(one.id),
            }];
            branch.append(&mut hanging(state, Some(one.id)));
            branch
        })
        .collect()
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

fn one_step_back(session: &Session, id: &str) -> Option<String> {
    let was = tisty_core::docs::read_before(session.paths.data(), id)?;
    let now = tisty_core::docs::read(&session.paths.docs(), id).ok()?;
    if tisty_core::docs::unchanged(&now, &was) {
        return None;
    }
    let print = tisty_core::attach::printed(now.as_bytes());
    match tisty_core::docs::before_left_at(session.paths.data(), id).as_deref() == Some(&print) {
        true => Some(was),
        false => None,
    }
}

fn went_back(session: &mut Session, id: &str) -> Answer<tisty_core::docs::Doc> {
    if session.state.bolted(id) {
        return Err(Refusal::of(match session.state.away(id) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    let Some(was) = one_step_back(session, id) else {
        return Err(Refusal::of("nothingKeptBeside"));
    };
    let root = session.paths.docs();
    let now = tisty_core::docs::read(&root, id)
        .map_err(|e| blamed(channel::WINDOW, "a document could not be read", e))?;
    let print = tisty_core::attach::printed(now.as_bytes());
    tisty_core::docs::kept_before(session.paths.data(), id, &now, &was)
        .map_err(|e| blamed(channel::WINDOW, "what a document said could not be kept", e))?;
    let made =
        tisty_core::docs::rewrite(&root, session.paths.data(), id, &was, &print).map_err(|e| {
            match e {
                tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
                _ => blamed(channel::WINDOW, "a document could not be written", e),
            }
        })?;
    let whole = match made {
        tisty_core::docs::Rewrite::Moved => return Err(Refusal::about("documentMoved", id)),
        tisty_core::docs::Rewrite::Made { whole, .. } => whole,
    };
    session.mind_body(id, &tisty_core::docs::settled(&whole));
    session.corpus.forget(id);
    let hand = signing(&session.state);
    session.retell(id, &whole, hand);
    Ok(tisty_core::docs::Doc {
        title: tisty_core::docs::titled(&whole),
        id: id.to_string(),
    })
}

#[tauri::command(async)]
fn doc_back(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    went_back(&mut session, &id)
}

#[tauri::command(async)]
fn doc_backable(session: tauri::State<'_, Mutex<Session>>, id: String) -> Answer<bool> {
    let session = held(&session);
    Ok(!session.state.bolted(&id) && one_step_back(&session, &id).is_some())
}

#[tauri::command(async)]
fn doc_write(
    session: tauri::State<'_, Mutex<Session>>,
    id: String,
    body: String,
    anyway: Option<bool>,
) -> Answer<tisty_core::docs::Doc> {
    let mut session = held(&session);
    if session.state.bolted(&id) {
        return Err(Refusal::of(match session.state.away(&id) {
            true => "documentAway",
            false => "documentLocked",
        }));
    }
    if !anyway.unwrap_or(false) && session.moved(&id) {
        return Err(Refusal::about("documentMoved", id));
    }
    let root = session.paths.docs();
    if tisty_core::docs::read(&root, &id).is_ok_and(|was| tisty_core::docs::unchanged(&was, &body))
    {
        return Ok(tisty_core::docs::Doc {
            title: tisty_core::docs::titled(&body),
            id,
        });
    }
    tisty_core::docs::write(&root, &id, &body).map_err(|e| match e {
        tisty_core::Error::DocumentTooBig { limit, .. } => {
            Refusal::about("documentTooLong", weighed(limit))
        }
        tisty_core::Error::AlreadyRunning => Refusal::of("documentBeingWritten"),
        _ => blamed(channel::WINDOW, "a document could not be written", e),
    })?;
    session.mind_body(&id, &tisty_core::docs::settled(&body));
    session.corpus.forget(&id);
    let hand = signing(&session.state);
    session.retell(&id, &body, hand);
    let title = tisty_core::docs::titled(&body);
    Ok(tisty_core::docs::Doc { title, id })
}

/// What reading a body catches up with: the title and the tags both come out of it, and neither
/// is worth a line in the log unless it changed. Size alone is not news — it moves with every
/// keystroke — but where there is news anyway, the note carries the size it was read at.
fn noted(session: &mut Session, file: &str, body: &str) {
    let Some(kept) = session.state.docs.values().find(|one| one.file == file) else {
        return;
    };
    let told = tisty_core::event::Said {
        title: tisty_core::docs::titled(body),
        bytes: kept.bytes,
        tags: Some(tisty_core::tagging::tags_in(body)),
        by: None,
    };
    if !told.news_for(kept) {
        return;
    }
    let id = kept.id;
    let said = tisty_core::event::Said {
        bytes: Some(tisty_core::docs::settled(body).len() as u64),
        // Reading a document is not writing into it, whoever happens to be signing.
        by: None,
        ..told
    };
    if let Err(e) = session.commit(Op::DocSaid { id, d: said }) {
        witness::warn(
            channel::STORE,
            "what a document says of itself could not be written down",
            &[
                ("at", Fact::Id(id.to_string())),
                ("why", Fact::Why(e.to_string())),
            ],
        );
    }
}

/// Read as a file, ordered from the log: a body that arrived from elsewhere may say otherwise.
#[tauri::command(async)]
fn doc_order(session: tauri::State<'_, Mutex<Session>>, id: String, body: String) -> Answer<bool> {
    Ok(held(&session).retell(&id, &body, None))
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
                            let _ =
                                tauri_plugin_opener::open_url(where_it_comes_from(), None::<&str>);
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
            docs,
            docs_catch_up,
            read_tags,
            answers::shelves::folder_add,
            answers::shelves::folder_rename,
            answers::shelves::folder_look,
            answers::shelves::folder_drop,
            answers::papers::doc_file,
            answers::papers::doc_read,
            answers::papers::doc_facts,
            answers::papers::keep_pdf,
            doc_back,
            doc_backable,
            doc_write,
            doc_order,
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
mod copying {
    use super::Session;
    use tisty_core::model::DocId;
    use tisty_core::{Op, Paths};

    struct Desk {
        _tmp: tempfile::TempDir,
        paths: Paths,
    }

    fn desk() -> Desk {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        Desk { _tmp: tmp, paths }
    }

    fn wrote(session: &mut Session, title: &str, up: Option<DocId>) -> DocId {
        let made = tisty_core::docs::create(
            &session.paths.docs(),
            &session.config.device_id,
            &format!("# {title}"),
        )
        .unwrap();
        let id = ulid::Ulid::generate();
        session
            .commit(Op::DocAdd {
                id,
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: made.id,
                    order: tisty_core::order::last_of(
                        session
                            .state
                            .docs
                            .values()
                            .filter(|one| one.page_of == up)
                            .map(|one| one.order.as_str()),
                    ),
                    folder: None,
                    page_of: up,
                },
            })
            .unwrap();
        id
    }

    fn twin_of(session: &Session, made: &tisty_core::docs::Doc) -> DocId {
        session
            .state
            .docs
            .values()
            .find(|one| one.file == made.id)
            .unwrap()
            .id
    }

    #[test]
    fn a_copy_of_a_page_in_the_archive_is_in_the_archive_too() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let book = wrote(&mut session, "Book", None);
        let page = wrote(&mut session, "Page", Some(book));
        session.commit(Op::DocArchive { id: page }).unwrap();

        let made = session.copy_doc(&page.to_string()).unwrap();
        let twin = twin_of(&session, &made);

        assert!(
            session.state.docs[&twin].archived,
            "a copy of what was put away comes back awake and loses what the person had decided"
        );
        assert_eq!(session.state.docs[&twin].page_of, Some(book));
    }

    #[test]
    fn copying_a_book_keeps_each_page_where_the_person_left_it() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let book = wrote(&mut session, "Book", None);
        let first = wrote(&mut session, "First", Some(book));
        wrote(&mut session, "Second", Some(book));
        session.commit(Op::DocArchive { id: first }).unwrap();

        let made = session.copy_doc(&book.to_string()).unwrap();
        let twin = twin_of(&session, &made);

        assert!(!session.state.docs[&twin].archived);
        let pages: Vec<bool> = session
            .state
            .pages_of(twin)
            .iter()
            .map(|one| one.archived)
            .collect();
        assert_eq!(
            pages,
            vec![true, false],
            "the copy has to hold the same two pages, one of them put away"
        );
    }

    #[test]
    fn a_page_of_a_document_in_the_archive_is_not_copied_into_it() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let book = wrote(&mut session, "Book", None);
        let page = wrote(&mut session, "Page", Some(book));
        session.commit(Op::DocArchive { id: book }).unwrap();

        assert!(
            session.copy_doc(&page.to_string()).is_err(),
            "a copy is a new page, and nothing new goes into the archive"
        );
        assert_eq!(session.state.pages_of(book).len(), 1);
    }

    #[test]
    fn copying_a_document_in_the_archive_leaves_the_copy_there() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let book = wrote(&mut session, "Book", None);
        wrote(&mut session, "Page", Some(book));
        session.commit(Op::DocArchive { id: book }).unwrap();

        let made = session.copy_doc(&book.to_string()).unwrap();
        let twin = twin_of(&session, &made);

        assert!(session.state.docs[&twin].archived);
        let page = session.state.pages_of(twin)[0];
        assert!(
            !page.archived && session.state.held_away(page),
            "the page is covered by the copy, not marked on its own"
        );
    }
}

#[cfg(test)]
mod hanging_again {
    use super::Session;
    use tisty_core::{Op, Paths};

    fn desk() -> (tempfile::TempDir, Paths) {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        (tmp, paths)
    }

    fn wrote(
        session: &mut Session,
        body: &str,
        page_of: Option<tisty_core::model::DocId>,
    ) -> String {
        let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body)
            .unwrap();
        session
            .commit(Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: made.id.clone(),
                    order: tisty_core::order::last_of(
                        session.state.docs.values().map(|one| one.order.as_str()),
                    ),
                    folder: None,
                    page_of,
                },
            })
            .unwrap();
        made.id
    }

    fn id_of(session: &Session, file: &str) -> tisty_core::model::DocId {
        session
            .state
            .docs
            .values()
            .find(|one| one.file == file)
            .unwrap()
            .id
    }

    fn held(session: &Session, up: tisty_core::model::DocId) -> Vec<String> {
        session
            .state
            .pages_of(up)
            .into_iter()
            .map(|one| one.file.clone())
            .collect()
    }

    #[test]
    fn a_page_hung_again_is_read_where_the_book_still_names_it() {
        let (_tmp, paths) = desk();
        let mut session = Session::at(paths).unwrap();
        let book = wrote(&mut session, "# Libro\n\nintro\n", None);
        let up = id_of(&session, &book);
        let one = wrote(&mut session, "# Uno\n", Some(up));
        let two = wrote(&mut session, "# Dos\n", Some(up));
        tisty_core::docs::write(
            &session.paths.docs(),
            &book,
            &format!("# Libro\n\nintro\n\n![Uno](tisty:doc/{one})\n\n![Dos](tisty:doc/{two})\n"),
        )
        .unwrap();
        let body = tisty_core::docs::read(&session.paths.docs(), &book).unwrap();
        session.retell(&book, &body, None);
        let was = held(&session, up);
        assert_eq!(was, vec![one.clone(), two.clone()]);

        // What dropping it on the book again commits: membership, and a key of its own at the end.
        let mine = id_of(&session, &one);
        let last =
            tisty_core::order::last_of(session.state.docs.values().map(|one| one.order.as_str()));
        session
            .commit(Op::DocMove {
                id: mine,
                d: tisty_core::event::Filed {
                    folder: None,
                    page_of: Some(Some(up)),
                    order: Some(last),
                },
            })
            .unwrap();
        let body = tisty_core::docs::read(&session.paths.docs(), &book).unwrap();
        session.retell(&book, &body, None);

        assert_eq!(
            held(&session, up),
            was,
            "the text did not change, so neither did the order"
        );
    }
}

#[cfg(test)]
mod going_back {
    use super::{Session, one_step_back, went_back};
    use tisty_core::{Op, Paths};

    struct Desk {
        _tmp: tempfile::TempDir,
        paths: Paths,
    }

    fn desk() -> Desk {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        Desk { _tmp: tmp, paths }
    }

    fn wrote(session: &mut Session, body: &str) -> String {
        let made = tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, body)
            .unwrap();
        session
            .commit(Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: made.id.clone(),
                    order: tisty_core::order::first(),
                    folder: None,
                    page_of: None,
                },
            })
            .unwrap();
        made.id
    }

    fn agent_wrote(session: &Session, id: &str, body: &str) {
        let was = tisty_core::docs::read(&session.paths.docs(), id).unwrap();
        let print = tisty_core::attach::printed(was.as_bytes());
        tisty_core::docs::rewrite(
            &session.paths.docs(),
            session.paths.data(),
            id,
            body,
            &print,
        )
        .unwrap();
    }

    #[test]
    fn a_document_nothing_wrote_over_has_nothing_to_go_back_to() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let doc = wrote(&mut session, "# Acta\n\nlo que hay\n");

        assert!(one_step_back(&session, &doc).is_none());
        let why = went_back(&mut session, &doc).unwrap_err();
        assert_eq!(why.code, "nothingKeptBeside");
    }

    #[test]
    fn going_back_puts_the_body_back_and_keeps_the_one_it_wrote_over() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let doc = wrote(&mut session, "# Acta\n\nlo que hay\n");
        agent_wrote(&session, &doc, "# Acta\n\nlo que dejo el agente\n");

        let said = went_back(&mut session, &doc).unwrap();

        assert_eq!(said.title, "Acta");
        let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
        assert!(now.contains("lo que hay"), "{now}");
        let kept = tisty_core::docs::read_before(session.paths.data(), &doc).unwrap();
        assert!(kept.contains("lo que dejo el agente"), "{kept}");
    }

    #[test]
    fn going_back_twice_leaves_the_document_where_it_started() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let doc = wrote(&mut session, "# Acta\n\nuno\n");
        agent_wrote(&session, &doc, "# Acta\n\ndos\n");

        went_back(&mut session, &doc).unwrap();
        went_back(&mut session, &doc).unwrap();

        let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
        assert!(now.contains("dos"), "{now}");
    }

    #[test]
    fn a_body_written_since_the_copy_was_set_aside_is_not_taken_with_it() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let doc = wrote(&mut session, "# Acta\n\nuno\n");
        agent_wrote(&session, &doc, "# Acta\n\ndos\n");
        tisty_core::docs::write(&session.paths.docs(), &doc, "# Acta\n\ntres\n").unwrap();

        assert!(
            one_step_back(&session, &doc).is_none(),
            "what is kept is two writes back, so it is not one step"
        );
        let why = went_back(&mut session, &doc).unwrap_err();
        assert_eq!(why.code, "nothingKeptBeside");
        let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
        assert!(now.contains("tres"), "nothing was written over: {now}");
    }

    #[test]
    fn a_document_the_archive_holds_does_not_go_back() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let doc = wrote(&mut session, "# Acta\n\nuno\n");
        agent_wrote(&session, &doc, "# Acta\n\ndos\n");
        let id = session
            .state
            .docs
            .values()
            .find(|one| one.file == doc)
            .unwrap()
            .id;
        session.commit(Op::DocArchive { id }).unwrap();

        let why = went_back(&mut session, &doc).unwrap_err();
        assert_eq!(why.code, "documentAway");
        let now = tisty_core::docs::read(&session.paths.docs(), &doc).unwrap();
        assert!(now.contains("dos"), "nothing was written: {now}");
    }
}

#[cfg(test)]
mod deleting {
    use super::Session;
    use tisty_core::{Op, Paths};

    struct Desk {
        _tmp: tempfile::TempDir,
        paths: Paths,
    }

    fn desk() -> Desk {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        Desk { _tmp: tmp, paths }
    }

    fn a_page_under(session: &mut Session, up: Option<tisty_core::model::DocId>) -> String {
        let made =
            tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "# A note")
                .unwrap();
        session
            .commit(Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: made.id.clone(),
                    order: tisty_core::order::first(),
                    folder: None,
                    page_of: up,
                },
            })
            .unwrap();
        made.id
    }

    fn ledgered(desk: &Desk, files: &[&str]) {
        let mut said = tisty_core::docs::Carried::read(desk.paths.data());
        for file in files {
            said.keep(file, "a print");
        }
        said.save(desk.paths.data()).unwrap();
    }

    fn there(desk: &Desk, file: &str) -> bool {
        tisty_core::docs::resolve(&desk.paths.docs(), file).is_ok_and(|at| at.exists())
    }

    fn named(session: &Session, file: &str) -> tisty_core::model::DocId {
        session
            .state
            .docs
            .values()
            .find(|one| one.file == file)
            .unwrap()
            .id
    }

    #[test]
    fn deleting_a_document_takes_its_pages_off_the_disk_and_out_of_the_log() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let parent = a_page_under(&mut session, None);
        let up = named(&session, &parent);
        let page = a_page_under(&mut session, Some(up));
        ledgered(&desk, &[&parent, &page]);

        session.drop_doc(&up.to_string()).unwrap();

        assert!(session.state.docs.is_empty(), "neither is named any more");
        for file in [&parent, &page] {
            assert!(!there(&desk, file), "{file} stayed on the disk");
        }
        assert!(
            tisty_core::docs::Carried::read(desk.paths.data())
                .of(&parent)
                .is_none()
        );
    }

    #[test]
    fn a_page_the_archive_holds_through_its_document_is_not_deleted() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let parent = a_page_under(&mut session, None);
        let up = named(&session, &parent);
        let page = a_page_under(&mut session, Some(up));
        let leaf = named(&session, &page);
        session.commit(Op::DocArchive { id: leaf }).unwrap();
        session.commit(Op::DocArchive { id: up }).unwrap();
        ledgered(&desk, &[&parent, &page]);

        assert!(
            session.drop_doc(&leaf.to_string()).is_err(),
            "deleting has no undo, and the archive keeps what it holds"
        );
        assert!(there(&desk, &page), "and the file is still on the disk");
    }

    #[test]
    fn a_file_that_will_not_go_leaves_the_rest_deleted_and_is_swept_at_the_next_opening() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let parent = a_page_under(&mut session, None);
        let up = named(&session, &parent);
        let page = a_page_under(&mut session, Some(up));

        ledgered(&desk, &[&parent, &page]);

        let at = tisty_core::docs::resolve(&desk.paths.docs(), &parent).unwrap();
        std::fs::remove_file(&at).unwrap();
        std::fs::create_dir(&at).unwrap();

        session
            .drop_doc(&up.to_string())
            .expect("the log is the truth, so a file left behind is not a refusal");
        assert!(session.state.docs.is_empty(), "both are gone from the log");
        assert!(
            !there(&desk, &page),
            "the run carried on past the one that would not go"
        );
        assert!(
            session.state.shed.contains(&parent),
            "the one that stayed is still swept for"
        );
        let said = tisty_core::docs::Carried::read(desk.paths.data());
        assert!(
            said.of(&parent).is_none() && said.of(&page).is_none(),
            "and the ledger forgets both either way"
        );

        std::fs::remove_dir(&at).unwrap();
        std::fs::write(&at, b"# A note").unwrap();
        drop(session);
        let session = Session::at(desk.paths.clone()).unwrap();
        assert!(
            !there(&desk, &parent),
            "the shed is taken out when the window opens"
        );
        drop(session);
    }

    #[test]
    fn a_name_that_is_not_a_document_deletes_nothing() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let file = a_page_under(&mut session, None);

        assert!(session.drop_doc(&file).is_err(), "a file name is not an id");
        assert!(session.drop_doc("nonsense").is_err());
        assert!(there(&desk, &file));
        assert_eq!(session.state.docs.len(), 1);
    }

    #[test]
    fn a_stray_file_is_taken_in_and_then_cannot_be_taken_in_twice() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let made =
            tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "# Minuta")
                .unwrap();

        assert_eq!(
            tisty_core::docs::strayed(&desk.paths.docs(), &[]).len(),
            1,
            "nothing names it yet"
        );
        let took = session.take_in(&made.id).unwrap();
        assert_eq!(took.title, "Minuta");
        assert!(session.state.docs.values().any(|one| one.file == made.id));
        assert!(session.take_in(&made.id).is_err(), "not twice");
    }

    #[test]
    fn a_file_the_log_still_names_is_not_let_go_of() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let file = a_page_under(&mut session, None);

        assert!(
            session.let_go_of(&file).is_err(),
            "the log names it, so it stays"
        );
        assert!(there(&desk, &file));
    }

    #[test]
    fn a_stray_file_nobody_names_is_let_go_of() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let made =
            tisty_core::docs::create(&session.paths.docs(), &session.config.device_id, "# Suelto")
                .unwrap();

        session.let_go_of(&made.id).unwrap();
        assert!(!there(&desk, &made.id));
        assert!(session.state.docs.is_empty(), "and no event was written");
    }

    #[test]
    fn a_file_the_log_already_shed_is_not_taken_in() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let file = a_page_under(&mut session, None);
        let id = named(&session, &file);
        session.drop_doc(&id.to_string()).unwrap();

        std::fs::write(
            tisty_core::docs::resolve(&desk.paths.docs(), &file).unwrap(),
            b"# Vuelve",
        )
        .unwrap();
        assert!(
            session.take_in(&file).is_err(),
            "the next sweep would only take it again"
        );
        assert!(session.state.docs.is_empty());
    }
}

#[cfg(test)]
mod tests {
    /// What answers for an attachment is remembered process-wide, so the tests that reach it
    /// take turns rather than reading each other's answers.
    static ALONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn what_answered_for_its_name_is_remembered_past_this_launch() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());

        let room = tempfile::tempdir().unwrap();
        let kept = room.path().join("cache").join("vouched.json");
        crate::vouching::vouching_kept_at(kept.clone());

        // The name has to answer for the bytes, or nothing is written down to remember.
        let loose = room.path().join("loose.mp4");
        std::fs::write(&loose, b"lo que pesa").unwrap();
        let (sha256, _) = tisty_core::attach::hashed(&loose).unwrap();
        let shelf = &sha256[..2];
        let leaf = format!("charla-{}.mp4", &sha256[2..10]);
        let at = room.path().join("attachments").join(shelf);
        std::fs::create_dir_all(&at).unwrap();
        let file = at.join(&leaf);
        std::fs::copy(&loose, &file).unwrap();
        let reference = format!("attachments/{shelf}/{leaf}");

        assert!(
            vouching::vouches(&file, &reference),
            "the name does not answer for it"
        );
        assert!(kept.is_file(), "the answer was not written down");

        let said: std::collections::HashMap<String, bool> =
            serde_json::from_str(&std::fs::read_to_string(&kept).unwrap()).unwrap();
        let (row, answered) = said.iter().next().expect("one row");
        assert!(*answered, "it was written down as not answering for itself");
        assert!(
            row.starts_with(&file.display().to_string()),
            "the row does not name the file it answered for: {row}"
        );
    }

    #[test]
    fn a_file_icloud_took_away_is_not_read_as_one_that_was_lost() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());

        let here = tempfile::tempdir().unwrap();
        let shared = tempfile::tempdir().unwrap();
        let reference = "attachments/cd/charla-e5f6a7b8.mp4";
        let shelf = shared.path().join("attachments/cd");
        std::fs::create_dir_all(&shelf).unwrap();
        std::fs::write(shelf.join(".charla-e5f6a7b8.mp4.icloud"), b"a few bytes").unwrap();

        // Off a Mac nothing can be asked back, but it is still told apart from what is gone.
        let told = finding::found_in(reference, here.path(), Some(shared.path()));
        match cfg!(target_os = "macos") {
            true => assert!(matches!(
                told,
                finding::Sought::Coming | finding::Sought::At(_)
            )),
            false => assert!(
                matches!(told, finding::Sought::Away),
                "nobody here to ask iCloud"
            ),
        }
        assert!(matches!(
            finding::found_in(
                "attachments/ab/nope-00000000.txt",
                here.path(),
                Some(shared.path())
            ),
            finding::Sought::No
        ));
    }

    #[test]
    fn an_attachment_is_looked_for_here_first_and_then_where_it_is_shared() {
        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
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
            finding::found_in(mine, here.path(), Some(shared.path())),
            finding::Sought::At(_)
        ));
        assert!(
            matches!(
                finding::found_in(theirs, here.path(), Some(shared.path())),
                finding::Sought::At(_)
            ),
            "what only the shared folder holds is still reachable"
        );
        assert!(
            matches!(
                finding::found_in(theirs, here.path(), None),
                finding::Sought::No
            ),
            "without a shared folder there is nowhere else to look"
        );
        assert!(matches!(
            finding::found_in("attachments/ab/nope-00000000.txt", here.path(), None),
            finding::Sought::No
        ));
        let lying = shared.path().join(theirs);
        std::fs::write(&lying, b"other bytes entirely").unwrap();
        assert!(
            matches!(
                finding::found_in(theirs, here.path(), Some(shared.path())),
                finding::Sought::Torn
            ),
            "what does not answer for its own name is not handed over"
        );
        assert!(
            matches!(
                finding::found_in("../outside.txt", here.path(), Some(shared.path())),
                finding::Sought::No
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
        use crate::answers::shelves::named_folder;
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

        let mut allowed: Vec<String> = files
            .iter()
            .filter(|at| {
                std::fs::read_to_string(at)
                    .map(|body| body.contains("allow(unsafe_code)"))
                    .unwrap_or(false)
            })
            .map(|at| at.display().to_string())
            .collect();
        allowed.sort();

        let audited = ["src-tauri/src/lib.rs", "src-tauri/src/shop.rs"];
        assert_eq!(
            allowed.len(),
            audited.len(),
            "unsafe is allowed outside the audited places: {allowed:?}"
        );
        for (mine, is) in allowed.iter().zip(audited) {
            assert!(std::path::Path::new(mine).ends_with(is), "{allowed:?}");
        }
    }

    fn now() -> jiff::Zoned {
        "2026-08-05T09:00:00[America/Santiago]".parse().unwrap()
    }

    fn held(title: &str) -> tisty_core::model::Task {
        tisty_core::model::Task::new(ulid::Ulid::generate(), title, "a0")
    }

    fn away(from: jiff::civil::Date, days: i64) -> jiff::civil::Date {
        from.checked_add(jiff::Span::new().try_days(days).unwrap())
            .unwrap()
    }

    fn kept(state: &mut State, task: tisty_core::model::Task) {
        state.tasks.insert(task.id, task);
    }

    #[test]
    fn what_comes_reaches_a_week_and_stops() {
        let from = today();
        let mut state = State::default();
        for days in [1_i64, 7, 8] {
            let mut task = held("somewhere ahead");
            task.date = Some(tisty_core::model::DateSpec::all_day(
                away(from, days),
                "America/Santiago",
            ));
            kept(&mut state, task);
        }

        assert_eq!(coming(&state, from).len(), 2, "the eighth day is outside");
    }

    #[test]
    fn today_keeps_its_own_place_and_is_not_ahead() {
        let from = today();
        let mut state = State::default();
        let mut task = held("call the bank");
        task.date = Some(tisty_core::model::DateSpec::all_day(
            from,
            "America/Santiago",
        ));
        kept(&mut state, task);

        assert!(coming(&state, from).is_empty());
    }

    #[test]
    fn what_only_falls_due_still_comes() {
        let from = today();
        let mut state = State::default();
        let mut task = held("hand in the report");
        task.deadline = Some(tisty_core::model::DateSpec::all_day(
            away(from, 2),
            "America/Santiago",
        ));
        kept(&mut state, task);

        let out = coming(&state, from);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].on, away(from, 2));
        assert!(out[0].due);
    }

    #[test]
    fn a_deadline_still_counts_when_the_work_was_meant_for_another_day() {
        let from = today();
        let mut state = State::default();
        let mut task = held("finish the report");
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, -1),
            "America/Santiago",
        ));
        task.deadline = Some(tisty_core::model::DateSpec::all_day(
            away(from, 2),
            "America/Santiago",
        ));
        kept(&mut state, task);

        let out = coming(&state, from);

        assert_eq!(out.len(), 1, "the day it was meant for is behind us");
        assert_eq!(out[0].on, away(from, 2));
        assert!(out[0].due);
    }

    #[test]
    fn one_day_that_is_both_is_said_once() {
        let from = today();
        let mut state = State::default();
        let mut task = held("the interview");
        let on = tisty_core::model::DateSpec::all_day(away(from, 1), "America/Santiago");
        task.date = Some(on.clone());
        task.deadline = Some(on);
        kept(&mut state, task);

        let out = coming(&state, from);

        assert_eq!(
            out.len(),
            1,
            "working on it and owing it is one day, not two"
        );
        assert!(!out[0].due);
    }

    #[test]
    fn a_day_to_work_on_it_and_a_day_it_falls_due_are_both_worth_saying() {
        let from = today();
        let mut state = State::default();
        let mut task = held("finish the report");
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, 1),
            "America/Santiago",
        ));
        task.deadline = Some(tisty_core::model::DateSpec::all_day(
            away(from, 3),
            "America/Santiago",
        ));
        kept(&mut state, task);

        let out = coming(&state, from);

        assert_eq!(out.len(), 2);
        assert!(!out[0].due);
        assert!(out[1].due);
    }

    fn daily(from: jiff::civil::Date, until: Option<jiff::civil::Date>) -> tisty_core::model::Task {
        let mut task = held("take the pills");
        task.date = Some(tisty_core::model::DateSpec::all_day(
            from,
            "America/Santiago",
        ));
        let mut repeat = tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 1,
            unit: tisty_core::model::Unit::Day,
        });
        repeat.until = until;
        task.repeat = Some(repeat);
        task
    }

    #[test]
    fn a_routine_is_named_once_and_crowds_no_day() {
        let from = today();
        let mut state = State::default();
        kept(&mut state, daily(from, None));

        assert!(
            coming(&state, from).is_empty(),
            "it holds no day of its own"
        );
        assert_eq!(recurring(&state, from).len(), 1);
    }

    #[test]
    fn a_snapshot_carries_only_the_turns_the_strip_draws() {
        let from = today();
        let mut state = State::default();
        let mut last = None;
        for step in 0..12 {
            let mut turn = daily(away(from, -40 + step), None);
            turn.after = last;
            turn.status = tisty_core::model::Status::Done;
            last = Some(turn.id);
            kept(&mut state, turn);
        }
        let mut open = daily(from, None);
        open.after = last;
        let id = open.id;
        kept(&mut state, open);

        let whole = tisty_core::series::series(&state, id).expect("a routine keeps a series");
        let out = recurring(&state, from);
        let told = out[0].series.as_ref().expect("a routine keeps a series");

        assert!(
            whole.turns.len() > BEADS,
            "the chain is longer than the strip draws, or this proves nothing"
        );
        assert_eq!(
            told.turns.len(),
            BEADS,
            "the strip draws {BEADS} beads, so {BEADS} turns cross the bridge"
        );
        assert_eq!(
            told.kept, whole.kept,
            "the counters still see the whole chain"
        );
    }

    #[test]
    fn a_routine_left_unkept_still_falls_on_its_own_weekday() {
        let from = today();
        let mut state = State::default();
        let mut task = daily(from, None);
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, -10),
            "America/Santiago",
        ));
        task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 1,
            unit: tisty_core::model::Unit::Week,
        }));
        kept(&mut state, task);

        let out = recurring(&state, from);

        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].on,
            Some(away(from, 4)),
            "ten days late, its turn is still the weekday it was dealt"
        );
    }

    #[test]
    fn a_monthly_routine_left_unkept_does_not_vanish_from_the_week() {
        let from = today();
        let mut state = State::default();
        let mut task = daily(from, None);
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, -27),
            "America/Santiago",
        ));
        task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 1,
            unit: tisty_core::model::Unit::Month,
        }));
        kept(&mut state, task);

        assert_eq!(
            recurring(&state, from).len(),
            1,
            "its turn falls inside the week ahead, however late it is"
        );
    }

    #[test]
    fn a_routine_falling_once_this_week_says_which_day() {
        let from = today();
        let mut state = State::default();
        let mut task = daily(from, None);
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, 2),
            "America/Santiago",
        ));
        task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 2,
            unit: tisty_core::model::Unit::Month,
        }));
        kept(&mut state, task);

        let out = recurring(&state, from);

        assert_eq!(out.len(), 1, "its own date lands inside the week");
        assert_eq!(out[0].on, Some(away(from, 2)));
        assert!(coming(&state, from).is_empty(), "and it crowds no day");
    }

    #[test]
    fn a_routine_falling_every_day_names_no_day_at_all() {
        let from = today();
        let mut state = State::default();
        kept(&mut state, daily(from, None));

        let out = recurring(&state, from);

        assert_eq!(out.len(), 1);
        assert_eq!(out[0].on, None, "seven turns name no single day");
    }

    #[test]
    fn a_cadence_owing_nothing_this_week_is_no_routine_of_this_week() {
        let from = today();
        let mut state = State::default();
        let mut task = daily(from, None);
        task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 2,
            unit: tisty_core::model::Unit::Month,
        }));
        kept(&mut state, task);

        assert!(recurring(&state, from).is_empty());
    }

    #[test]
    fn a_cadence_that_has_ended_owes_nothing_at_all() {
        let from = today();
        let mut state = State::default();
        kept(&mut state, daily(from, Some(from)));

        assert!(recurring(&state, from).is_empty());
    }

    #[test]
    fn a_cadence_of_zero_still_names_the_single_day_it_falls_on() {
        let from = today();
        let mut state = State::default();
        let mut task = daily(from, None);
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, 2),
            "America/Santiago",
        ));
        task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 0,
            unit: tisty_core::model::Unit::Day,
        }));
        kept(&mut state, task);

        let out = recurring(&state, from);

        assert_eq!(out.len(), 1, "a cadence of zero should still surface once");
        assert_eq!(
            out[0].on,
            Some(away(from, 2)),
            "«every 0 days» stands still on the same date instead of advancing, so the loop \
             pushes that one real day seven more times and the day gets folded away as if the \
             routine crowded the whole week"
        );
    }

    #[test]
    fn a_cadence_of_four_hundred_days_owes_nothing_this_week() {
        let from = today();
        let mut state = State::default();
        let mut task = daily(from, None);
        task.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 400,
            unit: tisty_core::model::Unit::Day,
        }));
        kept(&mut state, task);

        assert!(recurring(&state, from).is_empty());
    }

    #[test]
    fn a_horizon_past_the_edge_of_the_calendar_is_none() {
        assert_eq!(horizon(jiff::civil::Date::MAX), None);
    }

    #[test]
    fn nothing_comes_or_recurs_once_the_calendar_runs_out() {
        let from = jiff::civil::Date::MAX;
        let mut state = State::default();

        let mut plain = held("at the edge of time");
        plain.date = Some(tisty_core::model::DateSpec::all_day(from, "UTC"));
        kept(&mut state, plain);

        let mut routine = daily(from, None);
        routine.date = Some(tisty_core::model::DateSpec::all_day(from, "UTC"));
        kept(&mut state, routine);

        assert!(coming(&state, from).is_empty());
        assert!(recurring(&state, from).is_empty());
    }

    #[test]
    fn a_hidden_task_neither_comes_nor_recurs() {
        let from = today();
        let mut state = State::default();
        let mut task = held("folded away");
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, 2),
            "America/Santiago",
        ));
        task.hidden = true;
        kept(&mut state, task);

        let mut routine = daily(from, None);
        routine.hidden = true;
        kept(&mut state, routine);

        assert!(coming(&state, from).is_empty());
        assert!(recurring(&state, from).is_empty());
    }

    #[test]
    fn a_dropped_task_neither_comes_nor_recurs() {
        let from = today();
        let mut state = State::default();
        let mut task = held("let go");
        task.date = Some(tisty_core::model::DateSpec::all_day(
            away(from, 2),
            "America/Santiago",
        ));
        task.status = tisty_core::model::Status::Dropped;
        kept(&mut state, task);

        let mut routine = daily(from, None);
        routine.status = tisty_core::model::Status::Dropped;
        kept(&mut state, routine);

        assert!(coming(&state, from).is_empty());
        assert!(recurring(&state, from).is_empty());
    }

    #[test]
    fn a_deadline_on_the_last_day_of_the_window_still_counts() {
        let from = today();
        let mut state = State::default();
        let mut task = held("submit the form");
        task.deadline = Some(tisty_core::model::DateSpec::all_day(
            away(from, AHEAD),
            "America/Santiago",
        ));
        kept(&mut state, task);

        let out = coming(&state, from);

        assert_eq!(out.len(), 1, "the seventh day still belongs to the window");
        assert_eq!(out[0].on, away(from, AHEAD));
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

        let edits = answers::tasks::Edits {
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

        answers::tasks::Edits {
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

        let edits = answers::tasks::Edits {
            no_tags: vec!["casa".to_string()],
            ..Default::default()
        };
        assert_eq!(
            edits.retitled(text, &read, "es").as_deref(),
            Some("comprar pan #casa")
        );
    }

    #[test]
    fn a_refusal_the_window_showed_says_what_it_was_about() {
        use crate::answers::tasks::note_trouble;

        let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());

        let kept = tempfile::tempdir().unwrap();
        let paths =
            tisty_core::paths::Paths::new(kept.path().join("data"), kept.path().join("config"));
        tisty_core::witness::keeps(tisty_core::witness::file(&paths), false);

        note_trouble("noSuchDoc".into(), Some("ycqcwz50-0007".into()));
        note_trouble("noSuchDoc".into(), None);
        note_trouble("comprar pan".into(), Some("ycqcwz50-0008".into()));

        let seen = tisty_core::witness::recent(&paths, 50);
        let shown: Vec<&String> = seen
            .iter()
            .filter(|line| line.contains("the window showed a refusal"))
            .collect();
        assert_eq!(shown.len(), 2, "{shown:?}");
        assert!(shown[0].contains("ycqcwz50-0007"), "{shown:?}");
        assert!(!shown[1].contains("at="), "{shown:?}");
        assert!(
            !seen.iter().any(|line| line.contains("ycqcwz50-0008")),
            "{seen:?}"
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
        let edits = answers::tasks::Edits {
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
        assert!(
            answers::settings::GUIDE_ES.starts_with("# "),
            "la guia en espanol no viaja"
        );
        assert!(
            answers::settings::GUIDE_EN.starts_with("# "),
            "la guia en ingles no viaja"
        );
    }

    #[test]
    fn the_guide_carries_pages_of_its_own_to_show_what_a_page_is() {
        for (told, leaves, tongue) in [
            (
                answers::settings::GUIDE_ES,
                answers::settings::GUIDE_PAGES_ES,
                "es",
            ),
            (
                answers::settings::GUIDE_EN,
                answers::settings::GUIDE_PAGES_EN,
                "en",
            ),
        ] {
            assert_eq!(leaves.len(), 2, "the {tongue} guide lost a page");
            for (marker, leaf) in leaves {
                assert!(
                    told.contains(&format!("]({marker})")),
                    "the {tongue} guide never names {marker}"
                );
                assert!(
                    leaf.starts_with("# "),
                    "a {tongue} page has no title: {marker}"
                );
                assert!(
                    tisty_core::docs::survives(leaf).is_ok(),
                    "the {tongue} page {marker} would open read-only: {:?}",
                    tisty_core::docs::survives(leaf)
                );
            }
            assert!(
                leaves.iter().any(|(_, one)| one.contains("](rina.jpg)")),
                "no {tongue} page shows the picture"
            );
            assert!(
                leaves.iter().any(|(_, one)| one.contains("```rust")),
                "no {tongue} page shows any code"
            );
        }
    }

    #[test]
    fn every_picture_the_guide_names_is_where_the_bundler_looks() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/guide");
        for (told, tongue) in [
            (answers::settings::GUIDE_ES, "es"),
            (answers::settings::GUIDE_EN, "en"),
        ] {
            for shot in answers::settings::PICTURES {
                if !told.contains(&format!("]({shot})")) {
                    continue;
                }
                assert!(root.join(tongue).join(shot).is_file(), "falta {shot}");
            }
        }
    }
}

#[cfg(test)]
mod behind_tests {
    use super::*;

    #[test]
    fn what_a_machine_left_behind_is_told_says_what_to_do_not_what_broke() {
        for spanish in [true, false] {
            for itself in [true, false] {
                let (said, yes, no) = behind_words(spanish, itself);
                assert!(!said.contains("schema"), "{said}");
                assert!(!said.chars().any(|one| one.is_ascii_digit()), "{said}");
                assert!(!yes.is_empty() && !no.is_empty());
                assert_eq!(
                    itself,
                    !said.contains("instaló") && !said.contains("installed"),
                    "a copy something else updates has to be told so: {said}"
                );
            }
        }
        assert!(
            update::ours(RELEASES),
            "the offer has to lead where our releases live"
        );
    }
}

#[cfg(test)]
mod hands_of {
    use crate::answers::agents::hands;
    use tisty_core::{DeviceId, Event, Op};

    fn wrote(device: &str, via: Option<&str>, at: i64, op: Op) -> Event {
        let mut event = Event::new(
            DeviceId(device.into()),
            jiff::Timestamp::from_second(at).unwrap(),
            op,
        );
        event.via = via.map(str::to_string);
        event
    }

    fn task(title: &str) -> Op {
        Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::TaskAdd::new(title, "a0"),
        }
    }

    #[test]
    fn a_client_is_one_row_whatever_it_called_itself_and_a_join_is_not_writing() {
        let agent = DeviceId("dev_agent".into());
        let events = vec![
            wrote(
                "dev_agent",
                None,
                1,
                Op::DeviceJoin {
                    d: agent.clone(),
                    k: Some(tisty_core::event::DeviceKind::Agent),
                },
            ),
            wrote(
                "dev_agent",
                None,
                1,
                Op::DeviceHost {
                    d: agent.clone(),
                    of: DeviceId("dev_laptop".into()),
                },
            ),
            wrote("dev_agent", Some("codex-mcp-client"), 10, task("one")),
            wrote("dev_agent", Some("Codex"), 20, task("two")),
            wrote("dev_agent", Some("claude-code"), 30, task("three")),
            wrote("dev_laptop", None, 40, task("the person's own")),
        ];
        let seen = vec![super::wiring::Seen {
            id: "codex",
            name: "Codex",
            at: "~/.codex/config.toml".into(),
            wired: true,
            astray: false,
            points: None,
        }];

        let rows = hands(&events, &[agent].into_iter().collect(), &seen);

        let said: Vec<String> = rows
            .iter()
            .map(|row| {
                format!(
                    "{}={} wired:{:?} filed:{} wrote:{}",
                    row.via.as_deref().unwrap_or("-"),
                    row.named,
                    row.wired,
                    row.filed,
                    row.wrote
                )
            })
            .collect();
        assert_eq!(
            said,
            vec![
                "codex=Codex wired:Some(true) filed:2 wrote:2",
                "claude-code=Claude Code wired:None filed:1 wrote:1",
            ],
            "{rows:?}"
        );
        assert_eq!(rows[0].last.as_deref(), Some("1970-01-01T00:00:20Z"));
    }
}

#[cfg(test)]
mod starring {
    use super::{Asking, CLOSED_ENOUGH, PAPERS_ENOUGH, asking};

    fn at(text: &str) -> jiff::Timestamp {
        text.parse().expect("a timestamp")
    }

    fn now() -> jiff::Timestamp {
        at("2026-09-22T12:00:00Z")
    }

    const LONG_AGO: &str = "2026-01-01T00:00:00Z";

    #[test]
    fn a_copy_that_was_asked_once_is_never_asked_again() {
        assert_eq!(
            asking(Some(true), Some(at(LONG_AGO)), now(), 10_000, || 10_000),
            Asking::Wait
        );
    }

    #[test]
    fn a_copy_with_no_mark_lays_one_and_says_nothing_yet() {
        assert_eq!(asking(None, None, now(), 10_000, || 10_000), Asking::Start);
    }

    #[test]
    fn a_mark_from_a_clock_that_was_ahead_is_laid_again_rather_than_waited_on_for_ever() {
        assert_eq!(
            asking(
                None,
                Some(at("2030-01-01T00:00:00Z")),
                now(),
                10_000,
                || { 10_000 }
            ),
            Asking::Start
        );
    }

    #[test]
    fn a_fortnight_is_the_floor_and_the_day_before_it_is_not() {
        assert_eq!(
            asking(
                None,
                Some(at("2026-09-08T12:00:01Z")),
                now(),
                10_000,
                || { 10_000 }
            ),
            Asking::Wait
        );
        assert_eq!(
            asking(
                None,
                Some(at("2026-09-08T12:00:00Z")),
                now(),
                10_000,
                || { 10_000 }
            ),
            Asking::Now
        );
    }

    #[test]
    fn either_floor_opens_it_and_neither_alone_being_short_does() {
        let long_ago = Some(at(LONG_AGO));
        assert_eq!(
            asking(None, long_ago, now(), PAPERS_ENOUGH - 1, || CLOSED_ENOUGH
                - 1),
            Asking::Wait
        );
        assert_eq!(
            asking(None, long_ago, now(), PAPERS_ENOUGH, || CLOSED_ENOUGH - 1),
            Asking::Now
        );
        assert_eq!(
            asking(None, long_ago, now(), PAPERS_ENOUGH - 1, || CLOSED_ENOUGH),
            Asking::Now
        );
    }

    #[test]
    fn the_archive_is_not_walked_when_the_answer_is_known_without_it() {
        let walked = std::cell::Cell::new(0);
        let count = || {
            walked.set(walked.get() + 1);
            10_000
        };
        assert_eq!(asking(Some(true), None, now(), 0, count), Asking::Wait);
        assert_eq!(walked.get(), 0);

        let count = || {
            walked.set(walked.get() + 1);
            10_000
        };
        assert_eq!(asking(None, Some(now()), now(), 0, count), Asking::Wait);
        assert_eq!(walked.get(), 0);

        let count = || {
            walked.set(walked.get() + 1);
            10_000
        };
        assert_eq!(
            asking(None, Some(at(LONG_AGO)), now(), PAPERS_ENOUGH, count),
            Asking::Now
        );
        assert_eq!(walked.get(), 0);
    }
}

#[cfg(test)]
mod doors {
    use super::offering;

    #[test]
    fn a_machine_without_an_assistant_is_never_offered_one() {
        assert!(!offering(0, 0));
    }

    #[test]
    fn an_assistant_that_is_found_and_not_yet_wired_is_worth_offering() {
        assert!(offering(1, 0));
        assert!(offering(6, 0));
    }

    #[test]
    fn somebody_who_already_wired_one_knows_where_the_door_is() {
        assert!(!offering(1, 1));
        assert!(!offering(6, 6));
    }

    #[test]
    fn one_wired_assistant_speaks_for_the_rest() {
        assert!(!offering(6, 1));
    }
}

#[cfg(test)]
mod hosting {
    use super::Session;
    use tisty_core::{Config, DeviceId, Op, Paths, Store};

    /// An agent that joined before `device.host` existed gets its host written by the window,
    /// once, as the machine — and a machine with no agent writes nothing.
    #[test]
    fn the_window_says_where_an_older_agent_lives_once() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        let lines = || {
            tisty_core::store::read_all(paths.store())
                .unwrap()
                .into_iter()
                .filter(|one| matches!(one.op, Op::DeviceHost { .. }))
                .count()
        };

        Session::at(paths.clone()).unwrap();
        assert_eq!(lines(), 0, "no agent, nothing to say");

        let mut config = Config::load_or_init(&paths).unwrap();
        let agent = DeviceId("dev_agent".into());
        config.agent_id = Some(agent.clone());
        config.save(&paths).unwrap();
        Store::open(paths.store(), agent.clone())
            .unwrap()
            .append(Op::DeviceJoin {
                d: agent.clone(),
                k: Some(tisty_core::event::DeviceKind::Agent),
            })
            .unwrap();

        let session = Session::at(paths.clone()).unwrap();
        assert_eq!(session.state.hosts.get(&agent), Some(&config.device_id));
        assert_eq!(lines(), 1);
        let said = tisty_core::store::read_all(paths.store())
            .unwrap()
            .into_iter()
            .find(|one| matches!(one.op, Op::DeviceHost { .. }))
            .unwrap();
        assert_eq!(said.device, config.device_id, "the machine's own word");
        assert!(said.optional, "an older build walks past it");

        Session::at(paths.clone()).unwrap();
        assert_eq!(lines(), 1, "said once");
    }
}

#[cfg(test)]
mod letting_go {
    use super::{Session, erasing, opening_to_agents, reading_as};
    use tisty_core::{DeviceId, Op, Paths, Reading, TaskId};

    struct Desk {
        _tmp: tempfile::TempDir,
        paths: Paths,
    }

    fn desk() -> Desk {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        Desk { _tmp: tmp, paths }
    }

    fn closed(session: &mut Session, title: &str) -> TaskId {
        let id = ulid::Ulid::generate();
        session
            .commit(Op::TaskAdd {
                id,
                d: tisty_core::event::TaskAdd::new(title, "a0"),
            })
            .unwrap();
        session.commit(Op::TaskDone { id, filled: false }).unwrap();
        id
    }

    fn code(said: Result<impl std::fmt::Debug, super::Refusal>) -> String {
        match said {
            Ok(_) => "ok".into(),
            Err(refusal) => refusal.code.to_string(),
        }
    }

    #[test]
    fn erasing_keeps_the_rule_the_core_keeps() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let open = ulid::Ulid::generate();
        session
            .commit(Op::TaskAdd {
                id: open,
                d: tisty_core::event::TaskAdd::new("still open", "a0"),
            })
            .unwrap();
        let errand = closed(&mut session, "buy bread");
        let kept = closed(&mut session, "the certificate");
        reading_as(&mut session, kept, Reading::Story).unwrap();

        assert_eq!(code(erasing(&mut session, open)), "onlyArchivedGoes");
        assert_eq!(code(erasing(&mut session, kept)), "storyStays");
        assert_eq!(
            code(erasing(&mut session, ulid::Ulid::generate())),
            "notATaskId"
        );
        assert_eq!(code(erasing(&mut session, errand)), "ok");
        assert!(session.state.is_erased(errand));
    }

    #[test]
    fn converting_is_for_what_is_closed_and_not_a_routine() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let open = ulid::Ulid::generate();
        session
            .commit(Op::TaskAdd {
                id: open,
                d: tisty_core::event::TaskAdd::new("still open", "a0"),
            })
            .unwrap();
        let turn = ulid::Ulid::generate();
        let mut d = tisty_core::event::TaskAdd::new("pills", "a1");
        d.after = Some(ulid::Ulid::generate());
        session.commit(Op::TaskAdd { id: turn, d }).unwrap();
        session
            .commit(Op::TaskDone {
                id: turn,
                filled: false,
            })
            .unwrap();
        let errand = closed(&mut session, "buy bread");

        assert_eq!(
            code(reading_as(&mut session, open, Reading::Story)),
            "onlyClosedConverts"
        );
        assert_eq!(
            code(reading_as(&mut session, turn, Reading::Trace)),
            "routineReadsAsRoutine"
        );
        let told = reading_as(&mut session, errand, Reading::Story).unwrap();
        assert_eq!(told.read_as, Some(Reading::Story));
        assert_eq!(code(erasing(&mut session, errand)), "storyStays");
        assert_eq!(code(erasing(&mut session, turn)), "routineStays");
    }

    /// A closed root with no repeat left reads as a trace by itself; the turn hanging from it
    /// makes it a routine's to the state, so the window refuses to erase it.
    #[test]
    fn a_bare_root_a_turn_hangs_from_is_never_erased() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let root = closed(&mut session, "pills");
        let turn = ulid::Ulid::generate();
        let mut d = tisty_core::event::TaskAdd::new("pills", "a1");
        d.after = Some(root);
        session.commit(Op::TaskAdd { id: turn, d }).unwrap();
        session
            .commit(Op::TaskDone {
                id: turn,
                filled: false,
            })
            .unwrap();
        let errand = closed(&mut session, "buy bread");

        assert_eq!(code(erasing(&mut session, root)), "routineStays");
        assert_eq!(code(erasing(&mut session, errand)), "ok");
        assert!(session.state.tasks.contains_key(&root));
    }

    #[test]
    fn opening_to_agents_is_for_an_open_task_the_person_wrote() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let mine = ulid::Ulid::generate();
        session
            .commit(Op::TaskAdd {
                id: mine,
                d: tisty_core::event::TaskAdd::new("renew the certificate", "a0"),
            })
            .unwrap();
        let errand = closed(&mut session, "buy bread");
        let theirs = ulid::Ulid::generate();
        let agent = DeviceId("dev_agent".into());
        let mut wrote = tisty_core::Store::open(desk.paths.store(), agent.clone()).unwrap();
        wrote
            .append(Op::DeviceJoin {
                d: agent,
                k: Some(tisty_core::event::DeviceKind::Agent),
            })
            .unwrap();
        wrote
            .append(Op::TaskAdd {
                id: theirs,
                d: tisty_core::event::TaskAdd::new("pasar biome", "a1"),
            })
            .unwrap();

        assert_eq!(
            code(opening_to_agents(&mut session, errand, true)),
            "onlyOpenOpens"
        );
        assert_eq!(
            code(opening_to_agents(&mut session, theirs, true)),
            "alreadyTheirs"
        );
        assert_eq!(
            code(opening_to_agents(
                &mut session,
                ulid::Ulid::generate(),
                true
            )),
            "notATaskId"
        );
        let told = opening_to_agents(&mut session, mine, true).unwrap();
        assert!(told.open_to_agents);
        assert!(
            session
                .state
                .attended_by_agents(&session.state.tasks[&mine])
        );
        let told = opening_to_agents(&mut session, mine, false).unwrap();
        assert!(!told.open_to_agents);
    }
}

#[cfg(test)]
mod ordering {
    use super::Session;
    use crate::answers::papers::beside_docs;
    use crate::answers::shelves::beside_folders;
    use tisty_core::{Op, Paths};

    struct Desk {
        _tmp: tempfile::TempDir,
        paths: Paths,
    }

    fn desk() -> Desk {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        std::fs::create_dir_all(paths.docs()).unwrap();
        Desk { _tmp: tmp, paths }
    }

    fn folder(session: &mut Session, name: &str, order: &str) -> tisty_core::model::FolderId {
        let id = ulid::Ulid::generate();
        session
            .commit(Op::FolderAdd {
                id,
                d: tisty_core::event::FolderAdd {
                    name: name.into(),
                    order: order.into(),
                    parent: None,
                    icon: None,
                    color: None,
                },
            })
            .unwrap();
        id
    }

    fn shelf(session: &Session) -> Vec<String> {
        session
            .state
            .under(None)
            .into_iter()
            .map(|one| one.name.clone())
            .collect()
    }

    #[test]
    fn a_first_run_holds_its_lists_until_the_welcome_says_where_the_copies_go() {
        let desk = desk();
        let session = Session::at(desk.paths.clone()).unwrap();

        assert!(
            session.state.lists.is_empty(),
            "sown, this store stops looking new and could not adopt a folder that already holds one"
        );
    }

    #[test]
    fn the_lists_arrive_once_the_welcome_is_through_and_never_a_second_time() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();

        session.sow_if_due();
        let first = session.state.lists.len();
        session.sow_if_due();

        assert!(
            first > 0,
            "a machine that syncs nothing still starts with lists"
        );
        assert_eq!(session.state.lists.len(), first);
        assert_eq!(session.config.sown, Some(true));
    }

    #[test]
    fn a_folder_that_already_holds_a_store_is_the_meeting_place_itself() {
        let desk = desk();
        let shared = desk.paths.data().join("shared");
        std::fs::create_dir_all(shared.join(tisty_sync::STORE)).unwrap();

        assert_eq!(super::room(&shared), shared);
    }

    #[test]
    fn anywhere_else_holds_a_folder_of_ours_inside_it() {
        let desk = desk();
        let shared = desk.paths.data().join("Documents");
        std::fs::create_dir_all(&shared).unwrap();

        assert_eq!(super::room(&shared), shared.join(tisty_core::keepers::OURS));
    }

    #[test]
    fn a_guide_that_came_from_another_machine_is_taken_rather_than_planted_again() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let file = "guia-a1b2".to_string();
        session
            .commit(Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    file: file.clone(),
                    order: "a1".into(),
                    said: Some(tisty_core::event::Said {
                        title: tisty_core::docs::titled(crate::answers::settings::GUIDE_ES),
                        bytes: None,
                        tags: Some(Vec::new()),
                        by: None,
                    }),
                    folder: None,
                    page_of: None,
                },
            })
            .unwrap();

        assert_eq!(
            crate::answers::settings::guide_already_here(&session).map(|one| one.0),
            Some(file)
        );
    }

    #[test]
    fn a_document_of_your_own_is_never_mistaken_for_the_guide() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        session
            .commit(Op::DocAdd {
                id: ulid::Ulid::generate(),
                d: tisty_core::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    file: "notas-c3d4".into(),
                    order: "a1".into(),
                    said: Some(tisty_core::event::Said {
                        title: "Mis notas".into(),
                        bytes: None,
                        tags: Some(Vec::new()),
                        by: None,
                    }),
                    folder: None,
                    page_of: None,
                },
            })
            .unwrap();

        assert_eq!(crate::answers::settings::guide_already_here(&session), None);
    }

    #[test]
    fn a_folder_dropped_before_one_that_is_gone_lands_last_rather_than_nowhere() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let moving = folder(&mut session, "mover", "a0");
        folder(&mut session, "uno", "a1");
        folder(&mut session, "dos", "a2");

        let ops = beside_folders(&session.state, moving, None, Some(ulid::Ulid::generate()));
        session.commit_all(ops).unwrap();

        assert_eq!(
            shelf(&session),
            ["uno", "dos", "mover"],
            "a neighbour that vanished must not leave the row where it was"
        );
    }

    #[test]
    fn a_folder_dropped_before_another_lands_right_there() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        folder(&mut session, "uno", "a1");
        let two = folder(&mut session, "dos", "a2");
        let moving = folder(&mut session, "mover", "a3");

        let ops = beside_folders(&session.state, moving, None, Some(two));
        session.commit_all(ops).unwrap();

        assert_eq!(shelf(&session), ["uno", "mover", "dos"]);
    }

    #[test]
    fn dropping_at_the_front_over_and_over_never_grows_a_key_out_of_hand() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let one = folder(&mut session, "uno", "a1");
        let two = folder(&mut session, "dos", "a2");

        for n in 0..600 {
            let (moving, before) = match n % 2 {
                0 => (two, one),
                _ => (one, two),
            };
            let ops = beside_folders(&session.state, moving, None, Some(before));
            session.commit_all(ops).unwrap();
        }

        let longest = session
            .state
            .under(None)
            .into_iter()
            .map(|one| one.order.len())
            .max()
            .unwrap_or(0);
        assert!(longest <= 24, "a key grew to {longest}");
        assert_eq!(shelf(&session).len(), 2);
    }

    #[test]
    fn a_document_dropped_before_one_that_is_gone_lands_last_too() {
        let desk = desk();
        let mut session = Session::at(desk.paths.clone()).unwrap();
        let mut made = |name: &str, order: &str| {
            let id = ulid::Ulid::generate();
            session
                .commit(Op::DocAdd {
                    id,
                    d: tisty_core::event::DocAdd {
                        wrote: None,
                        guest: false,
                        made: None,
                        by: None,
                        said: None,
                        file: name.into(),
                        order: order.into(),
                        folder: None,
                        page_of: None,
                    },
                })
                .unwrap();
            id
        };
        let moving = made("dev_a-0001", "a0");
        made("dev_a-0002", "a1");
        made("dev_a-0003", "a2");

        let ops = beside_docs(&session.state, moving, None, Some(ulid::Ulid::generate()));
        session.commit_all(ops).unwrap();

        let mut sitting: Vec<&tisty_core::model::Kept> = session.state.docs.values().collect();
        sitting.sort_by(|a, b| a.order.cmp(&b.order));
        assert_eq!(
            sitting.last().map(|one| one.file.as_str()),
            Some("dev_a-0001")
        );
    }
}
