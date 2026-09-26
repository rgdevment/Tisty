use super::*;
use crate::event::{DeviceId, LogAdd, StepAdd, TaskAdd};
use crate::{Op, Store};
use ulid::Ulid;

struct Fixture {
    _tmp: tempfile::TempDir,
    store_root: std::path::PathBuf,
    cache_dir: std::path::PathBuf,
    task: Ulid,
}

fn loaded() -> Fixture {
    let tmp = tempfile::tempdir().unwrap();
    let store_root = tmp.path().join("store");
    let cache_dir = tmp.path().join("cache");

    let mut store = Store::open(&store_root, DeviceId("dev_a".into())).unwrap();
    let task = Ulid::generate();
    store
        .append(Op::TaskAdd {
            id: task,
            d: TaskAdd::new("write the report", "a0"),
        })
        .unwrap();
    for body in ["spoke to accounting", "still waiting"] {
        store
            .append(Op::TaskLog {
                id: task,
                d: LogAdd::new(Ulid::generate(), body),
            })
            .unwrap();
    }
    store
        .append(Op::StepAdd {
            id: task,
            d: StepAdd {
                step: Ulid::generate(),
                text: "collect the figures".into(),
                order: "a0".into(),
            },
        })
        .unwrap();

    Fixture {
        _tmp: tmp,
        store_root,
        cache_dir,
        task,
    }
}

#[test]
fn a_word_about_a_machine_never_leaves_the_cache_claiming_to_be_current() {
    let dir = tempfile::tempdir().unwrap();
    let mut cache = Cache::open(dir.path()).unwrap().expect("a cache opens");
    let state = State::default();
    cache.store(&state, "before").unwrap();

    let said = crate::Event::new(
        crate::DeviceId("mac0".into()),
        jiff::Timestamp::from_second(1).unwrap(),
        crate::Op::DeviceRemove {
            d: crate::DeviceId("win1".into()),
        },
    );
    advance(Some(&mut cache), &state, &[said], dir.path(), false);

    assert!(
        cache.load("before", true).is_none(),
        "the cache still says it holds what it never saw"
    );
}

#[test]
fn what_the_cache_gives_back_still_knows_which_machines_write() {
    let dir = tempfile::tempdir().unwrap();
    let mut cache = Cache::open(dir.path()).unwrap().expect("a cache opens");
    let mut state = State::default();
    state.devices.insert(crate::DeviceId("mac0".into()));
    state.agents.insert(crate::DeviceId("dev_agent".into()));
    let id = ulid::Ulid::generate();
    let mut task = crate::Task::new(id, "buy pink card stock", "a0");
    task.source = Some("wa:msg-991".into());
    task.created_by = Some(crate::DeviceId("dev_agent".into()));
    task.filled = true;
    state.sourced.insert("wa:msg-991".into(), id);
    state.tasks.insert(id, task);
    state.shed.insert("a-0009".into());
    state.assistants.insert(crate::DeviceId("dev_agent".into()));
    state.forebears.insert("store-was".into());
    state
        .retired
        .insert("attachments/ab/charla-a3f9.mp4".into());

    cache.store(&state, "print").unwrap();
    let back = cache.load("print", true).expect("the cache had it");

    assert_eq!(back.devices, state.devices, "the list of machines was lost");
    assert_eq!(
        back.agents, state.agents,
        "which of them is an agent was lost"
    );
    assert_eq!(
        back.assistants, state.assistants,
        "who ever assisted must outlive the badge"
    );
    assert_eq!(
        back.forebears, state.forebears,
        "the stores this one grew from were lost"
    );
    assert_eq!(
        back.sourced, state.sourced,
        "the index is rebuilt from the tasks themselves, so it cannot outlive one"
    );
    let kept = &back.tasks[&id];
    assert_eq!(kept.source, state.tasks[&id].source);
    assert_eq!(kept.created_by, state.tasks[&id].created_by);
    assert!(
        kept.filled,
        "a backfilled turn came back as an ordinary one"
    );
    assert_eq!(back.retired, state.retired, "the retirements were lost");
    assert_eq!(
        back.shed, state.shed,
        "sin esto el barrido de documentos borrados no correria nunca con la cache caliente"
    );
}

