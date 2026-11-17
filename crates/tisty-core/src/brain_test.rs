use std::collections::BTreeMap;

use ulid::Ulid;

use super::*;
use crate::event::{DeviceId, DocAdd, Said, TaskAdd};
use crate::model::Tag;
use crate::{Op, Store, docs};

pub(super) struct World {
    _tmp: tempfile::TempDir,
    pub(super) paths: Paths,
}

impl World {
    pub(super) fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
        Self { _tmp: tmp, paths }
    }

    pub(super) fn store(&self) -> Store {
        Store::open(self.paths.store(), DeviceId("dev_a".into())).unwrap()
    }

    pub(super) fn state(&self) -> State {
        State::replay(&crate::store::read_all(self.paths.store()).unwrap())
    }

    pub(super) fn brain(&self) -> Brain {
        Brain::open(self.paths.cache()).expect("a brain opens")
    }

    pub(super) fn refreshed(&self, brain: &mut Brain, state: &State) -> Refreshed {
        brain
            .refresh(state, &Docs::on_disk(&self.paths.docs()), "seen")
            .unwrap()
    }
}

pub(super) fn task(store: &mut Store, title: &str, tags: &[&str]) -> Ulid {
    let id = Ulid::generate();
    let mut d = TaskAdd::new(title, "a0");
    d.tags = tags.iter().map(|one| Tag::new(one).unwrap()).collect();
    store.append(Op::TaskAdd { id, d }).unwrap();
    id
}

pub(super) fn doc(
    world: &World,
    store: &mut Store,
    body: &str,
    page_of: Option<Ulid>,
    task_of: Option<Ulid>,
) -> (Ulid, String) {
    let made = docs::create(&world.paths.docs(), store.device(), body).unwrap();
    let id = Ulid::generate();
    store
        .append(Op::DocAdd {
            id,
            d: DocAdd {
                wrote: None,
                guest: false,
                sown: false,
                made: None,
                by: None,
                said: Some(Said::of(body)),
                file: made.id.clone(),
                order: "a0".into(),
                folder: None,
                page_of,
                task_of,
                edited: None,
            },
        })
        .unwrap();
    (id, made.id)
}

pub(super) fn held(brain: &Brain, column: &str) -> BTreeMap<String, String> {
    let mut asked = brain
        .db
        .prepare(&format!("SELECT id, {column} FROM node"))
        .unwrap();
    asked
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

pub(super) fn edges(brain: &Brain) -> Vec<(String, String, String)> {
    let mut asked = brain
        .db
        .prepare("SELECT source, target, kind FROM edge ORDER BY source, target, kind")
        .unwrap();
    asked
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn counted(db: &Connection, table: &str) -> i64 {
    db.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
        row.get(0)
    })
    .unwrap()
}

#[test]
fn every_document_page_task_tag_and_file_is_a_node() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "renew the passport", &["errands"]);
    let (book, book_file) = doc(&world, &mut store, "# Trips\n\n#travel\n", None, None);
    let (_, page_file) = doc(&world, &mut store, "# Lisbon\n", Some(book), None);
    let (_, words_file) = doc(&world, &mut store, "# Notes\n", None, Some(errand));
    let mut state = world.state();
    state
        .kept
        .insert("attachments/map-1a2b.png".into(), ("ab12".into(), 3));

    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let kinds = held(&brain, "kind");
    let kind_of = |id: &str| kinds.get(id).map(String::as_str);
    assert_eq!(kind_of(&book_file), Some("doc"));
    assert_eq!(kind_of(&page_file), Some("page"));
    assert_eq!(kind_of(&errand.to_string()), Some("task"));
    assert_eq!(kind_of("#errands"), Some("tag"));
    assert_eq!(kind_of("#travel"), Some("tag"));
    assert_eq!(kind_of("attachments/map-1a2b.png"), Some("file"));
    assert_eq!(
        kind_of(&words_file),
        None,
        "a task's words belong to the task"
    );
    assert_eq!(
        held(&brain, "title").get(&book_file).map(String::as_str),
        Some("Trips")
    );
}

#[test]
fn a_hidden_task_is_still_a_node_for_the_window_to_show() {
    let world = World::new();
    let mut store = world.store();
    let kept_out = task(&mut store, "a surprise party", &[]);
    let mut state = world.state();
    if let Some(one) = state.tasks.get_mut(&kept_out) {
        one.hidden = true;
    }

    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    assert!(held(&brain, "kind").contains_key(&kept_out.to_string()));
}

