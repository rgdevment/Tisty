#[test]
fn the_segments_of_a_machine_always_come_back_in_the_order_they_were_written() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("dev_a");
    std::fs::create_dir_all(&at).unwrap();
    for leaf in [
        "active.tisty",
        "000002.tisty",
        "000010.tisty",
        "000001.tisty",
    ] {
        std::fs::write(at.join(leaf), b"x").unwrap();
    }

    let found: Vec<String> = segments_in(&at)
        .unwrap()
        .iter()
        .filter_map(|one| one.file_name()?.to_str().map(str::to_string))
        .collect();

    assert_eq!(
        found,
        vec![
            "000001.tisty",
            "000002.tisty",
            "000010.tisty",
            "active.tisty"
        ]
    );
}
use super::*;
use crate::event::TaskAdd;
use ulid::Ulid;

fn add(title: &str) -> Op {
    Op::TaskAdd {
        id: Ulid::generate(),
        d: TaskAdd::new(title, "a0"),
    }
}

fn seated(at: &Path, who: &str, ops: Vec<Op>) {
    let mut store = Store::open(at, DeviceId(who.into())).unwrap();
    for op in ops {
        store.append(op).unwrap();
    }
}

fn poured(from: &Path, into: &Path) {
    for one in std::fs::read_dir(from).unwrap().filter_map(|e| e.ok()) {
        if !one.file_type().unwrap().is_dir() {
            continue;
        }
        let there = into.join(one.file_name());
        std::fs::create_dir_all(&there).unwrap();
        for file in std::fs::read_dir(one.path())
            .unwrap()
            .filter_map(|e| e.ok())
        {
            std::fs::copy(file.path(), there.join(file.file_name())).unwrap();
        }
    }
}

#[test]
fn concatenating_two_histories_locks_nobody_who_was_writing_out() {
    let here = tempfile::tempdir().unwrap();
    let there = tempfile::tempdir().unwrap();
    seated(
        here.path(),
        "dev_here",
        vec![
            Op::DeviceJoin {
                d: DeviceId("dev_here".into()),
                k: Some(crate::event::DeviceKind::Machine),
            },
            add("lo de aqui"),
        ],
    );
    seated(
        there.path(),
        "dev_there",
        vec![
            Op::DeviceJoin {
                d: DeviceId("dev_there".into()),
                k: Some(crate::event::DeviceKind::Machine),
            },
            add("lo de alli"),
        ],
    );

    poured(there.path(), here.path());
    let said = ledger(here.path()).unwrap();

    assert!(said.may_write(&DeviceId("dev_here".into())));
    assert!(said.may_write(&DeviceId("dev_there".into())));
    assert!(!said.was_removed(&DeviceId("dev_here".into())));
    assert!(!said.was_removed(&DeviceId("dev_there".into())));
}

#[test]
fn a_history_that_never_named_anyone_is_not_shut_out_by_one_that_did() {
    let here = tempfile::tempdir().unwrap();
    let there = tempfile::tempdir().unwrap();
    seated(here.path(), "dev_here", vec![add("lo de aqui, sin alta")]);
    seated(
        there.path(),
        "dev_there",
        vec![
            Op::DeviceJoin {
                d: DeviceId("dev_there".into()),
                k: Some(crate::event::DeviceKind::Machine),
            },
            Op::DeviceRemove {
                d: DeviceId("dev_gone".into()),
            },
        ],
    );

    poured(there.path(), here.path());
    let said = ledger(here.path()).unwrap();

    assert!(
        said.may_write(&DeviceId("dev_here".into())),
        "la maquina que nunca se dio de alta quedo fuera al fusionar"
    );
    assert!(said.may_write(&DeviceId("dev_there".into())));
    assert!(said.was_removed(&DeviceId("dev_gone".into())));
}

