// What the person does in the window, done here through the core so the assistant's door is tested alone.
#![allow(dead_code)]

use std::path::Path;

use tisty_core::event::{Body, ListAdd, LogAdd, StepAdd, StepRef, TaskPatch};
use tisty_core::{Config, Op, Paths, State, Store, Tag, TaskId};
use ulid::Ulid;

pub fn task(said: &str) -> TaskId {
    said.parse().unwrap()
}

pub struct Person {
    paths: Paths,
}

impl Person {
    pub fn at(root: &Path) -> Self {
        Self {
            paths: Paths::new(root.join("data"), root.join("config")),
        }
    }

    pub fn begins(&self, title: &str) {
        if self.state().lists.is_empty() {
            self.wrote(tisty_core::model::sown("en"));
        }
        self.captures(title);
    }

    fn store(&self) -> Store {
        let config = Config::load_or_init(&self.paths).unwrap();
        Store::open(self.paths.store(), config.device_id.clone())
            .unwrap()
            .signing_with(tisty_core::signing::mine(&self.paths, &config.device_id))
    }

    pub fn state(&self) -> State {
        State::replay(&self.store().read_all().unwrap())
    }

    fn wrote(&self, ops: Vec<Op>) {
        if !ops.is_empty() {
            self.store().append_batch_tagged(ops, false, false).unwrap();
        }
    }

    pub fn lets_the_agent_in(&self) {
        tisty_core::agent::register(&self.paths).unwrap();
    }

    pub fn shuts_the_agent_out(&self) {
        tisty_core::agent::retire(&self.paths).unwrap();
    }

    pub fn captures(&self, text: &str) -> TaskId {
        let read = tisty_nl::parse(text, &jiff::Zoned::now(), "en");
        let plan = tisty_core::capture::plan(&self.state(), tisty_core::capture::Draft::from(read))
            .unwrap();
        self.wrote(plan.ops);
        plan.task
    }

    // A name taken already is refused, as the window refuses it.
    pub fn adds_list(&self, name: &str) {
        let state = self.state();
        if state
            .lists
            .values()
            .any(|one| one.name.eq_ignore_ascii_case(name))
        {
            return;
        }
        let order = state.next_list_order();
        self.wrote(vec![Op::ListAdd {
            id: Ulid::generate(),
            d: ListAdd {
                name: name.to_string(),
                order,
                color: None,
            },
        }]);
    }

