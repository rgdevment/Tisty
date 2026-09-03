use tisty_core::event::{DeviceId, TaskAdd, TaskPatch};
use tisty_core::{Op, Store, cache};
use ulid::Ulid;

fn seeded(root: &std::path::Path, many: usize) -> Store {
    let mut store = Store::open(root, DeviceId("dev_a".into())).unwrap();
    for lot in 0..(many / 5_000).max(1) {
        let ops: Vec<Op> = (0..many.min(5_000))
            .map(|n| Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new(format!("t {lot}-{n}"), "a0"),
            })
            .collect();
        store.append_batch(ops).unwrap();
    }
    store
}

#[test]
fn catching_up_lands_where_a_full_replay_would() {
    let room = tempfile::tempdir().unwrap();
    let store_root = room.path().join("store");
    let cache_dir = room.path().join("cache");
    let store = seeded(&store_root, 20_000);
    let _ = cache::project(&store_root, &cache_dir).unwrap();

    let mut outside = Store::open(&store_root, DeviceId("dev_a".into())).unwrap();
    let watched = Ulid::generate();
    outside
        .append(Op::TaskAdd {
            id: watched,
            d: TaskAdd::new("desde el mcp", "a0"),
        })
        .unwrap();
    outside
        .append(Op::TaskDone {
            id: watched,
            filled: false,
        })
        .unwrap();

    let quick = cache::project(&store_root, &cache_dir).unwrap();
    let whole = tisty_core::State::replay(&store.read_all().unwrap());
    assert_eq!(
        quick.tasks, whole.tasks,
        "catching up did not land where a replay does"
    );
    assert!(quick.tasks.contains_key(&watched), "the tail was not taken");
}

#[test]
fn a_rewritten_log_is_replayed_not_caught_up() {
    let room = tempfile::tempdir().unwrap();
    let store_root = room.path().join("store");
    let cache_dir = room.path().join("cache");
    let store = seeded(&store_root, 5_000);
    let _ = cache::project(&store_root, &cache_dir).unwrap();
    drop(store);

    let at = std::fs::read_dir(store_root.join("dev_a"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|e| e == "tisty"))
        .unwrap();
    let mut said = std::fs::read(&at).unwrap();
    said.extend_from_slice(b"{broken}\n");
    std::fs::write(&at, &said).unwrap();

    assert!(
        cache::project(&store_root, &cache_dir).is_err(),
        "corruption slipped through the tail"
    );
}

#[test]
fn a_tail_that_belongs_earlier_is_replayed_not_caught_up() {
    let room = tempfile::tempdir().unwrap();
    let store_root = room.path().join("store");
    let cache_dir = room.path().join("cache");
    let mut store = Store::open(&store_root, DeviceId("dev_a".into())).unwrap();
    let which = Ulid::generate();
    store
        .append(Op::TaskAdd {
            id: which,
            d: TaskAdd::new("primero", "a0"),
        })
        .unwrap();
    store
        .append(Op::TaskUpdate {
            id: which,
            d: TaskPatch {
                title: Some("lo tarde".into()),
                ..Default::default()
            },
        })
        .unwrap();
    let seen = cache::project(&store_root, &cache_dir).unwrap();
    assert_eq!(seen.tasks[&which].title, "lo tarde");

    let apart = room.path().join("apart");
    let mut other = Store::open(&apart, DeviceId("dev_a".into())).unwrap();
    other
        .append(Op::TaskAdd {
            id: which,
            d: TaskAdd::new("x", "a0"),
        })
        .unwrap();
    other
        .append(Op::TaskUpdate {
            id: which,
            d: TaskPatch {
                title: Some("lo temprano".into()),
                ..Default::default()
            },
        })
        .unwrap();
    drop(other);
    drop(store);

    let seg = |root: &std::path::Path| {
        std::fs::read_dir(root.join("dev_a"))
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .find(|p| p.extension().is_some_and(|e| e == "tisty"))
            .unwrap()
    };
    let borrowed = std::fs::read_to_string(seg(&apart)).unwrap();
    let mut one: serde_json::Value =
        serde_json::from_str(borrowed.lines().nth(1).unwrap()).unwrap();
    one["ts"] = serde_json::json!("2001-01-01T00:00:00Z");
    one["n"] = serde_json::json!(1u64);

    let at = seg(&store_root);
    let mut said = std::fs::read_to_string(&at).unwrap();
    said.push_str(&serde_json::to_string(&one).unwrap());
    said.push('\n');
    std::fs::write(&at, said).unwrap();

    let after = cache::project(&store_root, &cache_dir).unwrap();
    assert_eq!(
        after.tasks[&which].title, "lo tarde",
        "an out-of-order tail was applied last instead of in its place"
    );
}

/// Reading the tail means the segments before it are not read at all, and the surest way to see
/// that is to leave one behind that no reader could get through.
#[test]
fn catching_up_does_not_read_the_whole_log_again() {
    let room = tempfile::tempdir().unwrap();
    let store_root = room.path().join("store");
    let cache_dir = room.path().join("cache");
    let mut store = Store::open(&store_root, DeviceId("dev_a".into())).unwrap();
    for n in 0..6_000 {
        store
            .append(Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new(format!("t {n}"), "a0"),
            })
            .unwrap();
    }
    let seen = cache::project(&store_root, &cache_dir).unwrap();

    let mut outside = Store::open(&store_root, DeviceId("dev_a".into())).unwrap();
    outside
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("desde el mcp", "a0"),
        })
        .unwrap();
    drop(outside);
    drop(store);

    let mut sealed: Vec<std::path::PathBuf> = std::fs::read_dir(store_root.join("dev_a"))
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| e == "tisty"))
        .collect();
    sealed.sort();
    let first = sealed.first().unwrap().clone();
    // Same length, so the only thing the fingerprint sees changed is the file that grew.
    let mut said = std::fs::read(&first).unwrap();
    said[..42].copy_from_slice(b"{ not an event at all, not one bit of it }");
    std::fs::write(&first, said).unwrap();

    let after = cache::project(&store_root, &cache_dir).unwrap();
    assert_eq!(
        after.tasks.len(),
        seen.tasks.len() + 1,
        "the tail was not taken on its own: the whole log was read again"
    );
}
