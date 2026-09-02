use std::path::PathBuf;

use ulid::Ulid;

use tisty_core::cache::{self, Cache};
use tisty_core::event::{DeviceId, DocAdd, Filed, FolderAdd};
use tisty_core::model::{DocId, FolderId};
use tisty_core::{Event, Op, State, Store, docs, order, undo};

struct World {
    _tmp: tempfile::TempDir,
    store_root: PathBuf,
    docs_root: PathBuf,
    cache_dir: PathBuf,
}

impl World {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let docs_root = tmp.path().join("docs");
        let cache_dir = tmp.path().join("cache");
        Self {
            _tmp: tmp,
            store_root,
            docs_root,
            cache_dir,
        }
    }

    fn store(&self, device: &str) -> Store {
        Store::open(&self.store_root, DeviceId(device.into())).unwrap()
    }
}

fn replayed(world: &World) -> State {
    State::replay(&tisty_core::store::read_all(&world.store_root).unwrap())
}

fn t(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

fn doc_add(
    store: &mut Store,
    file: &str,
    order: &str,
    folder: Option<FolderId>,
    page_of: Option<DocId>,
) -> DocId {
    let id = Ulid::generate();
    store
        .append(Op::DocAdd {
            id,
            d: DocAdd {
                file: file.into(),
                order: order.into(),
                folder,
                page_of,
            },
        })
        .unwrap();
    id
}

fn folder_add(store: &mut Store, name: &str, parent: Option<FolderId>) -> FolderId {
    let id = Ulid::generate();
    store
        .append(Op::FolderAdd {
            id,
            d: FolderAdd {
                name: name.into(),
                order: "a0".into(),
                parent,
                icon: None,
                color: None,
            },
        })
        .unwrap();
    id
}

fn make(
    world: &World,
    store: &mut Store,
    body: &str,
    folder: Option<FolderId>,
    page_of: Option<DocId>,
    order: &str,
) -> (DocId, String) {
    let made = docs::create(&world.docs_root, store.device(), body).unwrap();
    let id = Ulid::generate();
    store
        .append(Op::DocAdd {
            id,
            d: DocAdd {
                file: made.id.clone(),
                order: order.into(),
                folder,
                page_of,
            },
        })
        .unwrap();
    (id, made.id)
}

fn body_of_exactly(bytes: u64, marker: &str) -> String {
    let head = format!("{marker}\n");
    let filler_len = bytes as usize - head.len() - 1;
    format!("{head}{}\n", "x".repeat(filler_len))
}

fn sorted_merge(mut a: Vec<Event>, b: Vec<Event>) -> Vec<Event> {
    a.extend(b);
    a.sort_by(|x, y| x.sort_key().cmp(&y.sort_key()));
    a.dedup_by(|x, y| x.sort_key() == y.sort_key());
    a
}

#[test]
fn ten_heavy_pages_all_read_back_intact_and_replay_agrees() {
    let world = World::new();
    let mut store = world.store("dev_a");

    let parent_body = format!("# Diario de campo\n\n{}\n", "p".repeat(4_000));
    let (parent, parent_file) = make(&world, &mut store, &parent_body, None, None, "a0");

    let mut pages = Vec::new();
    let mut order = "a0".to_string();
    for i in 0..10 {
        let body = body_of_exactly(docs::BODY_AT_MOST, &format!("# page {i:02}"));
        order = tisty_core::order::after(&order);
        let (id, file) = make(&world, &mut store, &body, None, Some(parent), &order);
        pages.push((id, file, body));
    }

    assert_eq!(
        docs::read(&world.docs_root, &parent_file).unwrap(),
        parent_body,
        "the parent's body must not move"
    );

    for (_, file, body) in &pages {
        let read = docs::read(&world.docs_root, file).unwrap();
        assert_eq!(
            read.len() as u64,
            docs::BODY_AT_MOST,
            "the page kept its whole weight"
        );
        assert_eq!(&read, body, "no page overwrote another");
    }

    let names: std::collections::BTreeSet<&str> = std::iter::once(parent_file.as_str())
        .chain(pages.iter().map(|(_, f, _)| f.as_str()))
        .collect();
    assert_eq!(names.len(), 11, "every file got a distinct name");

    let state = replayed(&world);
    assert_eq!(state.docs.len(), 11);
    assert_eq!(state.pages_of(parent).len(), 10);
    for (id, ..) in &pages {
        assert_eq!(state.docs[id].page_of, Some(parent));
    }

    assert_eq!(state, replayed(&world), "replaying twice must agree");
}

#[test]
fn deleting_the_parent_sheds_all_its_pages_and_they_can_be_swept() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let (parent, parent_file) = make(&world, &mut store, "# root\n\nhola\n", None, None, "a0");

    let mut files = vec![parent_file];
    for i in 0..10 {
        let (_, file) = make(
            &world,
            &mut store,
            &format!("# page {i}\n\ncontenido\n"),
            None,
            Some(parent),
            &format!("a{i}"),
        );
        files.push(file);
    }

    store.append(Op::DocDelete { id: parent }).unwrap();
    let state = replayed(&world);

    assert!(state.docs.is_empty(), "a page is part of its document");
    for file in &files {
        assert!(state.shed.contains(file), "{file} was not shed");
    }

    let swept = docs::sweep(&world.docs_root, &state.shed);
    assert_eq!(swept, files.len());
    for file in &files {
        assert!(
            docs::read(&world.docs_root, file).is_err(),
            "{file} should be gone"
        );
    }
}

