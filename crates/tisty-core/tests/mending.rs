use tisty_core::{DeviceId, Op, Store, event::ListAdd};

fn list(name: &str) -> Op {
    Op::ListAdd {
        id: ulid::Ulid::generate(),
        d: ListAdd {
            name: name.into(),
            order: "a1".into(),
            color: None,
        },
    }
}

/// What `mend` reaches and what it does not. A cut takes the tail of a file, so the tail is what
/// it answers for; damage further up is a different accident and one that asks for a person.
#[test]
fn a_hole_left_before_the_last_line_is_left_alone_and_said_out_loud() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store
        .append_batch((0..6).map(|n| list(&format!("l{n}"))).collect())
        .unwrap();
    drop(store);

    let active = root.join(&device.0).join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    let torn: String = whole
        .lines()
        .enumerate()
        .map(|(at, line)| match at {
            2 => format!("{}\n", &line[..line.len() / 2]),
            _ => format!("{line}\n"),
        })
        .collect();
    std::fs::write(&active, &torn).unwrap();

    let held = Store::open(&root, device).unwrap();

    assert!(
        held.read_all().is_err(),
        "una linea rota a mitad no se repara sola, y decirlo es mejor que leer media historia"
    );
    assert!(
        !active.with_extension("torn").exists(),
        "y nada se aparta: lo roto no esta al final"
    );
}

/// The lock is what keeps two writers from the same file, and mending is a write like any other:
/// it renames a fresh file over the old one.
#[test]
fn nothing_is_mended_while_another_writer_holds_the_lock() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store.append(list("uno")).unwrap();
    drop(store);

    let held = root.join(&device.0);
    let active = held.join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    let good = whole.lines().next().unwrap().to_string();
    let torn = format!("{good}\n{{\"v\":10,\"ts\":\"2026");
    std::fs::write(&active, &torn).unwrap();

    let guard = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(held.join(".lock"))
        .unwrap();
    guard.try_lock().unwrap();

    Store::open(&root, device.clone()).unwrap();

    assert_eq!(
        std::fs::read_to_string(&active).unwrap(),
        torn,
        "con el candado tomado por otro, el archivo se queda como esta"
    );

    drop(guard);
    Store::open(&root, device).unwrap();

    assert!(
        active.with_extension("torn").exists(),
        "y en cuanto se suelta, la media linea se aparta"
    );
}
