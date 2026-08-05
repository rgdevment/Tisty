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
                    priority: Some(Priority::P1),
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
    assert_eq!(t.priority, Priority::P1);
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