    pub fn open(&self) -> String {
        let state = self.state();
        state
            .ordered_open()
            .iter()
            .filter(|task| !task.hidden)
            .map(|task| line_of(task))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn inbox(&self) -> String {
        let state = self.state();
        state
            .inbox()
            .filter(|task| !task.hidden)
            .map(line_of)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn id_of(&self, title: &str) -> TaskId {
        self.state()
            .tasks
            .values()
            .find(|task| task.title == title)
            .unwrap_or_else(|| panic!("no task is called {title:?}"))
            .id
    }

    pub fn card(&self, id: TaskId) -> String {
        let state = self.state();
        let task = &state.tasks[&id];
        let mut said = vec![task.title.clone()];
        said.extend(task.source.clone());
        said.extend(task.description.clone());
        said.extend(task.steps.iter().map(|one| one.text.clone()));
        said.extend(task.log.iter().map(|one| one.body.clone()));
        said.join("\n")
    }

    // What the person wrote for the close in the round goes first, as the window does it.
    pub fn closes(&self, id: TaskId) {
        let mut store = self.store();
        let state = self.state();
        let closing = tisty_core::round::closing_words(&self.paths, &mut store, &state, id, None);
        self.wrote(closing.ops);
        let state = self.state();
        self.wrote(state.completing(id, jiff::Zoned::now()));
        if let Some(said) = &closing.said {
            let state = self.state();
            tisty_core::round::round_cleared(&self.paths, &mut self.store(), &state, id, said);
        }
    }

    pub fn reopens(&self, id: TaskId) {
        let ops = self.state().reopening(id);
        self.wrote(ops);
    }

    pub fn erases(&self, id: TaskId) {
        let ops = tisty_core::task_docs::erased_with(&self.state(), id);
        self.wrote(ops);
    }

    pub fn describes(&self, id: TaskId, body: &str) {
        self.wrote(vec![Op::TaskDescribe {
            id,
            d: Body {
                body: Some(body.to_string()),
            },
        }]);
    }

    pub fn notes(&self, id: TaskId, body: &str) {
        self.wrote(vec![Op::TaskLog {
            id,
            d: LogAdd::new(Ulid::generate(), body.to_string()),
        }]);
    }

    pub fn adds_step(&self, id: TaskId, text: &str) {
        let order = {
            let state = self.state();
            tisty_core::order::last_of(state.tasks[&id].steps.iter().map(|s| s.order.as_str()))
        };
        self.wrote(vec![Op::StepAdd {
            id,
            d: StepAdd {
                step: Ulid::generate(),
                text: text.to_string(),
                order,
            },
        }]);
    }

    pub fn ticks(&self, id: TaskId, place: usize) {
        let step = self.state().tasks[&id].steps[place - 1].id;
        self.wrote(vec![Op::StepDone {
            id,
            d: StepRef { step },
        }]);
    }

    pub fn dates(&self, id: TaskId, day: &str) {
        let date = tisty_nl::parse_date(day, &jiff::Zoned::now(), "en").unwrap();
        self.patches(
            id,
            TaskPatch {
                date: Some(Some(date)),
                ..TaskPatch::default()
            },
        );
    }

    pub fn tags(&self, id: TaskId, tag: &str) {
        let mut tags = self.state().tasks[&id].tags.clone();
        tags.push(Tag::written(tag).unwrap());
        self.patches(
            id,
            TaskPatch {
                tags: Some(tags),
                ..TaskPatch::default()
            },
        );
    }

    pub fn reads_as_a_story(&self, id: TaskId) {
        self.patches(
            id,
            TaskPatch {
                read_as: Some(Some(tisty_core::Reading::Story)),
                ..TaskPatch::default()
            },
        );
    }

    pub fn repeats(&self, id: TaskId, said: &str) {
        let read = tisty_nl::parse(&format!("· {said}"), &jiff::Zoned::now(), "en");
        let over = read
            .repeat
            .unwrap_or_else(|| panic!("{said:?} is not a cadence"));
        self.patches(
            id,
            TaskPatch {
                repeat: Some(Some(over)),
                ..TaskPatch::default()
            },
        );
    }

    // Undo passes over what only settles and what was itself an undo, so the helper does too.
    pub fn takes_back_the_last(&self) {
        let mut store = self.store();
        let events = store.read_all().unwrap();
        let last = events
            .iter()
            .rposition(|one| &one.device == store.device() && !one.undo && !one.op.settles())
            .unwrap();
        let batch = events[last].batch;
        let mut state = State::default();
        let mut inverses = Vec::new();
        for (at, event) in events.iter().enumerate() {
            let wanted = match batch {
                Some(batch) => event.batch == Some(batch),
                None => at == last,
            };
            if wanted {
                inverses.push(tisty_core::inverse(event, &state).unwrap());
            }
            state.apply(event);
        }
        let ops = inverses.into_iter().rev().flatten().collect();
        store.append_batch_tagged(ops, true, false).unwrap();
    }

    fn patches(&self, id: TaskId, d: TaskPatch) {
        self.wrote(vec![Op::TaskUpdate { id, d }]);
    }

    pub fn attaches(&self, id: TaskId, from: &Path) {
        let config = Config::load_or_init(&self.paths).unwrap();
        let kept =
            tisty_core::attach::keep(from, self.paths.data(), config.copies_up_to()).unwrap();
        let named = tisty_core::attach::called(from, None);
        let body = tisty_core::attach::journalled(&kept, &named, from, "kept from {path}");
        self.wrote(vec![
            kept.told(),
            Op::TaskLog {
                id,
                d: LogAdd::new(Ulid::generate(), body),
            },
        ]);
    }
}

impl Person {
    pub fn device(&self) -> String {
        Config::load_or_init(&self.paths).unwrap().device_id.0
    }

    pub fn shares_through(&self, folder: &Path) {
        let mut config = Config::load_or_init(&self.paths).unwrap();
        config.sync = Some(tisty_core::config::Sync::folder(folder));
        config.save(&self.paths).unwrap();
    }

    // The round the window runs, with the seat this machine takes in the place the first time.
    pub fn syncs(&self) -> Result<(), String> {
        let config = Config::load_or_init(&self.paths).unwrap();
        let via = tisty_carrier::chosen(config.sync.as_ref())
            .carrier
            .ok_or("no way of syncing is chosen")?;
        let state = self.state();
        let alive: Vec<String> = state.docs.values().map(|one| one.file.clone()).collect();
        let mut quiet = |_: tisty_carrier::Reached| {};
        let moved = via
            .carry(
                &tisty_carrier::Here::of(&self.paths, &config),
                tisty_carrier::Round {
                    way: tisty_carrier::Way::Both,
                    alive: &alive,
                    holds: config.holds(),
                    saying: &mut quiet,
                },
            )
            .map_err(|trouble| format!("{trouble:?}"))?;
        let mut ops: Vec<Op> = moved
            .let_go
            .iter()
            .map(|at| Op::AttachLetGo { d: at.clone() })
            .collect();
        ops.extend(
            moved
                .took_in
                .iter()
                .map(|(at, sha256, bytes)| Op::AttachKept {
                    d: tisty_core::event::Held {
                        at: at.clone(),
                        sha256: sha256.clone(),
                        bytes: *bytes,
                    },
                }),
        );
        self.wrote(ops);
        if moved.brought > 0 {
            let state = self.state();
            self.wrote(tisty_core::tidy::settling_what_arrived(
                &self.paths,
                &state,
                &moved.arrived,
            ));
        }
        self.seated(&config.device_id);
        Ok(())
    }

    fn seated(&self, who: &tisty_core::DeviceId) {
        let Some(shown) = tisty_core::signing::mine(&self.paths, who)
            .as_ref()
            .map(tisty_core::signing::shown)
        else {
            return;
        };
        tisty_core::vouched::confirm(self.paths.data(), who, &shown);
        let state = self.state();
        let starting = !state.keys.contains_key(who);
        if starting {
            tisty_core::vouched::confirm_each(self.paths.data(), &state.keys);
        }
        let allowed = tisty_core::store::ledger(self.paths.store())
            .unwrap()
            .allowed
            .contains(who);
        match (allowed, starting) {
            (true, true) => self.wrote(vec![Op::DeviceKey {
                d: who.clone(),
                p: shown,
            }]),
            (true, false) => {}
            (false, _) => self.wrote(vec![Op::DeviceJoin {
                d: who.clone(),
                k: Some(tisty_core::DeviceKind::Machine),
                p: Some(shown),
            }]),
        }
    }
}

fn line_of(task: &tisty_core::Task) -> String {
    std::iter::once(task.title.clone())
        .chain(task.tags.iter().map(|tag| format!("#{tag}")))
        .collect::<Vec<_>>()
        .join(" ")
}