#[test]
fn shelving_a_folder_needs_no_rebuild_because_no_documents_row_changes() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let shelf = Ulid::generate();
    store
        .append(Op::FolderAdd {
            id: shelf,
            d: crate::event::FolderAdd {
                name: "linio".into(),
                order: "a0".into(),
                parent: None,
                icon: None,
                color: None,
            },
        })
        .unwrap();
    let doc = Ulid::generate();
    store
        .append(Op::DocAdd {
            id: doc,
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: "a3f1-0001".into(),
                order: "a0".into(),
                folder: Some(shelf),
                page_of: None,
            },
        })
        .unwrap();

    let mut state = project(&f.store_root, &f.cache_dir).unwrap();
    let mut cache = Cache::open(&f.cache_dir).unwrap();
    let away = store.append(Op::FolderArchive { id: shelf }).unwrap();
    state.apply(&away);
    advance(cache.as_mut(), &state, &[away], &f.store_root, false);

    // The mark lives on the folder, so the documents rows are untouched and the row this
    // does rewrite carries it: reading it back has to agree with replaying the whole log.
    let again = project(&f.store_root, &f.cache_dir).unwrap();
    assert!(
        again.folders[&shelf].archived,
        "the shelved folder came back open"
    );
    assert!(again.stowed(doc), "what it holds came back into the tree");
    assert!(
        !again.docs[&doc].archived,
        "the document was marked one by one"
    );
}

#[test]
fn deleting_a_document_with_pages_leaves_no_page_behind_in_the_cache() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let doc = Ulid::generate();
    let page = Ulid::generate();
    for (id, file, page_of) in [(doc, "a3f1-0001", None), (page, "a3f1-0002", Some(doc))] {
        store
            .append(Op::DocAdd {
                id,
                d: crate::event::DocAdd {
                    wrote: None,
                    guest: false,
                    made: None,
                    by: None,
                    said: None,
                    file: file.into(),
                    order: "a0".into(),
                    folder: None,
                    page_of,
                },
            })
            .unwrap();
    }

    let mut state = project(&f.store_root, &f.cache_dir).unwrap();
    let mut cache = Cache::open(&f.cache_dir).unwrap();
    let gone = store.append(Op::DocDelete { id: doc }).unwrap();
    state.apply(&gone);
    advance(cache.as_mut(), &state, &[gone], &f.store_root, false);

    let again = project(&f.store_root, &f.cache_dir).unwrap();

    assert!(again.docs.is_empty(), "the page outlived its document");
    assert_eq!(again.shed, state.shed, "its file would never be swept");
    assert_eq!(again.docs.get(&page), None);
}

#[test]
fn a_second_launch_still_has_its_folders_and_documents() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let folder = Ulid::generate();
    store
        .append(Op::FolderAdd {
            id: folder,
            d: crate::event::FolderAdd {
                name: "trabajo".into(),
                order: "a0".into(),
                parent: None,
                icon: None,
                color: None,
            },
        })
        .unwrap();
    store
        .append(Op::DocAdd {
            id: Ulid::generate(),
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: "a3f1-0001".into(),
                order: "a0".into(),
                folder: Some(folder),
                page_of: None,
            },
        })
        .unwrap();

    let first = project(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(first.folders.len(), 1, "the log itself lost them");

    let second = project(&f.store_root, &f.cache_dir).unwrap();

    assert_eq!(second.folders.len(), 1, "the tree emptied itself");
    assert_eq!(second.docs.len(), 1, "every document came back unfiled");
    assert_eq!(second.inside(folder).len(), 1);
}

#[test]
fn a_summary_asked_for_on_a_cold_cache_is_as_light_as_on_a_warm_one() {
    let f = loaded();
    let cold = summarised(&f.store_root, &f.cache_dir).unwrap();
    let warm = summarised(&f.store_root, &f.cache_dir).unwrap();

    assert!(
        !cold.has_bodies(),
        "el primer resumen tras vaciar la cache traia el cuerpo entero"
    );
    assert!(
        cold.tasks[&f.task].steps.is_empty() && cold.tasks[&f.task].log.is_empty(),
        "el cuerpo viajo en el resumen"
    );
    assert_eq!(
        cold, warm,
        "el resumen depende de si la cache estaba caliente"
    );
}

#[test]
fn a_summary_knows_how_much_body_it_left_behind() {
    let f = loaded();
    project(&f.store_root, &f.cache_dir).unwrap();

    let light = summarised(&f.store_root, &f.cache_dir).unwrap();
    let task = &light.tasks[&f.task];

    assert!(task.log.is_empty(), "the body came along");
    assert!(task.steps.is_empty(), "the body came along");
    assert_eq!(task.journal_count(), 2);
    assert_eq!(task.steps_done(), (0, 1));
}