#[test]
fn once_a_machine_is_removed_no_ordering_of_the_log_lets_it_back_in() {
    let when: jiff::Timestamp = "2026-08-15T00:00:00Z".parse().unwrap();
    let stamped = |who: &str, seq: u64, op: Op| Event {
        version: SCHEMA_VERSION,
        timestamp: when,
        device: DeviceId(who.into()),
        batch: None,
        undo: false,
        redo: false,
        seq,
        op,
        optional: false,
        zone: None,
        via: None,
    };

    let told = |remover: &str, joiner: &str| {
        let root = tempfile::tempdir().unwrap();
        for (who, seq, op) in [
            (
                "dev_m",
                1,
                Op::DeviceJoin {
                    d: DeviceId("dev_m".into()),
                    k: Some(crate::event::DeviceKind::Machine),
                },
            ),
            (
                remover,
                2,
                Op::DeviceRemove {
                    d: DeviceId("dev_both".into()),
                },
            ),
            (
                joiner,
                3,
                Op::DeviceJoin {
                    d: DeviceId("dev_both".into()),
                    k: Some(crate::event::DeviceKind::Machine),
                },
            ),
        ] {
            let mut store = Store::open(root.path(), DeviceId(who.into())).unwrap();
            store.append_event(&stamped(who, seq, op)).unwrap();
        }
        ledger(root.path())
            .unwrap()
            .may_write(&DeviceId("dev_both".into()))
    };

    assert!(!told("dev_a", "dev_z"), "el alta posterior la resucito");
    assert!(
        !told("dev_z", "dev_a"),
        "el desempate por nombre la resucito"
    );
}

#[test]
fn a_removal_survives_a_clock_that_runs_behind_the_machine_it_removes() {
    let root = tempfile::tempdir().unwrap();
    let earlier: jiff::Timestamp = "2026-08-15T00:00:00Z".parse().unwrap();
    let later: jiff::Timestamp = "2026-08-15T00:00:05Z".parse().unwrap();
    let stamped = |who: &str, when: jiff::Timestamp, seq: u64, op: Op| Event {
        version: SCHEMA_VERSION,
        timestamp: when,
        device: DeviceId(who.into()),
        batch: None,
        undo: false,
        redo: false,
        seq,
        op,
        optional: false,
        zone: None,
        via: None,
    };

    for (who, when, seq, op) in [
        (
            "dev_keeper",
            earlier,
            1,
            Op::DeviceJoin {
                d: DeviceId("dev_keeper".into()),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ),
        (
            "dev_gone",
            later,
            1,
            Op::DeviceJoin {
                d: DeviceId("dev_gone".into()),
                k: Some(crate::event::DeviceKind::Machine),
            },
        ),
        (
            "dev_keeper",
            earlier,
            2,
            Op::DeviceRemove {
                d: DeviceId("dev_gone".into()),
            },
        ),
    ] {
        let mut store = Store::open(root.path(), DeviceId(who.into())).unwrap();
        store.append_event(&stamped(who, when, seq, op)).unwrap();
    }

    assert!(
        ledger(root.path())
            .unwrap()
            .was_removed(&DeviceId("dev_gone".into())),
        "la baja se deshizo sola porque el reloj de quien la dio iba atrasado"
    );
}

#[test]
fn a_segment_that_arrived_half_written_is_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    let device = DeviceId("dev_a".into());
    let mut store = Store::open(tmp.path(), device.clone()).unwrap();
    for i in 0..4 {
        store.append(add(&format!("task {i}"))).unwrap();
    }
    store.active_events = SEGMENT_MAX_EVENTS;
    store.append(add("one more")).unwrap();

    let dir = tmp.path().join(&device.0);
    let sealed = dir.join("000001.tisty");
    assert_eq!(declared_count(&sealed), Some(4));

    let kept: String = std::fs::read_to_string(&sealed)
        .unwrap()
        .lines()
        .take(2)
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&sealed, kept + "\n").unwrap();

    assert!(matches!(
        read_all(tmp.path()),
        Err(Error::TruncatedSegment {
            found: 2,
            declared: Some(4),
            ..
        })
    ));
}

#[test]
fn an_emptied_segment_is_an_error_not_an_empty_history() {
    let tmp = tempfile::tempdir().unwrap();
    let device = DeviceId("dev_a".into());
    let mut store = Store::open(tmp.path(), device.clone()).unwrap();
    store.append(add("kept")).unwrap();

    let dir = tmp.path().join(&device.0);
    std::fs::rename(dir.join(ACTIVE), dir.join("000001.tisty")).unwrap();
    std::fs::write(dir.join("000001.tisty"), "").unwrap();

    assert!(matches!(
        read_all(tmp.path()),
        Err(Error::TruncatedSegment { .. })
    ));
}

