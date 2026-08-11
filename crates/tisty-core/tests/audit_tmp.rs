use tisty_core::event::{DeviceId, LogAdd, Op, StepAdd, TaskAdd};
use tisty_core::{Store, cache, store};
use ulid::Ulid;

#[test]
fn audit_watch_costs() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut s = Store::open(&root, DeviceId("dev_a".into())).unwrap();

    let n = 4_000usize;
    let mut ids = Vec::new();
    for i in 0..n {
        let id = Ulid::generate();
        ids.push(id);
        s.append(Op::TaskAdd {
            id,
            d: TaskAdd::new(format!("tarea numero {i} con un titulo normal"), "a0"),
        })
        .unwrap();
        s.append(Op::TaskLog {
            id,
            d: LogAdd::new(Ulid::generate(), "hablé con el proveedor y quedó en llamar mañana por la tarde"),
        })
        .unwrap();
        s.append(Op::StepAdd {
            id,
            d: StepAdd {
                step: Ulid::generate(),
                text: "revisar el documento adjunto".into(),
                order: "a0".into(),
            },
        })
        .unwrap();
    }

    let events_total = store::read_all(&root).unwrap().len();
    let bytes: u64 = walk(&root);
    eprintln!("AUDIT events={events_total} bytes={bytes}");

    let t = std::time::Instant::now();
    for _ in 0..100 {
        std::hint::black_box(cache::fingerprint(&root));
    }
    eprintln!("AUDIT fingerprint x100 = {:?}", t.elapsed());

    let t = std::time::Instant::now();
    let events = store::read_all(&root).unwrap();
    let read = t.elapsed();
    let t = std::time::Instant::now();
    let state = tisty_core::State::replay(&events);
    let replay = t.elapsed();
    eprintln!(
        "AUDIT read_all={read:?} replay={replay:?} total={:?} tasks={}",
        read + replay,
        state.tasks.len()
    );

    let t = std::time::Instant::now();
    let cloned = state.clone();
    eprintln!("AUDIT clone_state={:?} tasks={}", t.elapsed(), cloned.tasks.len());

    // how many segment files fingerprint has to stat
    let mut segs = 0;
    for d in std::fs::read_dir(&root).unwrap().flatten() {
        if let Ok(files) = std::fs::read_dir(d.path()) {
            segs += files.flatten().filter(|f| f.path().extension().is_some_and(|e| e == "tisty")).count();
        }
    }
    eprintln!("AUDIT segments={segs}");
}

fn walk(at: &std::path::Path) -> u64 {
    let mut total = 0;
    if let Ok(entries) = std::fs::read_dir(at) {
        for e in entries.flatten() {
            let p = e.path();
            total += if p.is_dir() { walk(&p) } else { p.metadata().map(|m| m.len()).unwrap_or(0) };
        }
    }
    total
}