#[test]
fn moving_the_document_drags_its_pages_but_moving_a_page_alone_does_nothing() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let work = folder_add(&mut store, "trabajo", None);
    let home = folder_add(&mut store, "casa", None);
    let doc = doc_add(&mut store, "a3f1-0001", "a0", Some(work), None);
    let page = doc_add(&mut store, "a3f1-0002", "a0", None, Some(doc));

    store
        .append(Op::DocMove {
            id: doc,
            d: Filed {
                folder: Some(Some(home)),
                page_of: None,
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);
    assert_eq!(state.docs[&doc].folder, Some(home));
    assert_eq!(
        state.docs[&page].folder,
        Some(home),
        "the page followed its document"
    );

    store
        .append(Op::DocMove {
            id: page,
            d: Filed {
                folder: Some(Some(work)),
                page_of: None,
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);
    assert_eq!(
        state.docs[&page].folder,
        Some(home),
        "a page cannot be filed away from its document"
    );
    assert_eq!(state.docs[&doc].folder, Some(home));
}

#[test]
fn converting_between_document_and_page_and_back_twice_settles_on_the_same_shape() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let work = folder_add(&mut store, "trabajo", None);
    let parent = doc_add(&mut store, "a3f1-0001", "a0", Some(work), None);
    let loose = doc_add(&mut store, "a3f1-0002", "z9", None, None);

    store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(Some(parent)),
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);
    assert_eq!(state.docs[&loose].page_of, Some(parent));
    assert_eq!(
        state.docs[&loose].folder,
        Some(work),
        "it inherited its new parent's folder"
    );

    store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(None),
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);
    assert_eq!(state.docs[&loose].page_of, None);
    let after_one_round_trip = state.docs[&loose].clone();

    store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(Some(parent)),
                order: None,
            },
        })
        .unwrap();
    store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(None),
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);

    assert_eq!(
        state.docs[&loose], after_one_round_trip,
        "two round trips settle where one did"
    );
    assert_eq!(
        state.docs[&loose].folder,
        Some(work),
        "it keeps the folder it inherited as a page, even once independent again"
    );
}

#[test]
fn deleting_one_page_leaves_the_document_and_its_siblings_intact() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let parent = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let p1 = doc_add(&mut store, "a3f1-0002", "a0", None, Some(parent));
    let p2 = doc_add(&mut store, "a3f1-0003", "a1", None, Some(parent));
    let p3 = doc_add(&mut store, "a3f1-0004", "a2", None, Some(parent));

    store.append(Op::DocDelete { id: p2 }).unwrap();
    let state = replayed(&world);

    assert!(state.docs.contains_key(&parent));
    assert!(state.docs.contains_key(&p1));
    assert!(state.docs.contains_key(&p3));
    assert!(!state.docs.contains_key(&p2));
    assert_eq!(
        state
            .pages_of(parent)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![p1, p3]
    );
    assert!(state.shed.contains("a3f1-0003"));
}

#[test]
fn moving_a_page_into_a_document_in_another_folder_refiles_it_last_among_its_new_siblings() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let a = folder_add(&mut store, "a", None);
    let b = folder_add(&mut store, "b", None);
    let doc1 = doc_add(&mut store, "a3f1-0001", "a0", Some(a), None);
    let doc2 = doc_add(&mut store, "a3f1-0002", "a0", Some(b), None);
    let sibling = doc_add(&mut store, "a3f1-0003", "a0", None, Some(doc2));
    let p1 = doc_add(&mut store, "a3f1-0004", "a0", None, Some(doc1));
    let p2 = doc_add(&mut store, "a3f1-0005", "a1", None, Some(doc1));

    store
        .append(Op::DocMove {
            id: p1,
            d: Filed {
                folder: None,
                page_of: Some(Some(doc2)),
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);

    assert_eq!(state.docs[&p1].page_of, Some(doc2));
    assert_eq!(
        state.docs[&p1].folder,
        Some(b),
        "it took its new document's folder"
    );
    assert_eq!(
        state
            .pages_of(doc2)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![sibling, p1],
        "it landed last among its new siblings"
    );
    assert_eq!(
        state
            .pages_of(doc1)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![p2],
        "its old document keeps the rest"
    );
}

#[test]
fn mutual_page_of_between_two_documents_never_forms_in_either_order() {
    for reversed in [false, true] {
        let world = World::new();
        let mut store = world.store("dev_a");
        let a = doc_add(&mut store, "a3f1-0001", "a0", None, None);
        let b = doc_add(&mut store, "a3f1-0002", "a0", None, None);
        let (first, second) = if reversed { (b, a) } else { (a, b) };

        store
            .append(Op::DocMove {
                id: first,
                d: Filed {
                    folder: None,
                    page_of: Some(Some(second)),
                    order: None,
                },
            })
            .unwrap();
        store
            .append(Op::DocMove {
                id: second,
                d: Filed {
                    folder: None,
                    page_of: Some(Some(first)),
                    order: None,
                },
            })
            .unwrap();

        let state = replayed(&world);
        assert_eq!(state.docs[&first].page_of, Some(second));
        assert_eq!(
            state.docs[&second].page_of, None,
            "a mutual page_of would be a cycle, reversed={reversed}"
        );
    }
}

#[test]
fn a_document_cannot_become_a_page_of_itself_at_creation_or_by_moving() {
    let world = World::new();
    let mut store = world.store("dev_a");

    let itself = Ulid::generate();
    store
        .append(Op::DocAdd {
            id: itself,
            d: DocAdd {
                file: "a3f1-0001".into(),
                order: "a0".into(),
                folder: None,
                page_of: Some(itself),
            },
        })
        .unwrap();
    let state = replayed(&world);
    assert_eq!(
        state.docs[&itself].page_of, None,
        "it could not be its own parent before it existed"
    );

    store
        .append(Op::DocMove {
            id: itself,
            d: Filed {
                folder: None,
                page_of: Some(Some(itself)),
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);
    assert_eq!(state.docs[&itself].page_of, None, "nor after, by moving");
}

#[test]
fn a_page_of_a_page_is_refused_at_creation_and_by_moving() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let root = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let mid = doc_add(&mut store, "a3f1-0002", "a0", None, Some(root));

    let deep = doc_add(&mut store, "a3f1-0003", "a0", None, Some(mid));
    let state = replayed(&world);
    assert_eq!(
        state.docs[&deep].page_of, None,
        "a page of a page is kept as a document, at creation"
    );

    let loose = doc_add(&mut store, "a3f1-0004", "a1", None, None);
    store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(Some(mid)),
                order: None,
            },
        })
        .unwrap();
    let state = replayed(&world);
    assert_eq!(
        state.docs[&loose].page_of, None,
        "moving under a page is refused the same way"
    );
}

