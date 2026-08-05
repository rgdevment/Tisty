use tisty_core::{Config, Event, Op, Paths, State, Store, Task};

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

    pub fn commit(&mut self, op: Op) -> tisty_core::Result<Event> {
        let event = self.store.append(op)?;
        self.state.apply(&event);
        Ok(event)
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
}