#[test]
fn every_stamp_is_greater_than_the_one_before_it() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    for i in 0..50 {
        store.append(add(&format!("task {i}"))).unwrap();
    }

    let events = read_all(tmp.path()).unwrap();
    let keys: Vec<_> = events
        .iter()
        .map(|e| (e.timestamp, e.seq))
        .collect::<Vec<_>>();
    for pair in keys.windows(2) {
        assert!(
            pair[0] < pair[1],
            "{:?} does not precede {:?}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn a_batch_written_in_one_instant_still_orders() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    let written = store
        .append_batch(vec![add("first"), add("second"), add("third")])
        .unwrap();

    for pair in written.windows(2) {
        assert!(pair[0].sort_key() < pair[1].sort_key());
    }
}

#[test]
fn reopening_does_not_rewind_the_clock() {
    let tmp = tempfile::tempdir().unwrap();
    let device = DeviceId("dev_a".into());

    let mut first = Store::open(tmp.path(), device.clone()).unwrap();
    let before = first.append(add("before")).unwrap();
    drop(first);

    let mut second = Store::open(tmp.path(), device).unwrap();
    let after = second.append(add("after")).unwrap();

    assert!(after.sort_key() > before.sort_key());
}

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

#[test]
fn appends_and_reads_back() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    store.append(add("first")).unwrap();
    store.append(add("second")).unwrap();

    assert_eq!(store.read_all().unwrap().len(), 2);
}

#[test]
fn survives_reopening() {
    let tmp = tempfile::tempdir().unwrap();
    {
        let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
        store.append(add("persisted")).unwrap();
    }
    let store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    assert_eq!(store.read_all().unwrap().len(), 1);
}

#[test]
fn merges_devices_in_canonical_order() {
    let tmp = tempfile::tempdir().unwrap();
    let mut a = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    let mut b = Store::open(tmp.path(), DeviceId("dev_b".into())).unwrap();

    let same_instant = at(1_000);
    b.append_event(&Event::new(b.device().clone(), same_instant, add("from b")))
        .unwrap();
    a.append_event(&Event::new(a.device().clone(), same_instant, add("from a")))
        .unwrap();

    let from_a = a.read_all().unwrap();
    let from_b = b.read_all().unwrap();

    assert_eq!(from_a, from_b, "both devices see the same order");
    assert_eq!(from_a[0].device, DeviceId("dev_a".into()));
}

#[test]
fn a_writer_that_holds_the_lock_turns_the_other_away() {
    let tmp = tempfile::tempdir().unwrap();
    let mut holding = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    holding.acquire().unwrap();

    let mut other = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    assert!(matches!(
        other.append(add("second")),
        Err(Error::AlreadyRunning)
    ));
}

#[test]
fn reading_stays_possible_while_another_process_writes() {
    let tmp = tempfile::tempdir().unwrap();
    let mut writer = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    writer.append(add("written")).unwrap();

    let reader = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    assert_eq!(reader.read_all().unwrap().len(), 1);
}

#[test]
fn a_different_device_can_write_at_the_same_time() {
    let tmp = tempfile::tempdir().unwrap();
    let mut a = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    let mut b = Store::open(tmp.path(), DeviceId("dev_b".into())).unwrap();

    a.append(add("from a")).unwrap();
    assert!(b.append(add("from b")).is_ok());
}

#[test]
fn rotation_closes_segments_and_keeps_every_event() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();

    store.append(add("before rotation")).unwrap();
    store.active_events = SEGMENT_MAX_EVENTS;
    store.append(add("after rotation")).unwrap();

    let dir = tmp.path().join("dev_a");
    assert!(dir.join("000001.tisty").exists(), "segment was closed");
    assert!(dir.join(ACTIVE).exists(), "a fresh active file took over");
    assert_eq!(store.read_all().unwrap().len(), 2);
}

#[test]
fn rotation_tolerates_a_missing_active_file() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();

    store.active_events = SEGMENT_MAX_EVENTS;
    store.append(add("after a lost file")).unwrap();

    assert_eq!(store.read_all().unwrap().len(), 1);
}