#[test]
fn undoing_a_creation_and_a_plain_move_restores_the_exact_state_before() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let work = folder_add(&mut store, "trabajo", None);
    let home = folder_add(&mut store, "casa", None);
    doc_add(&mut store, "a3f1-0001", "a0", Some(work), None);

    let baseline = replayed(&world);
    let doc = *baseline.docs.keys().next().unwrap();

    let moved = store
        .append(Op::DocMove {
            id: doc,
            d: Filed {
                folder: Some(Some(home)),
                page_of: None,
                order: None,
            },
        })
        .unwrap();
    let undo_move = undo::inverse(&moved, &baseline).unwrap();
    store.append_batch(undo_move).unwrap();
    assert_eq!(
        replayed(&world),
        baseline,
        "undoing the move restores the folder"
    );

    let created = store
        .append(Op::DocAdd {
            id: Ulid::generate(),
            d: DocAdd {
                file: "a3f1-0002".into(),
                order: "a1".into(),
                folder: None,
                page_of: None,
            },
        })
        .unwrap();
    let undo_create = undo::inverse(&created, &baseline).unwrap();
    store.append_batch(undo_create).unwrap();
    assert_eq!(
        replayed(&world).docs,
        baseline.docs,
        "undoing the creation restores it too"
    );
}

#[test]
fn undoing_a_page_of_conversion_puts_the_order_back_too() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let parent = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let sibling_page = doc_add(&mut store, "a3f1-0002", "a0", None, Some(parent));
    let loose = doc_add(&mut store, "a3f1-0003", "z9", None, None);

    let baseline = replayed(&world);
    assert_eq!(baseline.docs[&loose].order, "z9");

    let converted = store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(Some(parent)),
                order: None,
            },
        })
        .unwrap();
    let undo_op = undo::inverse(&converted, &baseline).unwrap();
    store.append_batch(undo_op).unwrap();

    let after = replayed(&world);
    assert_eq!(after.docs[&sibling_page], baseline.docs[&sibling_page]);
    assert_eq!(
        after, baseline,
        "the order should have come back with everything else"
    );
}

#[test]
fn the_hot_cache_agrees_with_memory_through_every_cascading_operation_on_a_document_with_pages() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let work = folder_add(&mut store, "trabajo", None);
    let home = folder_add(&mut store, "casa", None);
    let doc = doc_add(&mut store, "a3f1-0001", "a0", Some(work), None);
    let pages: Vec<DocId> = (0..3)
        .map(|i| {
            doc_add(
                &mut store,
                &format!("a3f1-000{}", i + 2),
                &format!("a{i}"),
                None,
                Some(doc),
            )
        })
        .collect();

    let mut state = cache::project(&world.store_root, &world.cache_dir).unwrap();

    let steps = [
        Op::DocArchive { id: doc },
        Op::DocUnarchive { id: doc },
        Op::DocMove {
            id: doc,
            d: Filed {
                folder: Some(Some(home)),
                page_of: None,
                order: None,
            },
        },
        Op::DocDelete { id: doc },
    ];

    for op in steps {
        let event = store.append(op).unwrap();
        state.apply(&event);

        let mut cache = Cache::open(&world.cache_dir).unwrap();
        cache::advance(cache.as_mut(), &state, &[event], &world.store_root, false);

        let reprojected = cache::project(&world.store_root, &world.cache_dir).unwrap();

        assert_eq!(
            reprojected.docs, state.docs,
            "docs disagree after a cascading step"
        );
        assert_eq!(
            reprojected.shed, state.shed,
            "shed disagrees after a cascading step"
        );
        assert_eq!(
            reprojected.docs.values().filter(|k| k.archived).count(),
            state.docs.values().filter(|k| k.archived).count(),
            "archived count disagrees after a cascading step"
        );
    }

    assert!(
        pages.iter().all(|p| !state.docs.contains_key(p)),
        "the pages were deleted along with the document"
    );
}

#[test]
fn two_stores_that_diverge_over_a_page_and_a_deleted_parent_converge_regardless_of_merge_order() {
    let tmp = tempfile::tempdir().unwrap();
    let root_a = tmp.path().join("a");
    let root_b = tmp.path().join("b");
    let mut store_a = Store::open(&root_a, DeviceId("dev_a".into())).unwrap();
    let mut store_b = Store::open(&root_b, DeviceId("dev_b".into())).unwrap();

    let parent = Ulid::generate();
    let page = Ulid::generate();
    let dev_a = DeviceId("dev_a".into());
    let dev_b = DeviceId("dev_b".into());

    store_a
        .append_event(&Event::new(
            dev_a.clone(),
            t(1),
            Op::DocAdd {
                id: parent,
                d: DocAdd {
                    file: "a3f1-0001".into(),
                    order: "a0".into(),
                    folder: None,
                    page_of: None,
                },
            },
        ))
        .unwrap();
    store_b
        .append_event(&Event::new(dev_b, t(2), Op::DocDelete { id: parent }))
        .unwrap();
    store_a
        .append_event(&Event::new(
            dev_a,
            t(3),
            Op::DocAdd {
                id: page,
                d: DocAdd {
                    file: "a3f1-0002".into(),
                    order: "a0".into(),
                    folder: None,
                    page_of: Some(parent),
                },
            },
        ))
        .unwrap();

    let events_a = store_a.read_all().unwrap();
    let events_b = store_b.read_all().unwrap();

    let merged_ab = sorted_merge(events_a.clone(), events_b.clone());
    let merged_ba = sorted_merge(events_b, events_a);

    let state_ab = State::replay(&merged_ab);
    let state_ba = State::replay(&merged_ba);

    assert_eq!(state_ab, state_ba, "merging in either order must converge");
    assert!(
        !state_ab.docs.contains_key(&parent),
        "the parent stayed deleted"
    );
    assert_eq!(
        state_ab.docs[&page].page_of, None,
        "the parent no longer existed when this page was replayed, so it became its own document"
    );
    assert!(state_ab.shed.contains("a3f1-0001"));
    assert!(
        !state_ab.shed.contains("a3f1-0002"),
        "the page itself was never deleted, only orphaned"
    );
}

