use tisty_core::witness::{self, Fact, channel};
use tisty_core::{Config, Event, Op, Paths, State, Store, Task, order};

enum Load {
    Full,
    Summary,
    None,
}

pub struct App {
    pub paths: Paths,
    pub state: State,
    config: Config,
    store: Store,
    cache: Option<tisty_core::cache::Cache>,
    print: String,
}

impl App {
    pub fn device(&self) -> &tisty_core::event::DeviceId {
        self.store.device()
    }

    pub fn at(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, Load::Full)
    }

    pub fn listing(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, Load::Summary)
    }

    pub fn without_store(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, Load::None)
    }

    fn build(paths: Paths, load: Load) -> tisty_core::Result<Self> {
        let clean = !paths.config_file().exists();
        let config = Config::load_or_init(&paths)?;
        let store = Store::open(paths.store(), config.device_id.clone())?;
        let state = match load {
            Load::Full => tisty_core::cache::project(&paths.store(), paths.cache())?,
            Load::Summary => tisty_core::cache::summarised(&paths.store(), paths.cache())?,
            Load::None => State::default(),
        };

        let cache = if matches!(load, Load::Full | Load::Summary) {
            tisty_core::cache::Cache::open(paths.cache())?
        } else {
            None
        };

        let mut app = Self {
            print: tisty_core::cache::fingerprint(&paths.store()),
            paths,
            state,
            config,
            store,
            cache,
        };
        if clean && matches!(load, Load::Full | Load::Summary) && app.state.lists.is_empty() {
            let code = tisty_core::model::spoken(app.config.locale.as_deref());
            let _ = app.commit_all(tisty_core::model::sown(&code));
        }
        Ok(app)
    }

    pub fn locale(&self) -> Option<&str> {
        self.config.locale.as_deref()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn edit_config(&mut self, f: impl FnOnce(&mut Config)) -> tisty_core::Result<()> {
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
        f(&mut fresh);
        fresh.save(&self.paths)?;
        self.config = fresh;
        Ok(())
    }

    pub fn tidy_up(&mut self, bin: bool) {
        let dest = match self.config.sync.clone() {
            Some(tisty_core::config::Sync::Folder(at)) => Some(at),
            _ => None,
        };
        tisty_core::tidy::all_of_it(
            &self.paths,
            &self.state,
            self.cache.as_ref(),
            dest.as_deref(),
            bin,
        );
    }

    pub fn copies_up_to(&self) -> u64 {
        self.config.copies_up_to()
    }

    pub fn copies_in_a_doc(&self) -> u64 {
        self.config.copies_in_a_doc()
    }

    pub fn commit(&mut self, op: Op) -> tisty_core::Result<Event> {
        let event = self.store.append(op)?;
        self.state.apply(&event);
        self.refresh(std::slice::from_ref(&event));
        Ok(event)
    }

    pub fn commit_all(&mut self, ops: Vec<Op>) -> tisty_core::Result<usize> {
        self.commit_marked(ops, false, false)
    }

    pub fn commit_undo(&mut self, ops: Vec<Op>) -> tisty_core::Result<usize> {
        self.commit_marked(ops, true, false)
    }

    pub fn commit_redo(&mut self, ops: Vec<Op>) -> tisty_core::Result<usize> {
        self.commit_marked(ops, false, true)
    }

    fn commit_marked(&mut self, ops: Vec<Op>, undo: bool, redo: bool) -> tisty_core::Result<usize> {
        let events = self.store.append_batch_tagged(ops, undo, redo)?;
        for event in &events {
            self.state.apply(event);
        }
        self.refresh(&events);
        Ok(events.len())
    }

    fn refresh(&mut self, events: &[Event]) {
        self.print = tisty_core::cache::advance(
            self.cache.as_mut(),
            &self.state,
            events,
            &self.paths.store(),
            self.store.overtaken(),
        );
    }

    pub fn last_own_change(&self) -> tisty_core::Result<Vec<(Event, State)>> {
        self.reachable_change(false)
    }

    fn reachable_change(&self, want_undo: bool) -> tisty_core::Result<Vec<(Event, State)>> {
        let events = self.store.read_all()?;
        let mine: Vec<usize> = events
            .iter()
            .enumerate()
            .filter(|(_, e)| &e.device == self.store.device())
            .map(|(i, _)| i)
            .collect();

        let mut cursor = mine.len();
        let mut already_undone = 0usize;
        let last = loop {
            let Some(&i) = cursor.checked_sub(1).and_then(|c| mine.get(c)) else {
                return Ok(Vec::new());
            };
            let step = match events[i].batch {
                Some(batch) => mine
                    .iter()
                    .filter(|&&j| events[j].batch == Some(batch))
                    .count(),
                None => 1,
            };
            cursor -= step;

            let settling = match events[i].batch {
                Some(batch) => mine
                    .iter()
                    .filter(|&&j| events[j].batch == Some(batch))
                    .all(|&j| events[j].op.settles()),
                None => events[i].op.settles(),
            };
            if settling {
                continue;
            }

            if events[i].undo == want_undo {
                if already_undone == 0 {
                    break i;
                }
                already_undone -= 1;
            } else {
                already_undone += 1;
            }
        };

        let wanted: Vec<usize> = match events[last].batch {
            None => vec![last],
            Some(batch) => events
                .iter()
                .enumerate()
                .filter(|(_, e)| e.batch == Some(batch))
                .map(|(i, _)| i)
                .collect(),
        };

        let mut state = State::default();
        let mut found = Vec::with_capacity(wanted.len());
        for (i, event) in events.iter().enumerate() {
            if wanted.contains(&i) {
                found.push((event.clone(), state.clone()));
            }
            state.apply(event);
        }
        Ok(found)
    }

    pub fn last_undone_change(&self) -> tisty_core::Result<Vec<Event>> {
        let events = self.store.read_all()?;
        let mut live: Vec<Vec<Event>> = Vec::new();
        let mut undone: Vec<Vec<Event>> = Vec::new();

        for group in self.own_changes(&events) {
            if group[0].undo {
                if let Some(taken) = live.pop() {
                    undone.push(taken);
                }
            } else if group[0].redo {
                undone.pop();
                live.push(group);
            } else {
                live.push(group);
                undone.clear();
            }
        }
        Ok(undone.pop().unwrap_or_default())
    }

    fn own_changes(&self, events: &[Event]) -> Vec<Vec<Event>> {
        let mut groups: Vec<Vec<Event>> = Vec::new();
        for event in events.iter().filter(|e| &e.device == self.store.device()) {
            match groups.last_mut() {
                Some(last)
                    if event.batch.is_some()
                        && last.first().map(|e| e.batch) == Some(event.batch) =>
                {
                    last.push(event.clone());
                }
                _ => groups.push(vec![event.clone()]),
            }
        }
        groups
    }

    pub fn next_step_order(&self, task: &Task) -> String {
        order::last_of(task.steps.iter().map(|s| s.order.as_str()))
    }
}

