use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Config, Event, Op, Paths, State, Store};

use crate::{Answer, Refusal, blamed, folder_open, signing, stale, worded};

pub struct Session {
    pub paths: Paths,
    pub config: Config,
    pub state: State,
    pub store: Store,
    pub cache: Option<tisty_core::cache::Cache>,
    pub corpus: tisty_core::docs::Corpus,
    pub print: String,
    /// What each open document looked like when this window last read or wrote it.
    pub minded: std::collections::HashMap<String, String>,
    pub asked: std::collections::HashMap<String, String>,
    pub locale: Option<String>,
    pub log: Option<(String, Vec<Event>)>,
    /// The fingerprint is read from the directory entry, which lags a write on Windows, so what
    /// was committed here is counted rather than measured.
    writes: u64,
    behind: bool,
}

pub struct Projected {
    state: State,
    print: String,
    writes: u64,
}

/// The fingerprint is taken before the reading, so a store written while this runs leaves the
/// session looking older than it is and the next reload projects again, rather than the reverse.
pub fn projected(paths: &Paths, writes: u64) -> tisty_core::Result<Projected> {
    let print = tisty_core::cache::fingerprint(&paths.store());
    let state = tisty_core::cache::project(&paths.store(), paths.cache())?;
    Ok(Projected {
        state,
        print,
        writes,
    })
}

/// What the toolkit says goes where everything else does. Without this its own refusals — an
/// asset it would not serve, a window it could not draw — are written to a logger nobody set up,
/// so they leave no trace at all and the window simply shows nothing.
pub struct Relayed;

impl Session {
    pub fn open() -> tisty_core::Result<Self> {
        let settled = tisty_core::paths::settle_home();
        if let Some(tisty_core::moving::Settled::Failed(why)) = settled {
            return Err(tisty_core::Error::StoreNotMoved(why));
        }
        let paths = Paths::resolve()?;
        tisty_core::witness::keeps(
            tisty_core::witness::file(&paths),
            tisty_core::witness::wants_all(),
        );
        if settled == Some(tisty_core::moving::Settled::Moved) {
            tisty_core::witness::note(
                tisty_core::witness::channel::STORE,
                "this machine's store moved out of AppData, where the Store would delete it",
                &[],
            );
        }
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
        let guarding = paths.clone();
        let session = Self::at(paths)?;
        std::thread::spawn(move || {
            tisty_core::store::keys_guarded(&guarding);
            tisty_core::paths::home_set_aside(&guarding);
        });
        Ok(session)
    }

