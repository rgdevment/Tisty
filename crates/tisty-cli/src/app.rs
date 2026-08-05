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
        let events = self.store.append_batch(ops)?;
        for event in &events {
            self.state.apply(event);
        }
        Ok(events.len())
    }

    /// The last change made here, event by event, each paired with the state it
    /// was applied to. A change that took several events comes back whole, or
    /// undoing a tag rename would leave half the tasks renamed.
    ///
    /// Another device's events are left alone: undoing them from afar would
    /// surprise whoever is sitting in front of that machine.
    pub fn last_own_change(&self) -> tisty_core::Result<Vec<(Event, State)>> {
        let events = self.store.read_all()?;
        let Some(last) = events
            .iter()
            .rposition(|e| &e.device == self.store.device())
        else {
            return Ok(Vec::new());
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

        // Another device's event can sort between two of ours, so each one is
        // inverted against the state it actually saw.
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

    /// Lists are few and named by hand, so a case-insensitive substring is
    /// enough; an exact name always wins over a partial one.
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

/// Listings print «Mi Lista» as `#mi-lista`, so what is on screen has to be
/// what can be typed back.
fn loose(name: &str) -> String {
    name.to_lowercase().replace([' ', '_'], "-")
}
