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
