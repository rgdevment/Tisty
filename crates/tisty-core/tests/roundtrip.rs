use tisty_core::{
    Config, DeviceId, Event, Op, Paths, State, Status, Store, Tag,
    event::{LogAdd, StepAdd, StepRef, TaskAdd},
    model::Priority,
};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

/// The full lifecycle of the task from the design examples: created, described,
/// worked on across two machines, and archived. What ends up on disk must
/// project back into exactly what was written.
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
                    name: "unificación login".into(),
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
                    tags: vec![Tag::new("istio").unwrap(), Tag::new("brasil").unwrap()],
                    ..TaskAdd::new("issue en redirecciones istio para registration en BR", "a0")
                },
            },
        ),
        (
            3,
            &laptop,
            Op::TaskDescribe {
                id: task,
                d: tisty_core::event::Body {
                    body: Some("El query string se pierde. Ticket [[CUSLEG-3465]].".into()),
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
                    text: "desplegar charts".into(),
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
                    text: "validar en producción BR".into(),
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
                d: LogAdd {
                    entry: first_note,
                    body: "los charts fallaron por el sidecar".into(),
                },
            },
        ),
        (
            8,
            &desktop,
            Op::TaskLog {
                id: task,
                d: LogAdd {
                    entry: second_note,
                    body: "era el header X-Forwarded-Proto".into(),
                },
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

    assert_eq!(
        t.title,
        "issue en redirecciones istio para registration en BR"
    );
    assert_eq!(t.status, Status::Done);
    assert_eq!(t.completed_at, Some(at(10)));
    assert_eq!(t.priority, Priority::P1);
    assert_eq!(t.list, Some(list));
    assert_eq!(t.tags.len(), 2);
    assert_eq!(t.steps_done(), (2, 2));
    assert_eq!(t.log.len(), 2);
    assert!(t.description.as_ref().unwrap().contains("CUSLEG-3465"));

    assert!(t.log[0].at < t.log[1].at, "the log reads chronologically");
    assert_eq!(t.steps[0].text, "desplegar charts");

    assert!(t.is_archived());
    assert_eq!(state.open_tasks().count(), 0);
    assert_eq!(state.archived_tasks().count(), 1, "archived, not deleted");
    assert!(
        state.is_settled(list),
        "the list sinks once nothing is open"
    );
}

/// A trivial task and a documented one must both be first-class, and the weight
/// has to tell them apart so the interface can skip what was never used.
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
            d: TaskAdd::new("agendar reunión con Pepe", "a0"),
        })
        .unwrap();
    store
        .append(Op::TaskAdd {
            id: documented,
            d: TaskAdd {
                tags: vec![Tag::new("istio").unwrap()],
                ..TaskAdd::new("issue en redirecciones", "a1")
            },
        })
        .unwrap();
    store
        .append(Op::TaskLog {
            id: documented,
            d: LogAdd {
                entry: Ulid::generate(),
                body: "el sidecar no arrancaba".into(),
            },
        })
        .unwrap();

    let state = State::replay(&store.read_all().unwrap());

    assert_eq!(state.tasks[&trivial].weight(), 0);
    assert!(state.tasks[&documented].weight() > state.tasks[&trivial].weight());
}

/// Reading the store from a second device must produce the same state, or two
/// machines would disagree about what happened.
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
