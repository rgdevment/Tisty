use tisty_core::{Config, Event, List, ListId, Op, Paths, State, Store, Task, order};

pub struct App {
    pub paths: Paths,
    pub state: State,
    config: Config,
    store: Store,
}

impl App {
    pub fn open() -> tisty_core::Result<Self> {
        Self::at(Paths::resolve()?)
    }

    pub fn at(paths: Paths) -> tisty_core::Result<Self> {
        let config = Config::load_or_init(&paths)?;
        let store = Store::open(paths.store(), config.device_id.clone())?;
        let state = State::replay(&store.read_all()?);

        Ok(Self {
            paths,
            state,
            config,
            store,
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
        Ok(event)
    }

    pub fn commit_all(&mut self, ops: Vec<Op>) -> tisty_core::Result<usize> {
        self.commit_marked(ops, false)
    }

    pub fn commit_undo(&mut self, ops: Vec<Op>) -> tisty_core::Result<usize> {
        self.commit_marked(ops, true)
    }

    fn commit_marked(&mut self, ops: Vec<Op>, undo: bool) -> tisty_core::Result<usize> {
        let events = self.store.append_batch_marked(ops, undo)?;
        for event in &events {
            self.state.apply(event);
        }
        Ok(events.len())
    }

    /// Whole batch or nothing, and never another device's: half an undone tag
    /// rename leaves the tasks disagreeing.
    pub fn last_own_change(&self) -> tisty_core::Result<Vec<(Event, State)>> {
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

            if events[i].undo {
                already_undone += 1;
            } else if already_undone > 0 {
                already_undone -= 1;
            } else {
                break i;
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

    pub fn list_id(&self, name: &str) -> Option<ListId> {
        self.find_list(name).first().map(|l| l.id)
    }
}

/// Listings print «Mi Lista» as `#mi-lista`, which has to be typeable back.
fn loose(name: &str) -> String {
    name.to_lowercase().replace([' ', '_'], "-")
}