#[test]
fn the_full_load_carries_everything_the_log_had() {
    let f = loaded();
    let cached = project(&f.store_root, &f.cache_dir).unwrap();
    let replayed = State::replay(&store::read_all(&f.store_root).unwrap());

    assert_eq!(cached, replayed, "the cache disagrees with the log");
    assert_eq!(cached.tasks[&f.task].log.len(), 2);
}

#[test]
fn a_cache_built_once_is_reused_and_a_deleted_one_is_rebuilt() {
    let f = loaded();
    let first = project(&f.store_root, &f.cache_dir).unwrap();
    assert!(matches!(
        audit(&f.store_root, &f.cache_dir).unwrap(),
        Audit::Agrees { .. }
    ));

    std::fs::remove_dir_all(&f.cache_dir).unwrap();
    assert_eq!(project(&f.store_root, &f.cache_dir).unwrap(), first);
}

#[test]
fn a_log_that_grew_leaves_the_cache_behind() {
    let f = loaded();
    project(&f.store_root, &f.cache_dir).unwrap();

    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("something later", "a1"),
        })
        .unwrap();

    assert!(matches!(
        audit(&f.store_root, &f.cache_dir).unwrap(),
        Audit::Stale { .. }
    ));
    assert_eq!(project(&f.store_root, &f.cache_dir).unwrap().tasks.len(), 2);
}

#[test]
fn tombstones_survive_the_round_trip() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    store.append(Op::TaskDelete { id: f.task }).unwrap();

    let state = project(&f.store_root, &f.cache_dir).unwrap();
    assert!(state.is_erased(f.task));

    let again = project(&f.store_root, &f.cache_dir).unwrap();
    assert!(again.is_erased(f.task), "the cache forgot a deletion");
}