#[test]
fn a_page_written_with_a_timestamp_before_its_parent_is_kept_as_its_own_document() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let device = DeviceId("dev_a".into());
    let parent = Ulid::generate();
    let page = Ulid::generate();

    store
        .append_event(&Event::new(
            device.clone(),
            t(2_000),
            Op::DocAdd {
                id: page,
                d: DocAdd {
                    file: "a3f1-0002".into(),
                    order: "a0".into(),
                    folder: None,
                    page_of: Some(parent),
                },
            },
        ))
        .unwrap();
    store
        .append_event(&Event::new(
            device,
            t(3_000),
            Op::DocAdd {
                id: parent,
                d: DocAdd {
                    file: "a3f1-0001".into(),
                    order: "a0".into(),
                    folder: None,
                    page_of: None,
                },
            },
        ))
        .unwrap();

    let state = replayed(&world);

    assert_eq!(state.docs.len(), 2);
    assert_eq!(state.docs[&parent].page_of, None);
    assert_eq!(
        state.docs[&page].page_of, None,
        "the parent did not exist yet when this event was replayed, in timestamp order"
    );
}

#[test]
fn archiving_and_unarchiving_the_document_reaches_its_pages_but_a_loose_page_ignores_archiving() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let doc = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let page = doc_add(&mut store, "a3f1-0002", "a0", None, Some(doc));

    store.append(Op::DocArchive { id: doc }).unwrap();
    let state = replayed(&world);
    assert!(state.docs[&doc].archived);
    assert!(state.docs[&page].archived);

    store.append(Op::DocUnarchive { id: doc }).unwrap();
    let state = replayed(&world);
    assert!(!state.docs[&doc].archived);
    assert!(!state.docs[&page].archived);

    store.append(Op::DocArchive { id: page }).unwrap();
    let state = replayed(&world);
    assert!(
        !state.docs[&page].archived,
        "a page cannot be archived on its own"
    );
    assert!(!state.docs[&doc].archived);
}

#[test]
fn a_document_that_holds_pages_refuses_to_become_a_page_of_another() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let holder = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    doc_add(&mut store, "a3f1-0002", "a0", None, Some(holder));
    let other = doc_add(&mut store, "a3f1-0003", "a0", None, None);

    store
        .append(Op::DocMove {
            id: holder,
            d: Filed {
                folder: None,
                page_of: Some(Some(other)),
                order: None,
            },
        })
        .unwrap();

    let state = replayed(&world);
    assert_eq!(
        state.docs[&holder].page_of, None,
        "it still holds a page, so it cannot become one"
    );
}

#[test]
fn two_hundred_pages_keep_a_stable_order_across_replays_and_projections() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let parent = doc_add(&mut store, "a3f1-0001", "a0", None, None);

    let mut key = "a0".to_string();
    let mut ids = Vec::with_capacity(200);
    for i in 0..200 {
        key = order::after(&key);
        let id = doc_add(
            &mut store,
            &format!("a3f1-{:04}", i + 2),
            &key,
            None,
            Some(parent),
        );
        ids.push(id);
    }

    let first = replayed(&world)
        .pages_of(parent)
        .iter()
        .map(|k| k.id)
        .collect::<Vec<_>>();
    assert_eq!(
        first, ids,
        "insertion order, since the orders strictly increase"
    );

    let second = replayed(&world)
        .pages_of(parent)
        .iter()
        .map(|k| k.id)
        .collect::<Vec<_>>();
    assert_eq!(second, first, "a fresh replay keeps the same order");

    let projected_once = cache::project(&world.store_root, &world.cache_dir).unwrap();
    let projected_twice = cache::project(&world.store_root, &world.cache_dir).unwrap();
    assert_eq!(
        projected_once
            .pages_of(parent)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        ids
    );
    assert_eq!(
        projected_twice
            .pages_of(parent)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        ids
    );
}

#[test]
fn an_empty_body_still_creates_a_document_with_an_empty_title() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let (id, file) = make(&world, &mut store, "", None, None, "a0");

    assert_eq!(docs::titled(""), "");
    assert_eq!(docs::read(&world.docs_root, &file).unwrap(), "");
    let state = replayed(&world);
    assert_eq!(state.docs[&id].file, file);
}

#[test]
fn a_title_written_with_an_emoji_survives_the_round_trip_through_disk() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let body = "# \u{1F4C4} Notas de la reuni\u{f3}n\n\ncontenido\n";
    let (_, file) = make(&world, &mut store, body, None, None, "a0");

    assert_eq!(docs::titled(body), "\u{1F4C4} Notas de la reuni\u{f3}n");
    assert_eq!(docs::read(&world.docs_root, &file).unwrap(), body);
}

#[test]
fn a_document_whose_file_is_missing_from_disk_fails_to_read_but_the_state_and_sweep_stay_calm() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let ghost = doc_add(&mut store, "a3f1-0001", "a0", None, None);

    let state = replayed(&world);
    assert!(state.docs.contains_key(&ghost));
    assert!(
        docs::read(&world.docs_root, "a3f1-0001").is_err(),
        "no file was ever written for it"
    );

    store.append(Op::DocDelete { id: ghost }).unwrap();
    let state = replayed(&world);
    assert!(state.shed.contains("a3f1-0001"));

    let swept = docs::sweep(&world.docs_root, &state.shed);
    assert_eq!(swept, 0, "there was nothing on disk to remove");
}

#[test]
fn a_page_whose_parent_was_never_created_becomes_its_own_document_in_its_own_folder() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let work = folder_add(&mut store, "trabajo", None);
    let ghost_parent = Ulid::generate();
    let page = doc_add(
        &mut store,
        "a3f1-0001",
        "a0",
        Some(work),
        Some(ghost_parent),
    );

    let state = replayed(&world);
    assert_eq!(
        state.docs[&page].page_of, None,
        "an unknown parent cannot hold a page"
    );
    assert_eq!(
        state.docs[&page].folder,
        Some(work),
        "its own folder was honoured instead"
    );
}

fn moved(store: &mut Store, id: DocId, order: &str) {
    store
        .append(Op::DocMove {
            id,
            d: Filed {
                folder: None,
                page_of: None,
                order: Some(order.into()),
            },
        })
        .unwrap();
}

fn named_in(pages: &[&str]) -> String {
    pages
        .iter()
        .map(|file| format!("![Una](tisty:doc/{file})\n\n"))
        .collect()
}