#[test]
fn segments_are_read_in_order() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();

    let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    for i in 0..3 {
        store
            .append_event(&Event::new(
                store.device().clone(),
                at(i + 1),
                add(&format!("event {i}")),
            ))
            .unwrap();
        store.active_events = SEGMENT_MAX_EVENTS;
    }

    let events = store.read_all().unwrap();
    assert_eq!(events.len(), 3);
    assert!(events.windows(2).all(|w| w[0].sort_key() < w[1].sort_key()));
}

#[test]
fn a_malformed_line_reports_where_it_is() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(ACTIVE), "{\"v\":1}\n{\"not\":\"an event\"}\n").unwrap();

    match read_all(tmp.path()) {
        Err(Error::MalformedEvent { line, .. }) => assert_eq!(line, 1),
        other => panic!("expected MalformedEvent, got {other:?}"),
    }
}

#[test]
fn a_future_schema_version_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();

    let mut future = serde_json::to_value(Event::new(
        DeviceId("dev_a".into()),
        at(1),
        add("from the future"),
    ))
    .unwrap();
    future["v"] = serde_json::json!(SCHEMA_VERSION + 1);
    std::fs::write(dir.join(ACTIVE), format!("{future}\n")).unwrap();

    assert!(matches!(
        read_all(tmp.path()),
        Err(Error::UnsupportedVersion(_))
    ));
}

#[test]
fn an_operation_this_build_does_not_know_is_skipped_when_it_says_it_may_be() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    let known = format!(
        r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T10:00:00Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"title":"still here","order":"V"}}}}"#
    );
    let stranger = format!(
        r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.bless","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"halo":true}}}}"#
    );
    std::fs::write(
        dir.join(ACTIVE),
        format!(
            "{known}
{stranger}
"
        ),
    )
    .unwrap();

    let read = read_all(tmp.path()).unwrap();
    assert_eq!(
        read.len(),
        1,
        "the known event has to survive its unknown neighbour"
    );
}

#[test]
fn an_operation_this_build_does_not_know_stops_the_read_unless_it_says_otherwise() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    let stranger = format!(
        r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","op":"task.erase.everything","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{}}}}"#
    );
    std::fs::write(
        dir.join(ACTIVE),
        format!(
            "{stranger}
"
        ),
    )
    .unwrap();

    assert!(
        matches!(read_all(tmp.path()), Err(Error::MalformedEvent { .. })),
        "an operation that may change what exists cannot be waved through"
    );
}

#[test]
fn a_version_number_below_this_one_never_blocks_a_read() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();

    let mut lines = String::new();
    for v in 1..=SCHEMA_VERSION {
        lines.push_str(&format!(
            r#"{{"v":{v},"ts":"2026-08-28T10:00:{v:02}Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59X{v:02}","d":{{"title":"written by v{v}","order":"V"}}}}"#
        ));
        lines.push('\n');
    }
    std::fs::write(dir.join(ACTIVE), lines).unwrap();

    let read = read_all(tmp.path()).unwrap();
    assert_eq!(read.len(), SCHEMA_VERSION as usize);
    assert!(
        read.iter().all(|e| matches!(&e.op, Op::TaskAdd { .. })),
        "the guard only refuses upwards; what the older shapes look like is another test"
    );
}

#[test]
fn an_event_carries_the_zone_of_whoever_wrote_it() {
    let tmp = tempfile::tempdir().unwrap();
    let mut store = Store::open(tmp.path(), DeviceId("dev_a".into())).unwrap();
    let event = store
        .append(Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: crate::event::TaskAdd::new("where was I", "a0"),
        })
        .unwrap();

    assert!(
        event.zone.is_some(),
        "a written event knows where it was written"
    );
    assert_eq!(
        event.zoned().map(|z| z.timestamp()),
        Some(event.timestamp),
        "reading it back in its own zone is the same instant"
    );
}