/// An assistant reading the same message again is told the task was let go, and that
/// has to hold on a cache read back as much as on a log replayed.
#[test]
fn the_grave_keeps_what_the_task_was_written_from() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let filed = ulid::Ulid::generate();
    let mut d = crate::event::TaskAdd::new("comprar pan", "a9");
    d.source = Some("wa:msg-4410".into());
    store.append(Op::TaskAdd { id: filed, d }).unwrap();
    store.append(Op::TaskDelete { id: filed }).unwrap();

    let state = project(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(state.sourced.get("wa:msg-4410"), Some(&filed));
    assert!(state.is_erased(filed));

    let again = project(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(
        again.sourced.get("wa:msg-4410"),
        Some(&filed),
        "the cache forgot where an erased task came from"
    );
    let light = summarised(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(light.sourced.get("wa:msg-4410"), Some(&filed));
}

/// Erased, filed again from the same message with `again`, erased again: the cache caught
/// up one event at a time must point the source at the later grave, as a replay does.
#[test]
fn a_source_erased_twice_points_at_its_later_grave_however_the_cache_was_built() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let mut filed = Vec::new();
    for _ in 0..2 {
        let id = ulid::Ulid::generate();
        let mut d = crate::event::TaskAdd::new("comprar pan", "a9");
        d.source = Some("wa:msg-4410".into());
        store.append(Op::TaskAdd { id, d }).unwrap();
        project(&f.store_root, &f.cache_dir).unwrap();
        store.append(Op::TaskDelete { id }).unwrap();
        project(&f.store_root, &f.cache_dir).unwrap();
        filed.push(id);
    }

    let caught_up = summarised(&f.store_root, &f.cache_dir).unwrap();
    let replayed = State::replay(&store.read_all().unwrap());
    assert_eq!(replayed.sourced.get("wa:msg-4410"), Some(&filed[1]));
    assert_eq!(
        caught_up.sourced.get("wa:msg-4410"),
        replayed.sourced.get("wa:msg-4410"),
        "the cache and the log disagree on which grave the source points at"
    );
}

#[test]
fn advancing_leaves_the_cache_current_after_a_write() {
    let f = loaded();
    let mut state = project(&f.store_root, &f.cache_dir).unwrap();
    let mut cache = Cache::open(&f.cache_dir).unwrap();

    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let event = store
        .append(Op::TaskDone {
            id: f.task,
            filled: false,
        })
        .unwrap();
    state.apply(&event);

    let print = advance(
        cache.as_mut(),
        &state,
        std::slice::from_ref(&event),
        &f.store_root,
        false,
    );

    assert_eq!(print, fingerprint(&f.store_root));
    assert!(
        matches!(
            audit(&f.store_root, &f.cache_dir).unwrap(),
            Audit::Agrees { .. }
        ),
        "the cache fell behind the log"
    );
}

#[test]
fn a_summary_that_writes_never_erases_the_bodies_it_left_behind() {
    let f = loaded();
    project(&f.store_root, &f.cache_dir).unwrap();

    let mut light = summarised(&f.store_root, &f.cache_dir).unwrap();
    let mut cache = Cache::open(&f.cache_dir).unwrap();

    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let event = store
        .append(Op::TaskDone {
            id: f.task,
            filled: false,
        })
        .unwrap();
    light.apply(&event);
    advance(
        cache.as_mut(),
        &light,
        std::slice::from_ref(&event),
        &f.store_root,
        false,
    );

    let whole = project(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(
        whole,
        State::replay(&store::read_all(&f.store_root).unwrap()),
        "the cache kept a body-less state and called it fresh"
    );
    assert_eq!(whole.tasks[&f.task].log.len(), 2, "the journal was erased");
    assert_eq!(whole.tasks[&f.task].steps.len(), 1, "the steps were erased");
}

#[test]
fn a_summary_keeps_the_volume_it_was_handed() {
    let f = loaded();
    project(&f.store_root, &f.cache_dir).unwrap();

    let mut light = summarised(&f.store_root, &f.cache_dir).unwrap();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let event = store
        .append(Op::TaskDone {
            id: f.task,
            filled: false,
        })
        .unwrap();
    light.apply(&event);

    let task = &light.tasks[&f.task];
    assert_eq!(task.journal_count(), 2, "the volume was recounted from air");
    assert_eq!(task.steps_done(), (0, 1));
}

/// The tombstone table gained a column this schema: a cache written before it must be
/// rebuilt whole, or every write into it fails and the cache never loads again.
#[test]
fn a_cache_from_an_older_schema_is_rebuilt_rather_than_left_dead() {
    let f = loaded();
    std::fs::create_dir_all(&f.cache_dir).unwrap();
    let db = Connection::open(f.cache_dir.join("read.db")).unwrap();
    db.execute_batch(
        "CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
         CREATE TABLE tombstone(id TEXT PRIMARY KEY);
         INSERT INTO meta VALUES ('schema', '1');",
    )
    .unwrap();
    drop(db);

    project(&f.store_root, &f.cache_dir).unwrap();

    let print = fingerprint(&f.store_root);
    let cache = Cache::open(&f.cache_dir).unwrap().unwrap();
    assert_eq!(
        cache.meta("schema").as_deref(),
        Some(SCHEMA.to_string().as_str())
    );
    assert!(
        cache.load(&print, true).is_some(),
        "the cache loads again after the rebuild"
    );
}

#[test]
fn a_gist_outlives_a_schema_that_moved() {
    let f = loaded();
    let cache = Cache::open(&f.cache_dir).unwrap().unwrap();
    let said = crate::docs::Gist {
        print: "abc".into(),
        summary: "the roof, and who pays for it".into(),
        notes: String::new(),
        at: jiff::Timestamp::UNIX_EPOCH,
        by: None,
    };
    assert!(cache.note_gist("mac0-0001", &said));
    let card = crate::docs::Card::read_from("# Roof\n\nwho pays\n");
    cache.note_card("mac0-0001", (7, 7), &card, "roof who pays");
    assert!(cache.card("mac0-0001", (7, 7)).is_some());
    drop(cache);

    let db = Connection::open(f.cache_dir.join("read.db")).unwrap();
    db.execute("INSERT OR REPLACE INTO meta VALUES ('schema', '1')", [])
        .unwrap();
    drop(db);

    let cache = Cache::open(&f.cache_dir).unwrap().unwrap();
    assert!(
        cache.card("mac0-0001", (7, 7)).is_none(),
        "the rebuild did not run, so nothing was put to the test"
    );
    assert_eq!(
        cache.gist("mac0-0001").map(|one| one.summary),
        Some(said.summary),
        "the rebuild threw the agents' margin away"
    );
}

#[test]
fn a_gist_whose_document_is_gone_is_forgotten_even_without_a_card() {
    let f = loaded();
    let cache = Cache::open(&f.cache_dir).unwrap().unwrap();
    let said = crate::docs::Gist {
        print: "abc".into(),
        summary: "gone".into(),
        notes: String::new(),
        at: jiff::Timestamp::UNIX_EPOCH,
        by: None,
    };
    assert!(cache.note_gist("mac0-0009", &said));

    cache.forget_cards(&std::collections::BTreeSet::new());
    assert!(cache.gist("mac0-0009").is_none());
}

#[test]
fn a_summary_keeps_the_conversion_it_was_handed() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskDone {
            id: f.task,
            filled: false,
        })
        .unwrap();
    store
        .append(Op::TaskUpdate {
            id: f.task,
            d: crate::event::TaskPatch {
                read_as: Some(Some(crate::Reading::Trace)),
                ..Default::default()
            },
        })
        .unwrap();
    project(&f.store_root, &f.cache_dir).unwrap();

    let mut light = summarised(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(light.tasks[&f.task].read_as, Some(crate::Reading::Trace));
    assert_eq!(light.tasks[&f.task].reading(), crate::Reading::Trace);

    let event = store
        .append(Op::TaskUpdate {
            id: f.task,
            d: crate::event::TaskPatch {
                title: Some("retitled while light".into()),
                ..Default::default()
            },
        })
        .unwrap();
    light.apply(&event);
    assert_eq!(
        light.tasks[&f.task].read_as,
        Some(crate::Reading::Trace),
        "a patch in summary mode does not shake the conversion off"
    );
}

#[test]
fn a_cache_remembers_where_each_agent_lives() {
    let f = loaded();
    let agent = DeviceId("dev_agent".into());
    let mut store = Store::open(&f.store_root, agent.clone()).unwrap();
    store
        .append_batch(vec![
            Op::DeviceJoin {
                d: agent.clone(),
                k: Some(crate::event::DeviceKind::Agent),
            },
            Op::DeviceHost {
                d: agent.clone(),
                of: DeviceId("dev_a".into()),
            },
        ])
        .unwrap();
    project(&f.store_root, &f.cache_dir).unwrap();

    let light = summarised(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(light.hosts.get(&agent), Some(&DeviceId("dev_a".into())));
}

#[test]
fn a_host_said_in_a_tail_is_still_known_at_the_next_open() {
    let f = loaded();
    let agent = DeviceId("dev_agent".into());
    Store::open(&f.store_root, agent.clone())
        .unwrap()
        .append(Op::DeviceJoin {
            d: agent.clone(),
            k: Some(crate::event::DeviceKind::Agent),
        })
        .unwrap();
    project(&f.store_root, &f.cache_dir).unwrap();

    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::DeviceHost {
            d: agent.clone(),
            of: DeviceId("dev_a".into()),
        })
        .unwrap();
    store
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("after the host was said", "a1"),
        })
        .unwrap();

    let caught = project(&f.store_root, &f.cache_dir).unwrap();
    let opened = project(&f.store_root, &f.cache_dir).unwrap();
    let light = summarised(&f.store_root, &f.cache_dir).unwrap();
    assert_eq!(caught.hosts.get(&agent), Some(&DeviceId("dev_a".into())));
    assert_eq!(
        opened.hosts, caught.hosts,
        "a tail applied on a warm cache must leave the host where a replay would"
    );
    assert_eq!(light.hosts, caught.hosts);
}