#[test]
fn the_pages_a_document_names_are_ordered_the_way_it_names_them() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    let three = doc_add(&mut store, "a3f1-0004", "a2", None, Some(book));

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, &named_in(&["a3f1-0004", "a3f1-0002", "a3f1-0003"])) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    let told: Vec<&str> = state
        .pages_of(book)
        .iter()
        .map(|one| one.file.as_str())
        .collect();
    assert_eq!(told, ["a3f1-0004", "a3f1-0002", "a3f1-0003"]);
    assert_eq!([one, two, three].len(), 3);
}

#[test]
fn a_body_that_names_its_pages_in_order_moves_nothing() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));

    let state = replayed(&world);
    assert!(
        state
            .pages_told(book, &named_in(&["a3f1-0002", "a3f1-0003"]))
            .is_empty(),
        "a save that changes nothing must not write to the log"
    );
}

#[test]
fn a_page_the_body_never_names_keeps_the_place_it_had() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let loose = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    doc_add(&mut store, "a3f1-0004", "a2", None, Some(book));

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, &named_in(&["a3f1-0004", "a3f1-0002"])) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    let told: Vec<&str> = state
        .pages_of(book)
        .iter()
        .map(|one| one.file.as_str())
        .collect();
    let four = told.iter().position(|one| *one == "a3f1-0004").unwrap();
    let two = told.iter().position(|one| *one == "a3f1-0002").unwrap();
    assert!(
        four < two,
        "the two it names read as it names them: {told:?}"
    );
    let adrift = told.iter().position(|one| *one == "a3f1-0003").unwrap();
    assert_eq!(
        adrift, 1,
        "and the one it never names holds the place it held: {told:?}"
    );
    assert_eq!(state.docs[&loose].page_of, Some(book), "it is still a page");
}

#[test]
fn naming_a_document_that_is_not_its_page_changes_no_order() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    doc_add(&mut store, "a3f1-0009", "a5", None, None);

    let state = replayed(&world);
    assert!(
        state
            .pages_told(
                book,
                &named_in(&["a3f1-0009", "a3f1-0002", "a3f1-0009", "a3f1-0003"])
            )
            .is_empty()
    );
}

#[test]
fn reordering_pages_from_the_text_keeps_the_hot_cache_instead_of_throwing_it_away() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));

    let mut cache = Cache::open(&world.cache_dir).unwrap().unwrap();
    let state = replayed(&world);
    let print = cache::fingerprint(&world.store_root);
    cache.store(&state, &print).unwrap();

    let moved = store
        .append(Op::DocMove {
            id: one,
            d: Filed {
                folder: None,
                page_of: None,
                order: Some(order::after("a1")),
            },
        })
        .unwrap();
    let now = replayed(&world);
    let print = cache::advance(
        Some(&mut cache),
        &now,
        std::slice::from_ref(&moved),
        &world.store_root,
        false,
    );

    let held = cache
        .load(&print, true)
        .expect("an order is one row, not a cascade");
    assert_eq!(
        held.pages_of(book)
            .iter()
            .map(|one| one.file.as_str())
            .collect::<Vec<_>>(),
        ["a3f1-0003", "a3f1-0002"]
    );
}

fn permutations(items: Vec<String>) -> Vec<Vec<String>> {
    if items.len() <= 1 {
        return vec![items];
    }
    let mut out = Vec::new();
    for i in 0..items.len() {
        let mut rest = items.clone();
        let picked = rest.remove(i);
        for mut tail in permutations(rest) {
            tail.insert(0, picked.clone());
            out.push(tail);
        }
    }
    out
}

fn shuffled(seed: &mut u64, items: &mut [String]) {
    for i in (1..items.len()).rev() {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 7;
        *seed ^= *seed << 17;
        items.swap(i, (*seed as usize) % (i + 1));
    }
}

#[test]
fn two_hundred_pages_reordered_repeatedly_through_pages_told_keep_a_strictly_rising_order_with_short_keys()
 {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);

    let mut key = "a0".to_string();
    let mut files = Vec::with_capacity(200);
    for i in 0..200 {
        key = order::after(&key);
        let file = format!("a3f1-{:04}", i + 2);
        doc_add(&mut store, &file, &key, None, Some(book));
        files.push(file);
    }

    let mut seed = 0x2545_f491_4f6c_dd1du64;
    for round in 0..20 {
        let mut wanted = files.clone();
        shuffled(&mut seed, &mut wanted);
        let refs: Vec<&str> = wanted.iter().map(String::as_str).collect();

        let state = replayed(&world);
        let ops: Vec<Op> = state
            .pages_told(book, &named_in(&refs))
            .into_iter()
            .map(|(id, key)| Op::DocMove {
                id,
                d: Filed {
                    folder: None,
                    page_of: None,
                    order: Some(key),
                },
            })
            .collect();
        store.append_batch(ops).unwrap();

        let state = replayed(&world);
        let pages = state.pages_of(book);
        let told: Vec<&str> = pages.iter().map(|one| one.file.as_str()).collect();
        assert_eq!(told, refs, "round {round}");

        let orders: Vec<&str> = pages.iter().map(|one| one.order.as_str()).collect();
        assert!(
            orders.windows(2).all(|two| two[0] < two[1]),
            "round {round}: {orders:?}"
        );
        let longest = orders.iter().map(|one| one.len()).max().unwrap();
        assert!(longest <= 21, "round {round}: {longest} characters wide");
    }
}

#[test]
fn every_permutation_of_a_small_run_is_a_fixed_point_on_the_second_pass_through_pages_told() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let files: Vec<String> = (0..4)
        .map(|i| {
            let file = format!("a3f1-{:04}", i + 2);
            doc_add(&mut store, &file, &format!("a{i}"), None, Some(book));
            file
        })
        .collect();

    for perm in permutations(files) {
        let refs: Vec<&str> = perm.iter().map(String::as_str).collect();
        let body = named_in(&refs);

        let mut state = replayed(&world);
        for (id, order) in state.pages_told(book, &body) {
            moved(&mut store, id, &order);
        }
        state = replayed(&world);

        let told: Vec<&str> = state
            .pages_of(book)
            .iter()
            .map(|one| one.file.as_str())
            .collect();
        assert_eq!(told, refs, "{perm:?}");

        assert!(
            state.pages_told(book, &body).is_empty(),
            "a second pass with the same body must ask for nothing: {perm:?}"
        );
    }
}