#[test]
fn a_skipped_event_still_counts_towards_a_sealed_segment() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    let known = format!(
        r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T10:00:00Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"title":"sealed away","order":"V"}}}}"#
    );
    let stranger = format!(
        r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.bless","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{}}}}"#
    );
    std::fs::write(
        dir.join("000001.tisty"),
        format!(
            "{known}
{stranger}
"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("000001.count"), "2").unwrap();

    let read = read_all(tmp.path()).unwrap();
    assert_eq!(
        read.len(),
        1,
        "a sealed segment declares lines, so skipping one cannot read as a truncated download"
    );
}

#[test]
fn an_event_from_a_newer_schema_is_skipped_when_it_says_it_may_be() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    let ahead = format!(
        r#"{{"v":{},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.bless","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{}}}}"#,
        SCHEMA_VERSION + 1
    );
    std::fs::write(
        dir.join(ACTIVE),
        format!(
            "{ahead}
"
        ),
    )
    .unwrap();
    assert!(
        read_all(tmp.path()).unwrap().is_empty(),
        "the operations this guard exists for will arrive under a newer schema, not this one"
    );

    let sealed = ahead.replace(r#""opt":true,"#, "");
    std::fs::write(
        dir.join(ACTIVE),
        format!(
            "{sealed}
"
        ),
    )
    .unwrap();
    assert!(matches!(
        read_all(tmp.path()),
        Err(Error::UnsupportedVersion(_))
    ));
}

#[test]
fn a_known_operation_that_arrived_corrupt_is_never_waved_through() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    let rotten = format!(
        r#"{{"v":{SCHEMA_VERSION},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{{"title":12345,"order":"V"}}}}"#
    );
    std::fs::write(
        dir.join(ACTIVE),
        format!(
            "{rotten}
"
        ),
    )
    .unwrap();

    assert!(
        matches!(read_all(tmp.path()), Err(Error::MalformedEvent { .. })),
        "the mark forgives an operation nobody knows, never one that arrived broken"
    );
}

#[test]
fn the_shape_a_previous_build_wrote_still_projects() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    // Written by hand as a build before v7 would have: no tz, no opt, no k, no filled,
    // no source. A `default` dropped by accident has to fail here, not in someone's store.
    let before = concat!(
        r#"{"v":6,"ts":"2026-08-01T10:00:00Z","by":"dev_a","op":"device.join","d":"dev_a"}"#,
        "
",
        r#"{"v":6,"ts":"2026-08-01T10:00:01Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{"title":"written before v7","order":"V"}}"#,
        "
",
        r#"{"v":6,"ts":"2026-08-01T10:00:02Z","by":"dev_a","op":"task.done","id":"01M14RFT9ECC2B6E4CX4P59XPH"}"#,
        "
",
    );
    std::fs::write(dir.join(ACTIVE), before).unwrap();

    let read = read_all(tmp.path()).unwrap();
    assert_eq!(read.len(), 3);
    assert!(read.iter().all(|e| e.zone.is_none() && !e.optional));

    let state = crate::State::replay(&read);
    let task = &state.tasks[&"01M14RFT9ECC2B6E4CX4P59XPH".parse().unwrap()];
    assert_eq!(task.status, crate::model::Status::Done);
    assert!(
        !task.filled,
        "a closure from before the field is not a backfill"
    );
    assert_eq!(task.source, None);
    assert!(
        state.devices.contains(&DeviceId("dev_a".into())) && state.agents.is_empty(),
        "a join with nothing to say about its kind is not a claim that it is a machine"
    );
}

#[test]
fn a_known_operation_from_a_newer_schema_is_never_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("dev_a");
    std::fs::create_dir_all(&dir).unwrap();
    let ahead = format!(
        r#"{{"v":{},"ts":"2026-08-28T11:00:00Z","by":"dev_a","opt":true,"op":"task.delete","id":"01M14RFT9ECC2B6E4CX4P59XPH"}}"#,
        SCHEMA_VERSION + 1
    );
    std::fs::write(
        dir.join(ACTIVE),
        format!(
            "{ahead}
"
        ),
    )
    .unwrap();

    assert!(
        matches!(read_all(tmp.path()), Err(Error::UnsupportedVersion(_))),
        "a deletion this build understands cannot be dropped for wearing a newer number:              the task would stay alive here and be gone everywhere else"
    );
}

#[test]
fn an_empty_store_is_not_an_error() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(read_all(tmp.path().join("missing")).unwrap().is_empty());
}