#[test]
fn a_summary_keeps_the_door_to_agents_it_was_handed() {
    let f = loaded();
    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskUpdate {
            id: f.task,
            d: crate::event::TaskPatch {
                open_to_agents: Some(true),
                ..Default::default()
            },
        })
        .unwrap();
    project(&f.store_root, &f.cache_dir).unwrap();

    let light = summarised(&f.store_root, &f.cache_dir).unwrap();
    assert!(light.tasks[&f.task].open_to_agents);
    assert!(light.attended_by_agents(&light.tasks[&f.task]));
}

#[test]
fn a_write_that_is_not_carried_leaves_the_cache_behind() {
    let f = loaded();
    project(&f.store_root, &f.cache_dir).unwrap();

    let mut store = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    store
        .append(Op::TaskDone {
            id: f.task,
            filled: false,
        })
        .unwrap();

    assert!(matches!(
        audit(&f.store_root, &f.cache_dir).unwrap(),
        Audit::Stale { .. }
    ));
}

#[test]
fn a_state_that_was_overtaken_is_thrown_away_instead_of_carried() {
    let f = loaded();
    let state = project(&f.store_root, &f.cache_dir).unwrap();

    let mut theirs = Store::open(&f.store_root, DeviceId("dev_a".into())).unwrap();
    let event = theirs
        .append(Op::TaskDone {
            id: f.task,
            filled: false,
        })
        .unwrap();

    let mut cache = Cache::open(&f.cache_dir).unwrap();
    advance(
        cache.as_mut(),
        &state,
        std::slice::from_ref(&event),
        &f.store_root,
        true,
    );

    let fresh = Cache::open(&f.cache_dir).unwrap().unwrap();
    assert!(
        fresh.load(&fingerprint(&f.store_root), true).is_none(),
        "a cache written from an overtaken state must not answer as fresh"
    );
}