#[test]
fn a_page_named_twice_in_the_body_is_ordered_once_at_its_first_mention() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));

    let body = format!(
        "{}{}{}",
        named_in(&["a3f1-0003"]),
        named_in(&["a3f1-0002"]),
        named_in(&["a3f1-0003"]),
    );

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, &body) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![two, one],
        "the second mention changes nothing, the first is what counts"
    );
}

#[test]
fn naming_a_page_that_belongs_to_another_document_does_not_pull_it_into_this_ones_order() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    let other_book = doc_add(&mut store, "a3f1-0004", "a0", None, None);
    let foreign = doc_add(&mut store, "a3f1-0005", "a0", None, Some(other_book));

    let state = replayed(&world);
    let told = state.pages_told(book, &named_in(&["a3f1-0005", "a3f1-0003", "a3f1-0002"]));

    assert!(
        told.iter().all(|(id, _)| *id != foreign),
        "a page from another document must not be moved by this one's body"
    );

    for (id, order) in told {
        moved(&mut store, id, &order);
    }
    let state = replayed(&world);
    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![two, one],
        "the book's own two pages still get reordered to what its body says"
    );
    assert_eq!(state.docs[&foreign].page_of, Some(other_book));
    assert_eq!(
        state.docs[&foreign].order, "a0",
        "untouched by a body it does not belong to"
    );
}

#[test]
fn a_page_named_only_inside_a_code_fence_does_not_count_and_the_page_is_left_alone() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let fenced = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let named = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));

    let body = format!(
        "{}```\n{}```\n",
        named_in(&["a3f1-0003"]),
        named_in(&["a3f1-0002"]),
    );

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, &body) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![fenced, named],
        "the reference inside the fence is text, not a naming, so that page is not moved at all"
    );
}

#[test]
fn pages_titled_with_brackets_emoji_or_right_to_left_script_are_still_ordered_by_where_they_are_named()
 {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let bracketed = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let emoji = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    let rtl = doc_add(&mut store, "a3f1-0004", "a2", None, Some(book));

    let body = format!(
        "{}\n\n{}\n\n{}\n\n",
        tisty_core::refs::card("a3f1-0004", "الفصل الأول"),
        tisty_core::refs::card("a3f1-0002", "Chapter [draft]"),
        tisty_core::refs::card("a3f1-0003", "📄 Notes"),
    );

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, &body) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![rtl, bracketed, emoji]
    );
}

#[test]
fn a_page_named_with_the_angle_bracket_destination_form_is_still_recognised() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));

    let body = "[Dos](<tisty:doc/a3f1-0003>)\n\n[Uno](<tisty:doc/a3f1-0002>)\n\n";

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, body) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![two, one]
    );
}

#[test]
fn a_page_removed_from_the_text_and_later_put_back_returns_to_where_it_is_named() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    let three = doc_add(&mut store, "a3f1-0004", "a2", None, Some(book));

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, &named_in(&["a3f1-0003", "a3f1-0002"])) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);
    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![two, one, three],
        "the two it names read in that order, and the one it dropped holds its place"
    );

    for (id, order) in state.pages_told(book, &named_in(&["a3f1-0004", "a3f1-0003", "a3f1-0002"])) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);
    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![three, two, one],
        "naming it again puts it back exactly where the text says"
    );
}

#[test]
fn a_page_named_by_a_plain_inline_link_rather_than_a_card_is_still_ordered() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));

    let body =
        "visto en [la segunda](tisty:doc/a3f1-0003) antes que [la primera](tisty:doc/a3f1-0002)\n";

    let mut state = replayed(&world);
    for (id, order) in state.pages_told(book, body) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![two, one]
    );
}

#[test]
fn deleting_a_parent_with_fifty_named_pages_sheds_every_one_of_them() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let (parent, parent_file) = make(&world, &mut store, "# root\n\nhola\n", None, None, "a0");

    let mut files = vec![parent_file];
    let mut names = Vec::new();
    let mut key = "a0".to_string();
    for i in 0..50 {
        key = order::after(&key);
        let (_, file) = make(
            &world,
            &mut store,
            &format!("# page {i}\n\ncontenido\n"),
            None,
            Some(parent),
            &key,
        );
        names.push(file.clone());
        files.push(file);
    }
    names.reverse();

    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let mut state = replayed(&world);
    for (id, order) in state.pages_told(parent, &named_in(&refs)) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);
    assert_eq!(
        state
            .pages_of(parent)
            .iter()
            .map(|k| k.file.clone())
            .collect::<Vec<_>>(),
        names,
        "reordered before it is ever deleted"
    );

    store.append(Op::DocDelete { id: parent }).unwrap();
    let state = replayed(&world);

    assert!(state.docs.is_empty());
    for file in &files {
        assert!(state.shed.contains(file), "{file} was not shed");
    }

    let swept = docs::sweep(&world.docs_root, &state.shed);
    assert_eq!(swept, files.len());
}

#[test]
fn converting_a_page_to_a_document_and_back_lets_the_text_place_it_again() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let a = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let b = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    let c = doc_add(&mut store, "a3f1-0004", "a2", None, Some(book));

    let body = named_in(&["a3f1-0002", "a3f1-0003", "a3f1-0004"]);

    store
        .append(Op::DocMove {
            id: b,
            d: Filed {
                folder: None,
                page_of: Some(None),
                order: None,
            },
        })
        .unwrap();
    let mut state = replayed(&world);
    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![a, c],
        "the converted page is no longer part of its old document"
    );

    store
        .append(Op::DocMove {
            id: b,
            d: Filed {
                folder: None,
                page_of: Some(Some(book)),
                order: None,
            },
        })
        .unwrap();
    state = replayed(&world);
    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![a, c, b],
        "coming back, it lands last among its siblings"
    );

    for (id, order) in state.pages_told(book, &body) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);
    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|k| k.id)
            .collect::<Vec<_>>(),
        vec![a, b, c],
        "the text still said where it belongs, once it is asked"
    );
}

