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
    pub fn at(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, Load::Full)
    }

    /// Everything a listing needs, without the journals and steps that make up
    /// most of the bytes. Anything that shows or searches content wants `at`.
    pub fn listing(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, Load::Summary)
    }

    /// One malformed line must not lock the user out of `config`.
    pub fn without_store(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, Load::None)
    }

    fn build(paths: Paths, load: Load) -> tisty_core::Result<Self> {
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

        Ok(Self {
            print: tisty_core::cache::fingerprint(&paths.store()),
            paths,
            state,
            config,
            store,
            cache,
        })
    }

    pub fn locale(&self) -> Option<&str> {
        self.config.locale.as_deref()
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn edit_config(&mut self, f: impl FnOnce(&mut Config)) -> tisty_core::Result<()> {
        f(&mut self.config);
        self.config.save(&self.paths)
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

    /// Whole batch or nothing, and never another device's: half an undone tag
    /// rename leaves the tasks disagreeing.
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

        // Another device's event can sort between two of ours.
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

    /// The change the last undo took back, to apply again as it was. Redo is
    /// not undoing an undo: the inverse of a creation is a deletion, and that
    /// one has no inverse of its own.
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
                // Doing something new is what empties the redo stack everywhere.
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