#[test]
fn a_key_kept_inside_the_store_moves_out_and_leaves_nothing_behind() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let store = paths.store();
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(store.join(KEEP), [7u8; 32]).unwrap();

    brought_home(&paths);

    assert!(
        !store.join(KEEP).exists(),
        "the key stayed where a backup reaches it"
    );
    let named = peek_identity(&store).unwrap();
    assert_eq!(
        std::fs::read(kept_at(&paths, &named).unwrap()).unwrap(),
        [7u8; 32]
    );
    assert_eq!(secret(&paths).unwrap(), [7u8; 32]);
}

#[test]
fn bringing_the_key_home_when_there_never_was_one_inside_makes_none() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let store = paths.store();
    let private = paths.private();
    std::fs::create_dir_all(&store).unwrap();

    brought_home(&paths);

    assert!(
        !private.join(KEEP).exists(),
        "a key was made out of nothing"
    );
}

#[test]
fn bringing_the_key_home_twice_changes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let store = paths.store();
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(store.join(KEEP), [3u8; 32]).unwrap();

    brought_home(&paths);
    brought_home(&paths);

    let named = peek_identity(&store).unwrap();
    assert_eq!(
        std::fs::read(kept_at(&paths, &named).unwrap()).unwrap(),
        [3u8; 32]
    );
    assert_eq!(displaced(&paths).len(), 0, "the second pass set one aside");
}

#[test]
fn a_key_that_turns_up_inside_an_already_moved_store_does_not_win() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let store = paths.store();
    std::fs::create_dir_all(paths.private()).unwrap();
    let named = identity(&store).unwrap();
    std::fs::write(kept_at(&paths, &named).unwrap(), [2u8; 32]).unwrap();
    std::fs::write(store.join(KEEP), [1u8; 32]).unwrap();

    brought_home(&paths);

    assert_eq!(
        std::fs::read(kept_at(&paths, &named).unwrap()).unwrap(),
        [2u8; 32],
        "the newcomer took over from what this store seals with"
    );
    let kept: Vec<Vec<u8>> = displaced(&paths)
        .iter()
        .map(|one| std::fs::read(one).unwrap())
        .collect();
    assert_eq!(
        kept,
        vec![vec![1u8; 32]],
        "the one it displaced was destroyed instead of set aside"
    );
    assert!(!store.join(KEEP).exists());
}

#[test]
fn something_that_is_not_a_key_is_left_where_it_is() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let store = paths.store();
    let private = paths.private();
    std::fs::create_dir_all(&store).unwrap();
    std::fs::write(store.join(KEEP), b"not a key").unwrap();

    brought_home(&paths);

    assert!(
        store.join(KEEP).exists(),
        "it was taken for a key and moved"
    );
    assert!(!private.join(KEEP).exists());
}

#[test]
fn write_atomic_leaves_no_temporary_behind() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("config.toml");

    write_atomic(&target, b"first").unwrap();
    write_atomic(&target, b"second").unwrap();

    assert_eq!(std::fs::read_to_string(&target).unwrap(), "second");
    assert!(!target.with_extension("tmp").exists());
}

#[test]
fn a_write_does_not_keep_the_lock() {
    let tmp = tempfile::tempdir().unwrap();
    let device = DeviceId("dev_a".into());

    let mut gui = Store::open(tmp.path(), device.clone()).unwrap();
    gui.append(add("from the window")).unwrap();

    let mut cli = Store::open(tmp.path(), device).unwrap();
    assert!(cli.append(add("from the terminal")).is_ok());
}

#[test]
fn two_processes_on_one_device_never_stamp_the_same_event() {
    let tmp = tempfile::tempdir().unwrap();
    let device = DeviceId("dev_a".into());

    let mut gui = Store::open(tmp.path(), device.clone()).unwrap();
    let mut cli = Store::open(tmp.path(), device).unwrap();

    let mut stamps = Vec::new();
    for i in 0..8 {
        let who = if i % 2 == 0 { &mut gui } else { &mut cli };
        let event = who.append(add(&format!("task {i}"))).unwrap();
        stamps.push((event.timestamp, event.seq));
    }

    let mut unique = stamps.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(unique.len(), stamps.len(), "collided: {stamps:?}");
}