#[test]
fn only_what_moved_since_the_last_round_is_written_again() {
    let world = World::new();
    let mut store = world.store();
    let first = task(&mut store, "call the bank", &[]);
    task(&mut store, "water the plants", &[]);
    let mut state = world.state();
    let mut brain = world.brain();

    assert_eq!(
        world.refreshed(&mut brain, &state),
        Refreshed::Moved {
            written: 2,
            removed: 0
        }
    );
    assert_eq!(world.refreshed(&mut brain, &state), Refreshed::Unchanged);

    if let Some(one) = state.tasks.get_mut(&first) {
        one.title = "call the bank about the card".into();
    }
    assert_eq!(
        world.refreshed(&mut brain, &state),
        Refreshed::Moved {
            written: 1,
            removed: 0
        }
    );
    assert_eq!(
        held(&brain, "title")
            .get(&first.to_string())
            .map(String::as_str),
        Some("call the bank about the card")
    );
}

#[test]
fn a_word_in_a_task_s_description_moves_it() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "file the taxes", &[]);
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    if let Some(one) = state.tasks.get_mut(&errand) {
        one.description = Some("the receipts are in the blue folder".into());
    }

    assert_eq!(
        world.refreshed(&mut brain, &state),
        Refreshed::Moved {
            written: 1,
            removed: 0
        }
    );
}

#[test]
fn what_is_gone_leaves_with_what_it_wrote() {
    let world = World::new();
    let mut store = world.store();
    let gone = task(&mut store, "book the flight", &[]);
    let stays = task(&mut store, "pack", &[]);
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    let (gone_id, stays_id) = (gone.to_string(), stays.to_string());
    brain
        .db
        .execute(
            "INSERT INTO edge VALUES (?1, ?2, 'mention'), (?2, ?1, 'mention')",
            [&gone_id, &stays_id],
        )
        .unwrap();
    brain
        .db
        .execute("INSERT INTO card VALUES (?1, 's', '{}')", [&gone_id])
        .unwrap();
    brain
        .db
        .execute("INSERT INTO note VALUES (?1, '{}')", [&gone_id])
        .unwrap();

    state.tasks.remove(&gone);

    assert_eq!(
        world.refreshed(&mut brain, &state),
        Refreshed::Moved {
            written: 0,
            removed: 1
        }
    );
    assert!(!held(&brain, "kind").contains_key(&gone_id));
    for table in ["card", "note"] {
        assert_eq!(counted(&brain.db, table), 0, "{table} still holds it");
    }
    assert_eq!(
        edges(&brain),
        vec![(stays_id, gone_id, "mention".to_string())],
        "only what the gone task wrote leaves with it"
    );
}

#[test]
fn a_body_written_without_an_event_moves_its_document() {
    let world = World::new();
    let mut store = world.store();
    let (_, file) = doc(&world, &mut store, "# Trips\n", None, None);
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let at = docs::resolve(&world.paths.docs(), &file).unwrap();
    std::fs::write(at, "# Trips\n\nSomeone else wrote this.\n").unwrap();

    assert_eq!(
        world.refreshed(&mut brain, &state),
        Refreshed::Moved {
            written: 1,
            removed: 0
        }
    );
}

#[test]
fn an_older_base_is_built_again_from_nothing() {
    let dir = tempfile::tempdir().unwrap();
    {
        let brain = Brain::open(dir.path()).unwrap();
        brain
            .db
            .execute_batch(
                "INSERT INTO node (id, kind, title, stamp) VALUES ('x', 'task', 'old', 's');
                 UPDATE meta SET value = '0' WHERE key = 'schema';",
            )
            .unwrap();
    }

    let brain = Brain::open(dir.path()).expect("an older base opens");

    assert_eq!(counted(&brain.db, "node"), 0);
    assert_eq!(said(&brain.db, "schema").unwrap(), Some(SCHEMA.to_string()));
}

#[test]
fn a_newer_base_is_left_to_the_tisty_that_keeps_it() {
    let dir = tempfile::tempdir().unwrap();
    {
        let brain = Brain::open(dir.path()).unwrap();
        brain
            .db
            .execute(
                "INSERT INTO node (id, kind, title, stamp) VALUES ('x', 'task', 'newer', 's')",
                [],
            )
            .unwrap();
        brain
            .db
            .execute(
                "UPDATE meta SET value = ? WHERE key = 'schema'",
                [(SCHEMA + 1).to_string()],
            )
            .unwrap();
    }

    assert!(Brain::open(dir.path()).is_none());

    let raw = Connection::open(dir.path().join(FILE)).unwrap();
    assert_eq!(counted(&raw, "node"), 1, "a newer base keeps what it held");
}

