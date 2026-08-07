use tisty_core::{Config, Event, List, Op, Paths, State, Store, Task, order};

pub struct App {
    pub paths: Paths,
    pub state: State,
    config: Config,
    store: Store,
    cache: Option<tisty_core::cache::Cache>,
}

impl App {
    pub fn at(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, true)
    }

    /// One malformed line must not lock the user out of `config`.
    pub fn without_store(paths: Paths) -> tisty_core::Result<Self> {
        Self::build(paths, false)
    }

    fn build(paths: Paths, replay: bool) -> tisty_core::Result<Self> {
        let config = Config::load_or_init(&paths)?;
        let store = Store::open(paths.store(), config.device_id.clone())?;
        let state = if replay {
            tisty_core::cache::project(&paths.store(), paths.cache())?
        } else {
            State::default()
        };

        let cache = if replay {
            tisty_core::cache::Cache::open(paths.cache())?
        } else {
            None
        };

        Ok(Self {
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

    /// Carrying the cache forward instead of letting the next read rebuild it:
    /// a CLI writes and reads in the same breath, so invalidating on every
    /// write leaves the cache never warm.
    fn refresh(&mut self, events: &[Event]) {
        let Some(cache) = &mut self.cache else { return };
        if events.iter().any(|e| matches!(e.op, Op::ListDelete { .. })) {
            cache.invalidate();
            return;
        }

        let print = tisty_core::cache::fingerprint(&self.paths.store());
        for event in events {
            let _ = cache.touch(&self.state, event.entity_id(), &print);
        }
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

    pub fn ordered_open(&self) -> Vec<&Task> {
        let mut tasks: Vec<_> = self.state.open_tasks().collect();
        tasks.sort_by(|a, b| {
            let key = |t: &Task| {
                (
                    t.date.as_ref().map(|d| d.at),
                    t.priority,
                    t.order.clone(),
                    t.id,
                )
            };
            match (&a.date, &b.date) {
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                _ => key(a).cmp(&key(b)),
            }
        });
        tasks
    }

    pub fn ordered_lists(&self) -> Vec<&List> {
        let mut lists: Vec<_> = self.state.active_lists().collect();
        lists.sort_by(|a, b| (&a.order, a.id).cmp(&(&b.order, b.id)));
        lists
    }

    /// Case-insensitive substring, but an exact name always wins.
    pub fn find_list(&self, needle: &str) -> Vec<&List> {
        let needle = loose(needle);
        let exact: Vec<&List> = self
            .state
            .lists
            .values()
            .filter(|l| loose(&l.name) == needle)
            .collect();
        if !exact.is_empty() {
            return exact;
        }
        self.state
            .lists
            .values()
            .filter(|l| loose(&l.name).contains(&needle))
            .collect()
    }

    pub fn next_task_order(&self) -> String {
        order::last_of(self.state.tasks.values().map(|t| t.order.as_str()))
    }

    pub fn next_list_order(&self) -> String {
        order::last_of(self.state.lists.values().map(|l| l.order.as_str()))
    }

    pub fn next_step_order(&self, task: &Task) -> String {
        order::last_of(task.steps.iter().map(|s| s.order.as_str()))
    }
}

/// Listings print «Mi Lista» as `#mi-lista`, which has to be typeable back.
fn loose(name: &str) -> String {
    name.to_lowercase().replace([' ', '_'], "-")
}