#[test]
fn pages_told_asks_for_nothing_when_a_document_holds_fewer_than_two_pages() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let empty_book = doc_add(&mut store, "a3f1-0003", "a1", None, None);

    let state = replayed(&world);
    assert!(state.pages_told(book, &named_in(&["a3f1-0002"])).is_empty());
    assert!(state.pages_told(empty_book, "anything at all").is_empty());
}

#[test]
fn undoing_a_page_of_conversion_puts_the_folder_back_as_well_as_the_order() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let work = folder_add(&mut store, "trabajo", None);
    let home = folder_add(&mut store, "casa", None);
    let parent = doc_add(&mut store, "a3f1-0001", "a0", Some(work), None);
    let loose = doc_add(&mut store, "a3f1-0003", "z9", Some(home), None);

    let baseline = replayed(&world);
    assert_eq!(baseline.docs[&loose].folder, Some(home));

    let converted = store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(Some(parent)),
                order: None,
            },
        })
        .unwrap();
    assert_eq!(
        replayed(&world).docs[&loose].folder,
        Some(work),
        "hanging it took the folder of the document it hangs from"
    );

    let undo_op = undo::inverse(&converted, &baseline).unwrap();
    store.append_batch(undo_op).unwrap();

    assert_eq!(
        replayed(&world).docs[&loose].folder,
        Some(home),
        "so undoing it has to hand the folder back too"
    );
}

#[test]
fn a_book_written_before_any_of_this_is_not_turned_inside_out_by_its_first_named_page() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));
    let three = doc_add(&mut store, "a3f1-0004", "a2", None, Some(book));

    let mut state = replayed(&world);
    assert!(
        state
            .pages_told(book, "# Libro\n\nsin una sola tarjeta.")
            .is_empty(),
        "a body that names none of them says nothing about any of them"
    );

    let four = doc_add(&mut store, "a3f1-0005", "a3", None, Some(book));
    state = replayed(&world);
    for (id, order) in state.pages_told(book, &named_in(&["a3f1-0005"])) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    assert_eq!(
        state
            .pages_of(book)
            .iter()
            .map(|one| one.id)
            .collect::<Vec<_>>(),
        vec![one, two, three, four],
        "the new page is named and the old ones are not, and the book still reads as it did"
    );
}

#[test]
fn two_machines_that_each_name_a_new_page_first_can_land_on_the_same_key_and_still_read_back_in_a_stable_order()
 {
    let world_a = World::new();
    let mut store_a = world_a.store("dev_a");
    let book = doc_add(&mut store_a, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store_a, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store_a, "a3f1-0003", "a1", None, Some(book));

    // A second machine that pulled the same history before either of them wrote a word since.
    let world_b = World::new();
    let mut store_b = world_b.store("dev_b");
    for event in store_a.read_all().unwrap() {
        store_b.append_event(&event).unwrap();
    }

    // Both machines independently write a brand new page and name it first, from the same
    // starting text -- neither has seen the other's move.
    let new_a = doc_add(&mut store_a, "a3f1-0004", "a2", None, Some(book));
    for (id, order) in
        replayed(&world_a).pages_told(book, &named_in(&["a3f1-0004", "a3f1-0002", "a3f1-0003"]))
    {
        moved(&mut store_a, id, &order);
    }

    let new_b = doc_add(&mut store_b, "a3f1-0005", "a2", None, Some(book));
    for (id, order) in
        replayed(&world_b).pages_told(book, &named_in(&["a3f1-0005", "a3f1-0002", "a3f1-0003"]))
    {
        moved(&mut store_b, id, &order);
    }

    let merged_ab = sorted_merge(store_a.read_all().unwrap(), store_b.read_all().unwrap());
    let merged_ba = sorted_merge(store_b.read_all().unwrap(), store_a.read_all().unwrap());
    let state_ab = State::replay(&merged_ab);
    let state_ba = State::replay(&merged_ba);

    assert_eq!(state_ab, state_ba, "merging in either order must converge");

    // The two inserts computed a key from the very same pair of neighbours, with no way to know
    // about each other, so the key itself can collide -- that is expected, not a corruption.
    assert_eq!(
        state_ab.docs[&new_a].order, state_ab.docs[&new_b].order,
        "both machines inserted before the same neighbour from the same starting point"
    );

    // What must never happen is the merge losing a page or crashing on the tie: `pages_of`
    // still hands back all four in one strict, deterministic order.
    let told: Vec<DocId> = state_ab.pages_of(book).iter().map(|one| one.id).collect();
    assert_eq!(told.len(), 4, "no page vanished across the merge");
    assert!(
        told.contains(&new_a)
            && told.contains(&new_b)
            && told.contains(&one)
            && told.contains(&two),
        "{told:?}"
    );
    assert_eq!(
        told,
        state_ba
            .pages_of(book)
            .iter()
            .map(|one| one.id)
            .collect::<Vec<_>>(),
        "the tie breaks the same way regardless of merge order"
    );
}

#[test]
fn settling_a_book_whose_text_names_only_some_of_its_pages_leaves_every_key_apart() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let book = doc_add(&mut store, "a3f1-0001", "a0", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "V", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "W", None, Some(book));
    let three = doc_add(&mut store, "a3f1-0004", "X", None, Some(book));
    let four = doc_add(&mut store, "a3f1-0005", "Y", None, Some(book));

    let state = replayed(&world);
    let body = named_in(&["a3f1-0004", "a3f1-0003"]);
    let told = state.pages_told(book, &body);
    assert!(!told.is_empty(), "the text asks for a different order");

    store
        .append_batch(
            told.into_iter()
                .map(|(id, order)| Op::DocMove {
                    id,
                    d: Filed {
                        folder: None,
                        page_of: None,
                        order: Some(order),
                    },
                })
                .collect(),
        )
        .unwrap();

    let after = replayed(&world);
    let keys: Vec<&str> = after
        .pages_of(book)
        .iter()
        .map(|one| one.order.as_str())
        .collect();
    let apart: std::collections::BTreeSet<&&str> = keys.iter().collect();
    assert_eq!(
        apart.len(),
        keys.len(),
        "two pages share a key, so a random id decides where they sit: {keys:?}"
    );

    let seen: Vec<DocId> = after.pages_of(book).iter().map(|one| one.id).collect();
    let at = |who: DocId| seen.iter().position(|one| *one == who).unwrap();
    assert!(at(three) < at(two), "the text said this one comes first");
    assert!(at(one) < at(three), "and the rest kept their places");
    assert!(at(two) < at(four));
}