#[test]
fn one_file_growing_is_told_apart_on_either_platform() {
    let mac = |n: u64, m: u64| {
        format!(
            "/Users/mario/Library/Tisty/store/dev_a/0001.tisty:{n}|/Users/mario/Library/Tisty/store/dev_a/active.tisty:{m}"
        )
    };
    let (at, from) = grown(&mac(8330, 3936), &mac(8330, 4100)).expect("a tail on macOS");
    assert_eq!(from, 3936);
    assert_eq!(
        at.to_str(),
        Some("/Users/mario/Library/Tisty/store/dev_a/active.tisty")
    );

    let win = |n: u64, m: u64| {
        format!(
            "D:\\Code\\Tisty\\store\\dev_a\\0001.tisty:{n}|D:\\Code\\Tisty\\store\\dev_a\\active.tisty:{m}"
        )
    };
    let (at, from) =
        grown(&win(8330, 3936), &win(8330, 4100)).expect("a drive letter is not a separator");
    assert_eq!(from, 3936);
    assert_eq!(at.to_str(), Some(r"D:\Code\Tisty\store\dev_a\active.tisty"));
}

#[test]
fn anything_but_one_file_growing_is_refused() {
    let two = |n: u64, m: u64| format!("/s/a.tisty:{n}|/s/b.tisty:{m}");
    assert!(
        grown(&two(10, 20), &two(11, 21)).is_none(),
        "two files changed"
    );
    assert!(
        grown(&two(10, 20), &two(10, 19)).is_none(),
        "a file that shrank"
    );
    assert!(
        grown(&two(10, 20), &two(10, 20)).is_none(),
        "nothing changed"
    );
    assert!(
        grown(&two(10, 20), "/s/a.tisty:10|/s/b.tisty:20|/s/c.tisty:5").is_none(),
        "a file appeared"
    );
    assert!(grown("", &two(10, 20)).is_none(), "no cache to catch up");
    assert!(
        grown("/s/a.tisty:0", "/s/a.tisty:9").is_none(),
        "a file that was empty has no line to land after"
    );
}

#[test]
fn a_key_compares_as_its_parts_do() {
    let one = |at: i64, who: &str, seq: u64| {
        let mut event = crate::Event::new(
            crate::event::DeviceId(who.into()),
            jiff::Timestamp::from_second(at).unwrap(),
            crate::Op::TaskReopen {
                id: ulid::Ulid::generate(),
            },
        );
        event.seq = seq;
        keyed(&event).unwrap()
    };
    assert!(
        one(10, "dev_a", 1) < one(11, "dev_a", 1),
        "an earlier second sorts first"
    );
    assert!(
        one(10, "dev_a", 1) < one(10, "dev_b", 1),
        "a tie breaks by device"
    );
    assert!(
        one(10, "dev_a", 2) < one(10, "dev_a", 30),
        "seq is not compared as text"
    );
    assert!(
        one(10, "dev_a", 9) < one(10, "dev_b", 1),
        "device outranks seq"
    );
    assert!(
        one(10, "dev_a", 1) < one(10, "dev_a1", 1),
        "a device id that is another's prefix must not invert the order"
    );
}

#[test]
fn the_mark_only_ever_climbs() {
    let room = tempfile::tempdir().unwrap();
    let mut cache = Cache::open(room.path()).unwrap().unwrap();
    cache.mark("b");
    cache.mark("a");
    assert_eq!(
        cache.meta("last_key").as_deref(),
        Some("b"),
        "a batch of fresh events must not lower what the cache already holds"
    );
    cache.mark("c");
    assert_eq!(cache.meta("last_key").as_deref(), Some("c"));
}
