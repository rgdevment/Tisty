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

#[test]
fn a_line_cut_short_by_a_power_cut_never_takes_the_whole_store_with_it() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    for named in ["uno", "dos", "tres"] {
        store.append(list(named)).unwrap();
    }
    drop(store);

    let active = root.join(&device.0).join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    let cut = whole.len() - 40;
    std::fs::write(&active, &whole[..cut]).unwrap();

    let held = Store::open(&root, device).unwrap();
    let events = held.read_all().unwrap();

    assert_eq!(events.len(), 2, "los eventos enteros tienen que sobrevivir");
}

#[test]
fn the_half_line_is_kept_where_somebody_could_look_at_it() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store.append(list("uno")).unwrap();
    store.append(list("dos")).unwrap();
    drop(store);

    let active = root.join(&device.0).join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    std::fs::write(&active, &whole[..whole.len() - 30]).unwrap();

    Store::open(&root, device.clone()).unwrap();

    let aside = root.join(&device.0).join("active.torn");
    assert!(
        aside.is_file(),
        "la mitad que se apartó tiene que quedar a la vista"
    );
    assert!(!std::fs::read_to_string(&aside).unwrap().is_empty());
}

#[test]
fn a_whole_last_line_missing_only_its_newline_is_still_an_event() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store.append(list("uno")).unwrap();
    store.append(list("dos")).unwrap();
    drop(store);

    let active = root.join(&device.0).join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    std::fs::write(&active, whole.trim_end()).unwrap();

    let held = Store::open(&root, device).unwrap();

    assert_eq!(
        held.read_all().unwrap().len(),
        2,
        "no se descarta un evento entero"
    );
}

#[test]
fn a_sealed_segment_is_never_mended_however_broken_it_looks() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store.append(list("uno")).unwrap();
    drop(store);

    let dir = root.join(&device.0);
    std::fs::rename(dir.join("active.tisty"), dir.join("000001.tisty")).unwrap();
    let whole = std::fs::read_to_string(dir.join("000001.tisty")).unwrap();
    std::fs::write(dir.join("000001.tisty"), &whole[..whole.len() - 30]).unwrap();

    let held = Store::open(&root, device).unwrap();

    assert!(
        held.read_all().is_err(),
        "un segmento sellado tiene su cuenta que responder: no se toca"
    );
}

#[test]
fn a_batch_that_crosses_a_segment_keeps_every_event_and_seals_what_it_left() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    let many: Vec<Op> = (0..5_001).map(|n| list(&format!("lista {n}"))).collect();
    store.append_batch(many).unwrap();
    drop(store);

    let held = Store::open(&root, device.clone()).unwrap();
    assert_eq!(held.read_all().unwrap().len(), 5_001);

    let dir = root.join(&device.0);
    assert!(
        dir.join("000001.tisty").is_file(),
        "el segmento lleno se selló"
    );
    assert!(dir.join("000001.count").is_file(), "y quedó con su cuenta");
}

#[test]
fn a_first_line_cut_short_never_leaves_the_store_unreadable() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store.append(list("uno")).unwrap();
    drop(store);

    let active = root.join(&device.0).join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    std::fs::write(&active, &whole[..whole.len() - 30]).unwrap();

    let held = Store::open(&root, device).unwrap();

    assert!(
        held.read_all().is_ok(),
        "un corte de luz en el primer evento deja la historia ilegible: {:?}",
        held.read_all().err()
    );
    assert!(
        root.join("dev_a3f9").join("active.torn").is_file(),
        "la mitad tenia que apartarse"
    );
}

#[test]
fn a_last_line_without_its_newline_never_swallows_the_next_event() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store.append(list("uno")).unwrap();
    store.append(list("dos")).unwrap();
    drop(store);

    let active = root.join(&device.0).join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    std::fs::write(&active, whole.trim_end()).unwrap();

    let mut held = Store::open(&root, device.clone()).unwrap();
    held.append(list("tres")).unwrap();
    drop(held);

    let after = Store::open(&root, device).unwrap();
    let read = after.read_all();
    assert!(
        read.is_ok(),
        "el evento nuevo se pego al anterior y la historia ya no se lee: {:?}",
        read.err()
    );
    assert_eq!(read.unwrap().len(), 3);
}

#[test]
fn a_torn_tail_that_is_not_text_is_mended_all_the_same() {
    let room = tempfile::tempdir().unwrap();
    let root = room.path().join("store");
    let device = DeviceId("dev_a3f9".into());

    let mut store = Store::open(&root, device.clone()).unwrap();
    store.append(list("uno")).unwrap();
    store.append(list("dos")).unwrap();
    drop(store);

    let active = root.join(&device.0).join("active.tisty");
    let whole = std::fs::read(&active).unwrap();
    let cut = whole.len() - 30;
    let mut torn = whole[..cut].to_vec();
    torn.push(0xC3);
    std::fs::write(&active, &torn).unwrap();

    let held = Store::open(&root, device).unwrap();

    assert!(
        held.read_all().is_ok(),
        "media linea con un byte que no es utf8 deja la historia ilegible: {:?}",
        held.read_all().err()
    );
}
