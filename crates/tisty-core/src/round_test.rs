use super::{closing_words, round_cleared};
use crate::event::{DeviceId, Op, TaskAdd};
use crate::{Paths, State, Store};
use ulid::Ulid;

struct Room {
    _tmp: tempfile::TempDir,
    paths: Paths,
    store: Store,
}

fn room() -> Room {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let store = Store::open(paths.store(), DeviceId("dev_a".into())).unwrap();
    Room {
        _tmp: tmp,
        paths,
        store,
    }
}

impl Room {
    fn task(&mut self, routine: bool) -> Ulid {
        let id = Ulid::generate();
        let mut d = TaskAdd::new("Water the plants", "a0");
        if routine {
            d.after = Some(Ulid::generate());
        }
        self.store.append(Op::TaskAdd { id, d }).unwrap();
        id
    }

    fn paged(&mut self, id: Ulid, body: &str) {
        let body = body.to_string();
        crate::task_docs::wrote(&self.paths, &mut self.store, id, |_, _| Some(body)).unwrap();
    }

    fn state(&self) -> State {
        State::replay(&crate::store::read_all(self.paths.store()).unwrap())
    }

    fn page(&self, id: Ulid) -> Option<String> {
        let state = self.state();
        let kept = crate::task_docs::of(&state, crate::task_docs::owner(&state, id))?;
        crate::docs::read(&self.paths.docs(), &kept.file).ok()
    }
}

fn logged(ops: &[Op]) -> Vec<String> {
    ops.iter()
        .filter_map(|op| match op {
            Op::TaskLog { d, .. } => Some(d.body.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_round_logs_how_it_went_with_what_it_wrote_this_time_and_then_lets_it_go() {
    let mut room = room();
    let id = room.task(true);
    room.paged(id, "## Esta vez\n\nSe secó.\n");
    let state = room.state();

    let closing = closing_words(&room.paths, &mut room.store, &state, id, Some("Bien"));

    assert_eq!(logged(&closing.ops), vec!["Bien\n\nSe secó.".to_string()]);
    assert_eq!(closing.said.as_deref(), Some("Se secó."));
    let state = room.state();
    round_cleared(&room.paths, &mut room.store, &state, id, "Se secó.");
    assert_eq!(room.page(id).unwrap(), "## Esta vez\n");
}

#[test]
fn a_round_whose_words_changed_since_they_were_logged_keeps_them() {
    let mut room = room();
    let id = room.task(true);
    room.paged(id, "## Esta vez\n\nSe secó, y la regué.\n");

    let state = room.state();
    round_cleared(&room.paths, &mut room.store, &state, id, "Se secó.");

    assert_eq!(
        room.page(id).unwrap(),
        "## Esta vez\n\nSe secó, y la regué.\n"
    );
}

#[test]
fn a_summary_goes_on_the_page_it_has_or_into_the_log_when_there_is_none() {
    let mut room = room();
    let paged = room.task(false);
    room.paged(paged, "## Al cerrarla\n");
    let bare = room.task(false);
    let state = room.state();

    let onto = closing_words(&room.paths, &mut room.store, &state, paged, Some("Bien."));
    let off = closing_words(&room.paths, &mut room.store, &state, bare, Some("Bien."));

    assert!(onto.ops.is_empty());
    assert_eq!(room.page(paged).unwrap(), "## Al cerrarla\n\nBien.\n");
    assert_eq!(logged(&off.ops), vec!["Bien.".to_string()]);
    assert!(room.page(bare).is_none(), "no page is made only to hold it");
}

#[test]
fn a_locked_page_takes_no_summary_and_the_log_keeps_it() {
    let mut room = room();
    let id = room.task(false);
    room.paged(id, "## Al cerrarla\n");
    let doc = crate::task_docs::of(&room.state(), id).unwrap().id;
    room.store.append(Op::DocLock { id: doc }).unwrap();
    let state = room.state();

    let closing = closing_words(&room.paths, &mut room.store, &state, id, Some("Bien."));

    assert_eq!(logged(&closing.ops), vec!["Bien.".to_string()]);
    assert_eq!(room.page(id).unwrap(), "## Al cerrarla\n");
}

#[test]
fn nothing_said_and_nothing_written_closes_with_nothing_more() {
    let mut room = room();
    let routine = room.task(true);
    let plain = room.task(false);
    let state = room.state();

    assert!(
        closing_words(&room.paths, &mut room.store, &state, routine, None)
            .ops
            .is_empty()
    );
    assert!(
        closing_words(&room.paths, &mut room.store, &state, plain, None)
            .ops
            .is_empty()
    );
}

// A weekly round whose page held a step under This time, closed with its next round open.
fn closed_round(room: &mut Room) -> Ulid {
    let id = Ulid::generate();
    let mut d = TaskAdd::new("Water the plants", "a0");
    d.date = Some(crate::model::DateSpec::all_day(
        jiff::civil::date(2026, 10, 1),
        "UTC",
    ));
    d.repeat = Some(crate::model::Repeat::due(crate::model::Cadence {
        every: 1,
        unit: crate::model::Unit::Week,
    }));
    room.store.append(Op::TaskAdd { id, d }).unwrap();
    for (text, order) in [("water", "a"), ("buy a cake", "b")] {
        room.store
            .append(Op::StepAdd {
                id,
                d: crate::event::StepAdd {
                    step: Ulid::generate(),
                    text: text.into(),
                    order: order.into(),
                },
            })
            .unwrap();
    }
    room.paged(
        id,
        "## Every round\n\n- [ ] water\n\n## This time\n\n- [ ] buy a cake\n",
    );
    let state = room.state();
    let closing = closing_words(&room.paths, &mut room.store, &state, id, None);
    room.store.append_batch(closing.ops).unwrap();
    let ops = room.state().completing(id, jiff::Zoned::now());
    room.store.append_batch(ops).unwrap();
    id
}

fn next_steps(room: &Room, id: Ulid) -> Vec<String> {
    let state = room.state();
    let next = state
        .tasks
        .values()
        .find(|one| one.after == Some(id))
        .unwrap();
    next.steps.iter().map(|one| one.text.clone()).collect()
}

#[test]
fn a_step_written_under_this_time_does_not_come_back_with_the_next_round() {
    let mut room = room();
    let id = closed_round(&mut room);
    let state = room.state();

    round_cleared(&room.paths, &mut room.store, &state, id, "- [ ] buy a cake");

    assert_eq!(next_steps(&room, id), vec!["water"]);
    assert_eq!(
        room.state().tasks[&id].steps.len(),
        2,
        "the closed round keeps its own"
    );
}

#[test]
fn a_page_that_could_not_be_cleared_keeps_the_next_round_s_steps_too() {
    let mut room = room();
    let id = closed_round(&mut room);
    let state = room.state();
    let page = room.page(id).unwrap();
    let originals = room.paths.data().join("originals");
    let _ = std::fs::remove_dir_all(&originals);
    std::fs::write(&originals, b"not a folder").unwrap();

    round_cleared(&room.paths, &mut room.store, &state, id, "- [ ] buy a cake");

    assert_eq!(room.page(id).unwrap(), page);
    let mut left = next_steps(&room, id);
    left.sort();
    assert_eq!(left, vec!["buy a cake", "water"]);
}
