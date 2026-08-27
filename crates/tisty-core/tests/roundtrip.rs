use tisty_core::{
    Config, DeviceId, Event, Op, Paths, State, Status, Store, Tag,
    event::{LogAdd, StepAdd, StepRef, TaskAdd},
    model::Priority,
};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

#[test]
fn a_task_survives_the_round_trip_through_disk() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let config = Config::load_or_init(&paths).unwrap();

    let task = Ulid::generate();
    let list = Ulid::generate();
    let (charts, validate) = (Ulid::generate(), Ulid::generate());
    let (first_note, second_note) = (Ulid::generate(), Ulid::generate());

    let mut store = Store::open(paths.store(), config.device_id.clone()).unwrap();
    let laptop = config.device_id.clone();
    let desktop = DeviceId("dev_9c02".into());

    let script = [
        (
            1,
            &laptop,
            Op::ListAdd {
                id: list,
                d: tisty_core::event::ListAdd {
                    name: "checkout rewrite".into(),
                    order: "a0".into(),
                    color: None,
                },
            },
        ),
        (
            2,
            &laptop,
            Op::TaskAdd {
                id: task,
                d: TaskAdd {
                    list: Some(list),
                    priority: Some(Priority::Do),
                    tags: vec![Tag::new("work").unwrap(), Tag::new("urgent").unwrap()],
                    ..TaskAdd::new("fix the failing checkout", "a0")
                },
            },
        ),
        (
            3,
            &laptop,
            Op::TaskDescribe {
                id: task,
                d: tisty_core::event::Body {
                    body: Some("Reproduces only with an empty cart. Ticket [[ABC-123]].".into()),
                },
            },
        ),
        (
            4,
            &laptop,
            Op::StepAdd {
                id: task,
                d: StepAdd {
                    step: charts,
                    text: "reproduce it locally".into(),
                    order: "a1".into(),
                },
            },
        ),
        (
            5,
            &laptop,
            Op::StepAdd {
                id: task,
                d: StepAdd {
                    step: validate,
                    text: "verify in production".into(),
                    order: "a2".into(),
                },
            },
        ),
        (
            6,
            &laptop,
            Op::StepDone {
                id: task,
                d: StepRef { step: charts },
            },
        ),
        (
            7,
            &laptop,
            Op::TaskLog {
                id: task,
                d: LogAdd::new(first_note, "first attempt failed"),
            },
        ),
        (
            8,
            &desktop,
            Op::TaskLog {
                id: task,
                d: LogAdd::new(second_note, "an index was missing"),
            },
        ),
        (
            9,
            &desktop,
            Op::StepDone {
                id: task,
                d: StepRef { step: validate },
            },
        ),
        (10, &desktop, Op::TaskDone { id: task }),
    ];

    let mut desktop_store = Store::open(paths.store(), desktop.clone()).unwrap();
    for (ms, device, op) in script {
        let event = Event::new(device.clone(), at(ms), op);
        if *device == laptop {
            store.append_event(&event).unwrap();
        } else {
            desktop_store.append_event(&event).unwrap();
        }
    }

    let state = State::replay(&store.read_all().unwrap());
    let t = &state.tasks[&task];

    assert_eq!(t.title, "fix the failing checkout");
    assert_eq!(t.status, Status::Done);
    assert_eq!(t.completed_at, Some(at(10)));
    assert_eq!(t.priority, Priority::Do);
    assert_eq!(t.list, Some(list));
    assert_eq!(t.tags.len(), 2);
    assert_eq!(t.steps_done(), (2, 2));
    assert_eq!(t.log.len(), 2);
    assert!(t.description.as_ref().unwrap().contains("ABC-123"));

    assert!(t.log[0].at < t.log[1].at, "the log reads chronologically");
    assert_eq!(t.steps[0].text, "reproduce it locally");

    assert!(t.is_archived());
    assert_eq!(state.open_tasks().count(), 0);
    assert_eq!(state.archived_tasks().count(), 1, "archived, not deleted");
    assert!(
        state.is_settled(list),
        "the list sinks once nothing is open"
    );
}

#[test]
fn trivial_and_documented_tasks_live_side_by_side() {
    let tmp = tempfile::tempdir().unwrap();
    let paths = Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let config = Config::load_or_init(&paths).unwrap();
    let mut store = Store::open(paths.store(), config.device_id.clone()).unwrap();

    let trivial = Ulid::generate();
    let documented = Ulid::generate();

    store
        .append(Op::TaskAdd {
            id: trivial,
            d: TaskAdd::new("book a haircut", "a0"),
        })
        .unwrap();
    store
        .append(Op::TaskAdd {
            id: documented,
            d: TaskAdd {
                tags: vec![Tag::new("work").unwrap()],
                ..TaskAdd::new("fix the failing checkout", "a1")
            },
        })
        .unwrap();
    store
        .append(Op::TaskLog {
            id: documented,
            d: LogAdd::new(Ulid::generate(), "an index was missing"),
        })
        .unwrap();

    let state = State::replay(&store.read_all().unwrap());

    assert_eq!(state.tasks[&trivial].weight(), 0);
    assert!(state.tasks[&documented].weight() > state.tasks[&trivial].weight());
}