#[test]
fn a_file_that_is_no_database_is_built_again() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join(FILE), vec![b'A'; 4096]).unwrap();

    let mut brain = Brain::open(dir.path()).expect("built again");

    assert_eq!(
        brain
            .refresh(&State::default(), &Docs::on_disk(dir.path()), "seen")
            .unwrap(),
        Refreshed::Unchanged
    );
}

#[test]
fn damage_deep_in_the_file_is_found_at_the_start_of_a_run() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut brain = Brain::open(dir.path()).unwrap();
        let mut state = State::default();
        crowded(&mut state, 400);
        brain
            .refresh(&state, &Docs::on_disk(dir.path()), "seen")
            .unwrap();
        brain
            .db
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE)")
            .unwrap();
    }
    let at = dir.path().join(FILE);
    let mut bytes = std::fs::read(&at).unwrap();
    let middle = bytes.len() / 2;
    bytes[middle..middle + 2048].fill(0xFF);
    std::fs::write(&at, bytes).unwrap();

    let brain = Brain::checked(dir.path()).expect("built again");

    assert!(brain.whole());
    assert_eq!(counted(&brain.db, "node"), 0);
}

#[test]
fn two_processes_keep_one_base_between_them() {
    let world = World::new();
    let mut store = world.store();
    task(&mut store, "renew the passport", &[]);
    let state = world.state();
    let mut window = world.brain();
    let mut agent = world.brain();

    assert!(matches!(
        world.refreshed(&mut window, &state),
        Refreshed::Moved { .. }
    ));
    assert_eq!(world.refreshed(&mut agent, &state), Refreshed::Unchanged);
}

#[test]
fn a_look_trusts_the_stamp_and_a_nudge_reads_the_store_again() {
    let world = World::new();
    let filed = |title: &str| {
        let mut store = world.store();
        task(&mut store, title, &[]);
    };
    filed("renew the passport");

    assert!(matches!(
        kept_up(&world.paths, Round::Starting),
        Some(Refreshed::Moved { written: 1, .. })
    ));
    assert_eq!(
        kept_up(&world.paths, Round::Looking),
        Some(Refreshed::Unchanged)
    );

    filed("water the plants");

    assert_eq!(
        kept_up(&world.paths, Round::Nudged),
        Some(Refreshed::Moved {
            written: 1,
            removed: 0
        })
    );
}

#[test]
fn room_left_by_what_was_removed_is_given_back() {
    let dir = tempfile::tempdir().unwrap();
    let mut brain = Brain::open(dir.path()).unwrap();
    let mut state = State::default();
    crowded(&mut state, 3000);
    brain
        .refresh(&state, &Docs::on_disk(dir.path()), "full")
        .unwrap();

    brain
        .refresh(&State::default(), &Docs::on_disk(dir.path()), "empty")
        .unwrap();
    brain.compact_if_worth_it();

    let pages: i64 = brain
        .db
        .query_row("PRAGMA page_count", [], |row| row.get(0))
        .unwrap();
    assert!(pages < PAGES_WORTH_COMPACTING, "still {pages} pages");
}

fn crowded(state: &mut State, how_many: usize) {
    for n in 0..how_many {
        let id = Ulid::generate();
        let mut one = sample_task();
        one.id = id;
        one.title = format!("{n} {}", "a long title to fill the pages ".repeat(16));
        state.tasks.insert(id, one);
    }
}

fn sample_task() -> crate::Task {
    let id = Ulid::generate();
    let mut state = State::default();
    state.apply(&crate::Event::new(
        DeviceId("dev_a".into()),
        jiff::Timestamp::from_second(1).unwrap(),
        Op::TaskAdd {
            id,
            d: TaskAdd::new("sample", "a0"),
        },
    ));
    state.tasks.remove(&id).unwrap()
}

#[test]
fn a_swept_store_is_not_put_back() {
    let world = World::new();

    assert_eq!(kept_up(&world.paths, Round::Nudged), None);
    assert!(
        !world.paths.cache().exists(),
        "the round made the cache again"
    );
}

#[test]
fn a_damaged_base_leaves_no_journal_behind() {
    let dir = tempfile::tempdir().unwrap();
    for tail in ["", "-wal", "-shm"] {
        std::fs::write(dir.path().join(format!("{FILE}{tail}")), b"left").unwrap();
    }

    assert!(forget(dir.path()));

    for tail in ["", "-wal", "-shm"] {
        assert!(!dir.path().join(format!("{FILE}{tail}")).exists());
    }
}