    pub fn at(paths: Paths) -> tisty_core::Result<Self> {
        let config = Config::load_or_init(&paths)?;
        tisty_core::store::brought_home(&paths);
        let store = Store::open(paths.store(), config.device_id.clone())?
            .signing_with(tisty_core::signing::mine(&paths, &config.device_id));
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
            asked: std::collections::HashMap::new(),
            log: None,
            writes: 0,
            behind: false,
        };
        session.tidy_up(true);
        session.vouch_for_agent();
        if session.config.sync.is_some() {
            session.sow_if_due();
        }
        Ok(session)
    }

    /// A first run holds its lists back until the welcome has said where the copies go: sown, this
    /// store stops looking new, and a folder that already holds another machine could not be
    /// adopted without asking somebody to throw one of the two away.
    pub fn sow_if_due(&mut self) {
        if self.config.sown == Some(true)
            || !self.state.lists.is_empty()
            || !self.state.tasks.is_empty()
            || self.left_behind()
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

    // What an earlier install left, or a folder's history, is on its way: examples would be twice.
    fn left_behind(&self) -> bool {
        let holds = |at: &std::path::Path| {
            std::fs::read_dir(at).is_ok_and(|mut entries| entries.next().is_some())
        };
        holds(&self.paths.attachments())
            || holds(&self.paths.docs())
            || self
                .place()
                .is_some_and(|at| tisty_core::store::inhabited(at.join(tisty_carrier::STORE)))
    }

    fn settings_on_disk(&self) -> Config {
        match Config::load(&self.paths.config_file()) {
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
        }
    }

    pub fn keep(&mut self, change: impl FnOnce(&mut Config)) -> Answer<()> {
        self.keep_unless(|config| {
            change(config);
            None
        })
    }

    /// The change reads the settings where they are written, not where this window loaded them,
    /// and a refusal writes nothing: another build may have chosen since.
    pub fn keep_unless(
        &mut self,
        change: impl FnOnce(&mut Config) -> Option<Refusal>,
    ) -> Answer<()> {
        let mut fresh = self.settings_on_disk();
        if let Some(refusal) = change(&mut fresh) {
            self.config = fresh;
            return Err(refusal);
        }
        fresh
            .save(&self.paths)
            .map_err(|e| blamed(channel::CONFIG, "the settings could not be saved", e))?;
        self.config = fresh;
        Ok(())
    }

    pub fn reload(&mut self) -> tisty_core::Result<bool> {
        let print = tisty_core::cache::fingerprint(&self.paths.store());
        if !self.behind && print == self.print {
            return Ok(false);
        }
        self.reproject()?;
        Ok(true)
    }

    /// An agent minted before hosts spoke for theirs gets its host's word once, then never looks again.
    fn vouch_for_agent(&mut self) {
        let Some(agent) = self.config.agent_id.clone() else {
            return;
        };
        if self.config.agent_vouched.as_ref() == Some(&agent) {
            return;
        }
        let (config, paths) = (self.config.clone(), self.paths.clone());
        let Ok(log) = self.log() else {
            return;
        };
        let said = match tisty_core::agent::vouch(&config, &paths, log) {
            Some(key) => Some(tisty_core::Op::DeviceHost {
                d: agent.clone(),
                of: config.device_id.clone(),
                p: Some(key),
            }),
            None => tisty_core::agent::unhosted(&config, &self.state),
        };
        if let Some(said) = said
            && let Err(why) = self.commit(said)
        {
            witness::warn(
                channel::WINDOW,
                "this machine could not speak for the agent it runs",
                &[("why", Fact::Why(why.to_string()))],
            );
            return;
        }
        if !self.state.assistants.contains(&agent) {
            return;
        }
        self.config.agent_vouched = Some(agent);
        let _ = self.config.save(&self.paths);
    }

    pub fn log(&mut self) -> tisty_core::Result<&[Event]> {
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

    pub fn reproject(&mut self) -> tisty_core::Result<()> {
        self.adopt(projected(&self.paths, self.writes)?);
        Ok(())
    }

    pub fn writes(&self) -> u64 {
        self.writes
    }

    pub fn behind(&self) -> bool {
        self.behind
    }

    pub fn stale(&self) -> bool {
        self.behind || tisty_core::cache::fingerprint(&self.paths.store()) != self.print
    }

    pub fn adopt(&mut self, fresh: Projected) {
        self.behind = self.writes != fresh.writes
            || tisty_core::cache::fingerprint(&self.paths.store()) != fresh.print;
        self.state = fresh.state;
        self.print = fresh.print;
        self.log = None;
    }

    pub fn keeping(&self) -> tisty_carrier::Keeping {
        tisty_carrier::chosen(self.config.sync.as_ref())
    }

    pub fn carrier(&self) -> Option<tisty_carrier::Shared> {
        self.keeping().carrier
    }

    pub fn place(&self) -> Option<std::path::PathBuf> {
        tisty_carrier::place_of(self.config.sync.as_ref())
    }

    pub fn here(&self) -> tisty_carrier::Here {
        tisty_carrier::Here::of(&self.paths, &self.config)
    }

    /// The way of carrying this machine chose, or the refusal every command that needs one gives.
    pub fn carrying(&self) -> Result<tisty_carrier::Shared, Refusal> {
        let keeping = self.keeping();
        keeping.carrier.ok_or_else(|| {
            Refusal::of(match keeping.chosen {
                tisty_carrier::Chosen::Later => "syncLater",
                _ => "noRemote",
            })
        })
    }

    pub fn alive(&self) -> Vec<String> {
        self.state
            .docs
            .values()
            .map(|one| one.file.clone())
            .collect()
    }

    pub fn mind(&mut self, id: &str) {
        let now = tisty_core::docs::resolve(&self.paths.docs(), id)
            .ok()
            .and_then(|at| tisty_core::docs::print_of(&at).ok().flatten());
        match now {
            Some(print) => self.minded.insert(id.to_string(), print),
            None => self.minded.remove(id),
        };
    }

    /// The body itself, not the file: reading the disk again would mind what nobody here saw.
    pub fn mind_body(&mut self, id: &str, body: &str) {
        self.minded
            .insert(id.to_string(), tisty_core::attach::printed(body.as_bytes()));
    }

    /// `hand` is the alias to seal on the note, and only a write by the person has one: reading
    /// a body back to keep the state honest is not writing into it.
    pub fn retell(&mut self, file: &str, body: &str, hand: Option<String>) -> bool {
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

    pub fn moved(&self, id: &str) -> bool {
        let now = tisty_core::docs::resolve(&self.paths.docs(), id)
            .ok()
            .and_then(|at| tisty_core::docs::print_of(&at).ok().flatten());
        stale(self.minded.get(id).map(String::as_str), now.as_deref())
    }

    /// Here and in the shared folder both, or a machine that holds none of them sees none astray.
    /// The shared folder, but only while this machine leaves anything in it.
    pub fn shared_now(&self) -> Option<std::path::PathBuf> {
        self.place()
            .filter(|_| self.config.holds() != tisty_core::config::Holds::Everywhere)
    }

    /// What the tasks and the documents point at, as written.
    pub fn pointed_at(&self) -> Vec<String> {
        let mut named: Vec<String> = self
            .state
            .tasks
            .values()
            .flat_map(|task| task.references())
            .map(|one| one.target)
            .collect();
        named.extend(tisty_core::docs::referenced(&self.paths.docs()));
        named
    }

    pub fn adrift(&self, held: &[String]) -> tisty_core::attach::Loose {
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

    pub fn retire(&mut self, references: &[String]) -> Answer<usize> {
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
        let gone: std::collections::BTreeSet<String> = told
            .iter()
            .filter_map(|op| match op {
                Op::AttachRetire { d } => Some(d.clone()),
                _ => None,
            })
            .collect();
        let many = told.len();
        self.commit_all(told)
            .map_err(|e| blamed(channel::ATTACH, "the retirement could not be written", e))?;
        // Asked for by name, so the shared folder's copy goes now; the cloud keeps it in its own bin.
        if let Some(dest) = self.dest() {
            tisty_core::attach::sweep(&dest, &gone, &std::collections::BTreeSet::new());
        }
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

    pub fn take_in(&mut self, file: &str) -> Answer<tisty_core::docs::Doc> {
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

    pub fn let_go_of(&mut self, file: &str) -> Answer<()> {
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

    pub fn copy_doc(&mut self, id: &str) -> Answer<tisty_core::docs::Doc> {
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
                    print: None,
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
                        print: None,
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

    pub fn drop_doc(&mut self, id: &str) -> Answer<()> {
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
            if let Some(carrier) = self.carrier() {
                carrier.forget_paper(file);
            }
            said.forget(file);
            tisty_core::docs::forget_carried(self.paths.data(), file);
        }
        let _ = said.save(self.paths.data());
        Ok(())
    }

    pub fn unhang(
        &mut self,
        id: tisty_core::model::DocId,
    ) -> tisty_core::Result<tisty_core::event::Filed> {
        self.log()?;
        let told = &self.log.as_ref().expect("just read").1;
        Ok(tisty_core::undo::unhung(told, &self.state, id))
    }

    pub fn books_among(&self, files: &[String]) -> Vec<String> {
        self.state.books_among(files)
    }

    pub fn settle_what_came(&mut self, read: &[(String, String)], since: u64) {
        if self.writes != since {
            return;
        }
        let told = tisty_core::tidy::settling_what_came(&self.state, read);
        self.settled(told);
    }

    fn settled(&mut self, told: Vec<Op>) {
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

    pub fn dest(&self) -> Option<std::path::PathBuf> {
        self.place()
    }

    pub fn tidy_up(&mut self, bin: bool) {
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

    pub fn sweeping(&self, bin: bool) -> tisty_core::tidy::Sweeping {
        tisty_core::tidy::Sweeping::of(
            &self.paths,
            &self.state,
            self.cache.as_ref(),
            self.dest().as_deref(),
            bin,
        )
    }

    pub fn swept(&mut self, was: &tisty_core::tidy::Already, walked: tisty_core::tidy::Walked) {
        let (_, done) = walked.with(&self.state);
        if &done != was
            && let Some(cache) = self.cache.as_ref()
        {
            cache.note_already(&done);
        }
    }

    pub fn take_a_seat(&mut self) -> tisty_core::Result<()> {
        let who = self.config.device_id.clone();
        let shown = tisty_core::signing::mine(&self.paths, &who)
            .as_ref()
            .map(tisty_core::signing::shown);
        if let Some(shown) = &shown {
            tisty_core::vouched::confirm(self.paths.data(), &who, shown);
            if tisty_core::vouched::confirmed(self.paths.data(), &who)
                .is_some_and(|stood| &stood.key != shown)
            {
                witness::warn(
                    channel::STORE,
                    "this machine signs with a key other than the one it answered for, so the others will turn its history away",
                    &[("at", Fact::Id(who.0.clone()))],
                );
            }
        }
        let starting = shown.is_some() && !self.state.keys.contains_key(&who);
        if starting {
            tisty_core::vouched::confirm_each(self.paths.data(), &self.state.keys);
        }
        if self.state.devices.contains(&who) {
            if let Some(shown) = shown.filter(|_| starting) {
                self.commit(Op::DeviceKey {
                    d: who.clone(),
                    p: shown,
                })?;
            }
        } else {
            self.commit(Op::DeviceJoin {
                d: who.clone(),
                k: Some(tisty_core::DeviceKind::Machine),
                p: shown,
            })?;
        }
        self.say_name()
    }

    /// Kept only while it differs from the system's own name, so a rename back follows the system again.
    pub fn rename(&mut self, name: &str) -> Answer<()> {
        let name = tisty_core::called::cleaned(name);
        let system = tisty_core::called::here().map(|one| one.name);
        // With no system name to fall back to, an empty one would leave the old name standing elsewhere.
        if name.is_empty() && system.is_none() {
            return Ok(());
        }
        let given = (!name.is_empty() && Some(&name) != system.as_ref()).then_some(name);
        self.keep(|config| config.called = given.clone())?;
        Ok(self.say_name()?)
    }

    pub fn say_name(&mut self) -> tisty_core::Result<()> {
        let who = self.config.device_id.clone();
        let now = tisty_core::called::chosen(self.config.called.as_deref());
        match tisty_core::called::told(&self.state, &who, now) {
            Some(named) => self.commit(named),
            None => Ok(()),
        }
    }

    pub fn commit(&mut self, op: Op) -> tisty_core::Result<()> {
        let event = self.store.append(op)?;
        self.writes += 1;
        self.state.apply(&event);
        self.print = self.carry(std::slice::from_ref(&event));
        Ok(())
    }

    pub fn commit_all(&mut self, ops: Vec<Op>) -> tisty_core::Result<()> {
        let events = self.store.append_batch(ops)?;
        self.writes += 1;
        for event in &events {
            self.state.apply(event);
        }
        self.print = self.carry(&events);
        Ok(())
    }

    /// For a writer that knows the store moved underneath: a commit would stamp over what it missed.
    pub fn fell_behind(&mut self) {
        self.behind = true;
    }

    /// A state that missed a commit must not stamp the cache as current, or the projection that
    /// would have caught up loads the gap back.
    pub fn carry(&mut self, events: &[Event]) -> String {
        let cache = match self.behind {
            true => None,
            false => self.cache.as_mut(),
        };
        tisty_core::cache::advance(
            cache,
            &self.state,
            events,
            &self.paths.store(),
            self.store.overtaken(),
        )
    }
}

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

#[cfg(test)]
#[path = "session_test.rs"]
mod tests;