#[test]
fn catching_up_never_rewinds_the_clock() {
    let tmp = tempfile::tempdir().unwrap();
    let device = DeviceId("dev_a".into());

    let mut first = Store::open(tmp.path(), device.clone()).unwrap();
    first.append(add("one")).unwrap();

    let mut second = Store::open(tmp.path(), device).unwrap();
    second.append(add("two")).unwrap();

    let ahead = second.head;
    first.append(add("three")).unwrap();
    assert!(first.head >= ahead);
}

#[test]
fn rotation_resets_what_the_store_believes_it_has_seen() {
    let tmp = tempfile::tempdir().unwrap();
    let device = DeviceId("dev_a".into());
    let mut store = Store::open(tmp.path(), device).unwrap();

    store.append(add("before")).unwrap();
    store.active_events = SEGMENT_MAX_EVENTS;
    store.append(add("after")).unwrap();

    assert_eq!(store.active_events, 1);
    assert_eq!(store.seen, active_size(&store.dir.join(ACTIVE)));
}
#[test]
fn a_store_keeps_the_same_name_however_often_it_is_asked() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");

    let first = identity(&root).unwrap();
    assert!(!first.is_empty());
    assert_eq!(identity(&root).unwrap(), first);

    let other = tempfile::tempdir().unwrap();
    assert_ne!(identity(other.path()).unwrap(), first);
}

#[test]
fn an_empty_marker_is_replaced_rather_than_trusted() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path()).unwrap();
    std::fs::write(
        dir.path().join(MARKER),
        "   
",
    )
    .unwrap();

    let named = identity(dir.path()).unwrap();
    assert!(!named.trim().is_empty());
}

#[test]
fn the_marker_does_not_disturb_reading_the_log() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("store");
    let mut store = Store::open(&root, DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("comprar pan", "a0"),
        })
        .unwrap();

    identity(&root).unwrap();
    assert_eq!(read_all(&root).unwrap().len(), 1);
}

#[test]
fn what_is_set_aside_is_named_after_the_key_it_replaces() {
    assert!(
        DISPLACED.starts_with(KEEP),
        "{DISPLACED} would not be found beside {KEEP}"
    );
}

#[test]
fn the_key_is_named_after_the_store_it_proves() {
    let dir = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    let named = identity(paths.store()).unwrap();

    let kept = secret(&paths).unwrap();

    assert_eq!(
        std::fs::read(kept_at(&paths, &named).unwrap()).unwrap(),
        kept,
        "a second store on this machine would write over the first one's key"
    );
}

#[test]
fn two_stores_on_one_machine_do_not_share_a_key() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config");
    let one = crate::Paths::new(dir.path().join("one"), &config);
    let two = crate::Paths::new(dir.path().join("two"), &config);
    identity(one.store()).unwrap();
    identity(two.store()).unwrap();

    let first = secret(&one).unwrap();
    let second = secret(&two).unwrap();

    assert_ne!(first, second, "the second store took over the first's seal");
    assert_eq!(secret(&one).unwrap(), first, "the first store lost its key");
}

#[test]
fn a_key_set_aside_twice_in_the_same_second_keeps_both() {
    let dir = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    std::fs::create_dir_all(paths.private()).unwrap();
    let at = kept_at(&paths, "01ABCDEFGHJKMNPQRSTVWXYZ00").unwrap();

    assert!(set_aside(&paths, &at, &[1u8; 32], "the first"));
    assert!(set_aside(&paths, &at, &[2u8; 32], "the second"));

    let aside = displaced(&paths);
    assert_eq!(
        aside.len(),
        2,
        "the second write destroyed the first: {aside:?}"
    );
    let held: Vec<Vec<u8>> = aside
        .iter()
        .map(|one| std::fs::read(one).unwrap())
        .collect();
    assert!(held.contains(&vec![1u8; 32]), "the first key is gone");
    assert!(held.contains(&vec![2u8; 32]), "the second key is gone");
}

#[test]
fn a_key_that_is_not_one_is_set_aside_and_a_fresh_one_minted() {
    let dir = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    let named = identity(paths.store()).unwrap();
    std::fs::create_dir_all(paths.private()).unwrap();
    std::fs::write(kept_at(&paths, &named).unwrap(), b"").unwrap();

    let kept = secret(&paths).expect("a store with a broken key can never seal anything again");

    assert_eq!(
        std::fs::read(kept_at(&paths, &named).unwrap()).unwrap(),
        kept
    );
    assert_eq!(
        displaced(&paths).len(),
        1,
        "what could not be read was destroyed"
    );
}