#[test]
fn a_book_whose_pages_arrived_two_different_ways_still_keeps_every_key_apart() {
    let world = World::new();
    let mut store = world.store("dev_a");
    let shelf = folder_add(&mut store, "libro", None);
    let book = doc_add(&mut store, "a3f1-0001", "V", Some(shelf), None);
    let one = doc_add(&mut store, "a3f1-0002", "V", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "W", None, Some(book));
    let loose_a = doc_add(&mut store, "a3f1-0004", "W", Some(shelf), None);
    let loose_b = doc_add(&mut store, "a3f1-0005", "X", Some(shelf), None);

    for who in [loose_a, loose_b] {
        store
            .append(Op::DocMove {
                id: who,
                d: Filed {
                    folder: None,
                    page_of: Some(Some(book)),
                    order: None,
                },
            })
            .unwrap();
    }

    let mut state = replayed(&world);
    assert_eq!(state.pages_of(book).len(), 4);
    for (id, order) in state.pages_told(book, &named_in(&["a3f1-0003", "a3f1-0002"])) {
        moved(&mut store, id, &order);
    }
    state = replayed(&world);

    let pages = state.pages_of(book);
    let keys: Vec<&str> = pages.iter().map(|one| one.order.as_str()).collect();
    let apart: std::collections::BTreeSet<&&str> = keys.iter().collect();
    assert_eq!(apart.len(), keys.len(), "two pages share a key: {keys:?}");

    let seen: Vec<DocId> = pages.iter().map(|one| one.id).collect();
    assert_eq!(
        seen,
        vec![two, one, loose_a, loose_b],
        "the text says the first two, and the ones it never names hold their places"
    );

    assert!(
        state
            .pages_told(book, &named_in(&["a3f1-0003", "a3f1-0002"]))
            .is_empty(),
        "and asking again asks for nothing"
    );
}

#[test]
fn locking_a_document_locks_the_pages_it_holds() {
    let world = World::new();
    let mut store = world.store("a");
    let book = doc_add(&mut store, "a3f1-0001", "V", None, None);
    let one = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));
    let two = doc_add(&mut store, "a3f1-0003", "a1", None, Some(book));

    store.append(Op::DocLock { id: book }).unwrap();
    let state = replayed(&world);

    for id in [book, one, two] {
        assert!(state.shut(id), "{id} should be locked");
    }
    assert!(state.bolted("a3f1-0002"), "a page is asked for by its file");

    store.append(Op::DocUnlock { id: book }).unwrap();
    let state = replayed(&world);
    for id in [book, one, two] {
        assert!(!state.shut(id), "{id} should be open");
    }
}

#[test]
fn a_page_written_into_a_locked_document_is_born_locked() {
    let world = World::new();
    let mut store = world.store("a");
    let book = doc_add(&mut store, "a3f1-0001", "V", None, None);
    store.append(Op::DocLock { id: book }).unwrap();
    let late = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));

    let state = replayed(&world);
    assert!(state.shut(late));
}

#[test]
fn a_page_carries_no_lock_of_its_own_to_lose() {
    let world = World::new();
    let mut store = world.store("a");
    let book = doc_add(&mut store, "a3f1-0001", "V", None, None);
    let page = doc_add(&mut store, "a3f1-0002", "a0", None, Some(book));

    store.append(Op::DocLock { id: page }).unwrap();
    let state = replayed(&world);
    assert!(
        !state.shut(page),
        "a page is locked with the book that holds it"
    );

    store.append(Op::DocLock { id: book }).unwrap();
    store.append(Op::DocUnlock { id: book }).unwrap();
    let state = replayed(&world);
    assert!(
        !state.shut(page),
        "unlocking the book leaves nothing of its own behind"
    );
}

#[test]
fn a_document_landing_under_a_locked_book_is_locked_by_being_there() {
    let world = World::new();
    let mut store = world.store("a");
    let book = doc_add(&mut store, "a3f1-0001", "V", None, None);
    let loose = doc_add(&mut store, "a3f1-0002", "W", None, None);
    store.append(Op::DocLock { id: book }).unwrap();

    store
        .append(Op::DocMove {
            id: loose,
            d: Filed {
                folder: None,
                page_of: Some(Some(book)),
                order: None,
            },
        })
        .unwrap();

    let state = replayed(&world);
    assert!(state.shut(loose), "a page of a locked book is not a way in");
    assert!(state.bolted("a3f1-0002"));
}

#[test]
fn a_page_that_replay_left_holding_a_lock_can_always_be_let_go_of() {
    let world = World::new();
    let mut store = world.store("a");
    let book = doc_add(&mut store, "a3f1-0001", "V", None, None);
    let mine = doc_add(&mut store, "a3f1-0002", "W", None, None);
    store.append(Op::DocLock { id: mine }).unwrap();
    store
        .append(Op::DocMove {
            id: mine,
            d: Filed {
                folder: None,
                page_of: Some(Some(book)),
                order: None,
            },
        })
        .unwrap();

    let state = replayed(&world);
    assert!(
        !state.docs.get(&mine).unwrap().locked,
        "landing as a page leaves no lock of its own behind"
    );

    let mut store = world.store("b");
    store.append(Op::DocLock { id: mine }).unwrap();
    store.append(Op::DocUnlock { id: mine }).unwrap();
    let state = replayed(&world);
    assert!(!state.shut(mine), "there is always a way out");
}

#[test]
fn a_locked_document_is_told_apart_from_an_archived_one() {
    let world = World::new();
    let mut store = world.store("a");
    let doc = doc_add(&mut store, "a3f1-0001", "V", None, None);

    store.append(Op::DocLock { id: doc }).unwrap();
    let state = replayed(&world);
    let kept = state.docs.get(&doc).unwrap();

    assert!(kept.locked);
    assert!(!kept.archived, "locking is not putting away");
}