#[cfg(test)]
mod undoing {
    use super::App;
    use tisty_core::{Op, Paths, event::Filed, order};
    use ulid::Ulid;

    fn desk() -> (tempfile::TempDir, Paths) {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        (tmp, paths)
    }

    #[test]
    fn settling_a_book_is_not_what_undo_reaches_for() {
        let (_tmp, paths) = desk();
        let mut app = App::at(paths).unwrap();

        let list = Ulid::generate();
        app.commit(Op::ListAdd {
            id: list,
            d: tisty_core::event::ListAdd {
                name: "Casa".into(),
                order: order::first(),
                color: None,
            },
        })
        .unwrap();

        let book = Ulid::generate();
        let pages: Vec<Ulid> = (0..2).map(|_| Ulid::generate()).collect();
        app.commit(Op::DocAdd {
            id: book,
            d: tisty_core::event::DocAdd {
                said: None,
                file: "dev_a-0001".into(),
                order: order::first(),
                folder: None,
                page_of: None,
            },
        })
        .unwrap();
        for (n, id) in pages.iter().enumerate() {
            app.commit(Op::DocAdd {
                id: *id,
                d: tisty_core::event::DocAdd {
                    said: None,
                    file: format!("dev_a-000{}", n + 2),
                    order: order::first(),
                    folder: None,
                    page_of: Some(book),
                },
            })
            .unwrap();
        }

        app.commit_all(
            pages
                .iter()
                .map(|id| Op::DocMove {
                    id: *id,
                    d: Filed {
                        folder: None,
                        page_of: None,
                        order: Some(order::first()),
                    },
                })
                .collect(),
        )
        .unwrap();

        let reached = app.last_own_change().unwrap();
        assert_eq!(reached.len(), 1, "one event, not the settling batch");
        assert!(
            matches!(reached[0].0.op, Op::DocAdd { .. }),
            "undo reaches the last thing the person did, not the order it settled: {:?}",
            reached[0].0.op
        );
    }
}