#[test]
fn a_store_cannot_name_itself_out_of_the_private_folder() {
    for climbing in [
        "../../evil",
        "..\\..\\evil",
        "a/b",
        "",
        "   ",
        "01ABCDEFGHJKMNPQRSTVWXYZ0",
        "01abcdefghjkmnpqrstvwxyz00",
        "01ABCDEFGHJKMNPQRSTVWXYZ000",
        "01ABCDEFGHJKMNPQRSTVWXY:00",
    ] {
        assert!(!is_store_name(climbing), "«{climbing}» would become a path");
    }
    assert!(is_store_name("01ABCDEFGHJKMNPQRSTVWXYZ00"));
    assert!(is_store_name(&ulid::Ulid::generate().to_string()));
}

#[test]
fn a_name_a_shared_folder_made_up_never_reaches_the_private_folder() {
    let dir = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    std::fs::create_dir_all(paths.store()).unwrap();
    std::fs::write(paths.store().join(MARKER), "../../../taken").unwrap();
    std::fs::write(paths.store().join(KEEP), [1u8; 32]).unwrap();

    assert_eq!(
        peek_identity(paths.store()),
        None,
        "a name another machine wrote is trusted as a file name"
    );
    assert_eq!(kept_at(&paths, "../../../taken"), None);

    brought_home(&paths);
    secret(&paths);

    assert!(
        !dir.path().join("taken.store-key").exists()
            && !dir
                .path()
                .parent()
                .unwrap()
                .join("taken.store-key")
                .exists(),
        "a key was written outside the private folder"
    );
}

#[test]
fn a_key_the_migration_left_alone_is_still_what_the_store_seals_with() {
    let dir = tempfile::tempdir().unwrap();
    let mut paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    paths.unpaired_for_test();
    std::fs::create_dir_all(paths.store()).unwrap();
    std::fs::write(paths.store().join(KEEP), [5u8; 32]).unwrap();

    brought_home(&paths);

    assert_eq!(
        secret(&paths),
        Some([5u8; 32]),
        "the key was left in the store and a brand new one was minted beside it, so every parcel this store handed out is now a stranger's"
    );
}

#[test]
fn settings_kept_somewhere_else_are_still_this_installs_own() {
    let dir = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("elsewhere"));
    std::fs::create_dir_all(paths.store()).unwrap();
    std::fs::write(paths.store().join(KEEP), [8u8; 32]).unwrap();

    brought_home(&paths);

    let named = peek_identity(paths.store()).unwrap();
    assert_eq!(
        std::fs::read(kept_at(&paths, &named).unwrap()).unwrap(),
        [8u8; 32],
        "naming the settings directory made this install look like somebody else's"
    );
}

#[test]
fn a_key_that_cannot_be_removed_is_not_set_aside_again_and_again() {
    let dir = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    std::fs::create_dir_all(paths.private()).unwrap();
    let at = kept_at(&paths, "01ABCDEFGHJKMNPQRSTVWXYZ00").unwrap();

    for _ in 0..5 {
        assert!(set_aside(&paths, &at, &[4u8; 32], "the same one again"));
    }

    assert_eq!(
        displaced(&paths).len(),
        1,
        "one stuck key grew the private folder on every command"
    );
}

#[test]
fn a_store_opened_with_another_installs_settings_keeps_its_key_where_it_is() {
    let dir = tempfile::tempdir().unwrap();
    let mut paths = crate::Paths::new(dir.path().join("data"), dir.path().join("config"));
    paths.unpaired_for_test();
    let named = identity(paths.store()).unwrap();
    std::fs::write(paths.store().join(KEEP), [3u8; 32]).unwrap();

    brought_home(&paths);

    assert!(
        paths.store().join(KEEP).exists(),
        "somebody else's key was taken off their store"
    );
    assert!(
        !kept_at(&paths, &named).unwrap().exists(),
        "somebody else's key was installed on this machine"
    );
}