#[test]
fn two_devices_project_the_same_state() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");

    let mut a = Store::open(&root, DeviceId("dev_a".into())).unwrap();
    let mut b = Store::open(&root, DeviceId("dev_b".into())).unwrap();

    let task = Ulid::generate();
    a.append_event(&Event::new(
        DeviceId("dev_a".into()),
        at(1),
        Op::TaskAdd {
            id: task,
            d: TaskAdd::new("ship it", "a0"),
        },
    ))
    .unwrap();
    b.append_event(&Event::new(
        DeviceId("dev_b".into()),
        at(2),
        Op::TaskDone { id: task },
    ))
    .unwrap();

    assert_eq!(
        State::replay(&a.read_all().unwrap()),
        State::replay(&b.read_all().unwrap())
    );
}

fn a_chain(events: &mut Vec<Event>, turns: usize, from: &str) -> Vec<Ulid> {
    let mut ids = Vec::new();
    let mut before: Option<Ulid> = None;
    let mut day: jiff::civil::Date = from.parse().unwrap();
    for n in 0..turns {
        let id = Ulid::generate();
        let mut add = TaskAdd::new("take the pill", "a0");
        add.date = Some(tisty_core::DateSpec::all_day(day, "UTC"));
        add.repeat = Some(tisty_core::model::Repeat::due(tisty_core::model::Cadence {
            every: 1,
            unit: tisty_core::model::Unit::Day,
        }));
        add.after = before;
        events.push(Event::new(
            DeviceId("dev_a".into()),
            at(1_770_000_000_000 + n as i64 * 1000),
            Op::TaskAdd { id, d: add },
        ));
        events.push(Event::new(
            DeviceId("dev_a".into()),
            at(1_770_000_000_500 + n as i64 * 1000),
            Op::TaskDone { id },
        ));
        ids.push(id);
        before = Some(id);
        day = day.tomorrow().unwrap();
    }
    ids
}

#[test]
fn a_story_survives_the_round_trip_through_json() {
    let id = Ulid::generate();
    let step = Ulid::generate();
    let events = vec![
        Event::new(
            DeviceId("dev_a".into()),
            at(1_770_000_000_000),
            Op::TaskAdd {
                id,
                d: TaskAdd::new("ship the release", "a0"),
            },
        ),
        Event::new(
            DeviceId("dev_a".into()),
            at(1_770_000_001_000),
            Op::StepAdd {
                id,
                d: StepAdd {
                    step,
                    text: "sign the installer".into(),
                    order: "a0".into(),
                },
            },
        ),
        Event::new(
            DeviceId("dev_a".into()),
            at(1_770_000_002_000),
            Op::StepDone {
                id,
                d: StepRef { step },
            },
        ),
        Event::new(
            DeviceId("dev_a".into()),
            at(1_770_000_003_000),
            Op::TaskLog {
                id,
                d: LogAdd::new(Ulid::generate(), "the authority took nine days"),
            },
        ),
        Event::new(
            DeviceId("dev_a".into()),
            at(1_770_000_004_000),
            Op::TaskDone { id },
        ),
    ];

    let told = tisty_core::story::story(&events, id);
    assert!(told.pages.len() >= 5);

    let wire = serde_json::to_string(&told).unwrap();
    let back: tisty_core::story::Story = serde_json::from_str(&wire).unwrap();

    assert_eq!(
        told, back,
        "a flattened, tagged enum is the easiest shape to break"
    );
    assert!(
        wire.contains("\"chapter\":\"ticked\""),
        "the tag every reader keys on has to be in the wire: {wire}"
    );
}

#[test]
fn a_series_survives_the_round_trip_through_json() {
    let mut events = Vec::new();
    let ids = a_chain(&mut events, 3, "2026-08-01");
    let state = State::replay(&events);

    let told = tisty_core::series::series(&state, ids[0]).unwrap();
    let wire = serde_json::to_string(&told).unwrap();
    let back: tisty_core::series::Series = serde_json::from_str(&wire).unwrap();

    assert_eq!(told, back);
    assert_eq!(back.turns.len(), 3);
    assert_eq!(back.kept, 3);
}

#[test]
fn the_shape_of_the_archive_survives_the_round_trip_through_json() {
    let mut events = Vec::new();
    a_chain(&mut events, 2, "2026-08-01");
    let state = State::replay(&events);

    let told = tisty_core::shape::shape(
        &state,
        6,
        &jiff::tz::TimeZone::UTC,
        "2026-08-26".parse().unwrap(),
    );
    let wire = serde_json::to_string(&told).unwrap();
    let back: tisty_core::shape::Shape = serde_json::from_str(&wire).unwrap();

    assert_eq!(told, back);
    assert_eq!(back.months.len(), 6, "the strip keeps its quiet months");
}
