use std::path::{Path, PathBuf};

use tisty_core::answering::{Reached, answers};
use tisty_core::event::{SCHEMA_VERSION, SIGNED_FROM, TaskAdd};
use tisty_core::store::{key_said_in, newest_schema, read_all, written_since};
use tisty_core::{DeviceId, Op, State, Status, Store, signing};
use ulid::Ulid;

const WRITTEN_BY: [(u32, &str); 2] = [(15, "1.23.1"), (16, "1.24.4")];

struct Held {
    _room: tempfile::TempDir,
    root: PathBuf,
    who: DeviceId,
}

impl Held {
    fn dir(&self) -> PathBuf {
        self.root.join(&self.who.0)
    }
}

fn copied(from: &Path, into: &Path) {
    std::fs::create_dir_all(into).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let to = into.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copied(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), to).unwrap();
        }
    }
}

fn held(schema: u32) -> Held {
    let from = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/stores")
        .join(format!("v{schema}"));
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    copied(&from, &root);
    let who = std::fs::read_dir(&root)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|one| one.path().is_dir())
        .find_map(|one| {
            let name = one.file_name().into_string().ok()?;
            name.starts_with("dev_").then_some(DeviceId(name))
        })
        .expect("the machine's own directory");
    Held {
        _room: room,
        root,
        who,
    }
}

fn titled(state: &State, status: Status) -> Vec<String> {
    let mut found: Vec<String> = state
        .tasks
        .values()
        .filter(|task| task.status == status)
        .map(|task| task.title.clone())
        .collect();
    found.sort();
    found
}

#[test]
fn every_event_an_earlier_release_wrote_is_read_back_as_it_was_meant() {
    for (schema, release) in WRITTEN_BY {
        let store = held(schema);

        let events = read_all(&store.root).unwrap();
        let state = State::replay(&events);

        assert_eq!(events.len(), 6, "{release} wrote six events");
        assert_eq!(
            titled(&state, Status::Open),
            ["call the bank", "renew the certificate", "write the report"],
            "{release}"
        );
        assert_eq!(titled(&state, Status::Done), ["buy bread"], "{release}");
    }
}

#[test]
fn the_schema_an_earlier_release_wrote_under_is_the_one_its_history_says() {
    for (schema, release) in WRITTEN_BY {
        let store = held(schema);

        assert_eq!(newest_schema(&store.dir()).unwrap(), schema, "{release}");
        assert_eq!(
            written_since(&store.dir(), SIGNED_FROM).unwrap(),
            schema >= SIGNED_FROM,
            "{release}: whether its history owes a signature"
        );
    }
}

#[test]
fn a_history_an_earlier_release_signed_still_answers_for_itself() {
    let store = held(16);
    let said = key_said_in(&store.dir(), &store.who).expect("the key the join carries");
    let key = signing::read(&said).expect("a key that reads");

    let reached = answers(&store.dir(), &store.who, &key, Reached::default(), &|_| {
        false
    })
    .expect("a signature 1.24.4 wrote did not answer");

    assert!(reached.signing);
}

#[test]
fn a_store_from_before_signing_takes_what_this_build_writes_next() {
    let store = held(15);
    let mut open = Store::open(&store.root, store.who.clone()).unwrap();

    open.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: TaskAdd::new("written by this build", "Z"),
    })
    .unwrap();
    drop(open);

    let events = read_all(&store.root).unwrap();
    assert_eq!(events.len(), 7);
    assert!(
        titled(&State::replay(&events), Status::Open)
            .contains(&"written by this build".to_string())
    );
    assert_eq!(newest_schema(&store.dir()).unwrap(), SCHEMA_VERSION);
}
