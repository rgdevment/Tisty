use ulid::Ulid;

use super::super::tests::{World, doc, edges, held, task};
use crate::event::TaskAdd;
use crate::model::{Resolved, Status};
use crate::{Op, docs};

fn tie(source: &str, target: &str, kind: &str) -> (String, String, String) {
    (source.into(), target.into(), kind.into())
}

fn task_page(world: &World, store: &mut crate::Store, task: Ulid, body: &str) -> String {
    let file = crate::task_docs::file_at(task, 1);
    docs::create_named(&world.paths.docs(), &file, body).unwrap();
    store
        .append(Op::DocAdd {
            id: crate::task_docs::id_at(task, 1),
            d: crate::event::DocAdd {
                wrote: None,
                guest: false,
                sown: false,
                made: None,
                by: None,
                said: Some(crate::event::Said::of(body)),
                file: file.clone(),
                order: "a0".into(),
                folder: None,
                page_of: None,
                task_of: Some(task),
                edited: None,
            },
        })
        .unwrap();
    file
}

fn rewrite(world: &World, file: &str, body: &str) {
    std::fs::write(docs::resolve(&world.paths.docs(), file).unwrap(), body).unwrap();
}

#[test]
fn every_kind_of_connection_comes_from_what_was_written() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "call the bank", &["errands"]);
    let whole = task(&mut store, "move house", &[]);
    let part = Ulid::generate();
    let mut d = TaskAdd::new("pack the books", "a0");
    d.part_of = Some(whole);
    store.append(Op::TaskAdd { id: part, d }).unwrap();
    let (_, sheet) = doc(
        &world,
        &mut store,
        &crate::brainstorm::opening("Ideas"),
        None,
        None,
    );
    let born = Ulid::generate();
    let mut d = TaskAdd::new("try the new bakery", "a0");
    d.source = Some(format!("{sheet}#2026-11-17 10:00 · yellow"));
    store.append(Op::TaskAdd { id: born, d }).unwrap();
    let (book, book_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, page) = doc(&world, &mut store, "# Lisbon\n", Some(book), None);
    rewrite(
        &world,
        &book_file,
        &format!("# Trips\n\n![Lisbon](tisty:doc/{page})\n"),
    );
    let idea = crate::brainstorm::jotted(
        "walk along the river",
        "2026-11-17T10:00".parse().unwrap(),
        crate::brainstorm::COLOURS[0],
    );
    let notes = format!(
        "# Notes\n\n![Trips](tisty:doc/{book_file})\n\nCall [the bank](tisty:task/{errand}) \
         and see ![map](attachments/map-1a2b3c4d.png).\n\n```\n[x](tisty:doc/{book_file})\n```\n\n\
         {idea}\n\n#travel\n"
    );
    let (_, notes_file) = doc(&world, &mut store, &notes, None, None);
    let state = world.state();
    let mut brain = world.brain();

    world.refreshed(&mut brain, &state);

    let all = edges(&brain);
    let mut wanted = vec![
        tie(&notes_file, &book_file, "cite"),
        tie(&notes_file, &errand.to_string(), "mention"),
        tie(&notes_file, "attachments/map-1a2b3c4d.png", "uses"),
        tie(&notes_file, &sheet, "structure"),
        tie(&notes_file, "#travel", "tag"),
        tie(&page, &book_file, "structure"),
        tie(&part.to_string(), &whole.to_string(), "structure"),
        tie(&born.to_string(), &sheet, "structure"),
        tie(&errand.to_string(), "#errands", "tag"),
    ];
    wanted.sort();
    assert_eq!(
        all, wanted,
        "the book's page line and the link in code tie nothing"
    );
}

#[test]
fn a_link_to_what_has_not_arrived_waits_for_it() {
    let world = World::new();
    let mut store = world.store();
    let (_, notes) = doc(
        &world,
        &mut store,
        "# Notes\n\n[later](tisty:doc/zz-0001)\n",
        None,
        None,
    );
    let state = world.state();
    let mut brain = world.brain();

    world.refreshed(&mut brain, &state);

    assert_eq!(edges(&brain), vec![tie(&notes, "zz-0001", "mention")]);
}

#[test]
fn a_link_taken_out_unties_it() {
    let world = World::new();
    let mut store = world.store();
    let (_, trips) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, notes) = doc(
        &world,
        &mut store,
        &format!("# Notes\n\n[t](tisty:doc/{trips})\n"),
        None,
        None,
    );
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    assert_eq!(edges(&brain), vec![tie(&notes, &trips, "mention")]);

    rewrite(&world, &notes, "# Notes\n\nNothing linked any more.\n");
    world.refreshed(&mut brain, &state);

    assert!(edges(&brain).is_empty());
}

#[test]
fn a_link_on_a_task_s_page_is_the_task_s() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "plan the trip", &[]);
    let (_, trips) = doc(&world, &mut store, "# Trips\n", None, None);
    let words = task_page(
        &world,
        &mut store,
        errand,
        "---\ntask: x\n---\n\n## Notes\n",
    );
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    assert!(edges(&brain).is_empty());

    rewrite(
        &world,
        &words,
        &format!("---\ntask: x\n---\n\n## Notes\n\nSee [t](tisty:doc/{trips})\n"),
    );
    world.refreshed(&mut brain, &state);

    assert_eq!(
        edges(&brain),
        vec![tie(&errand.to_string(), &trips, "mention")]
    );
}

#[test]
fn a_link_to_a_task_s_page_reaches_the_task() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "plan the trip", &[]);
    let words = task_page(&world, &mut store, errand, "---\ntask: x\n---\n");
    let (_, notes) = doc(
        &world,
        &mut store,
        &format!("# Notes\n\n[p](tisty:doc/{words})\n"),
        None,
        None,
    );
    let state = world.state();
    let mut brain = world.brain();

    world.refreshed(&mut brain, &state);

    assert_eq!(
        edges(&brain),
        vec![tie(&notes, &errand.to_string(), "mention")]
    );
}

#[test]
fn a_book_moved_to_another_folder_takes_its_pages_along() {
    let world = World::new();
    let mut store = world.store();
    let (book, book_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, page) = doc(&world, &mut store, "# Lisbon\n", Some(book), None);
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let away = Ulid::generate();
    if let Some(one) = state.docs.get_mut(&book) {
        one.folder = Some(away);
    }
    world.refreshed(&mut brain, &state);

    let groups = held(&brain, "COALESCE(grp, '')");
    assert_eq!(groups.get(&book_file), Some(&away.to_string()));
    assert_eq!(
        groups.get(&page),
        Some(&away.to_string()),
        "the page stays with its book"
    );
}

#[test]
fn each_task_and_document_tells_where_it_stands() {
    let world = World::new();
    let mut store = world.store();
    let open = task(&mut store, "open", &[]);
    let said = task(&mut store, "said done", &[]);
    let let_go = task(&mut store, "said dropped", &[]);
    let done = task(&mut store, "done", &[]);
    let (shelved, shelved_file) = doc(&world, &mut store, "# Old\n", None, None);
    let mut state = world.state();
    let resolved = |drop: bool| Resolved {
        at: jiff::Timestamp::from_second(1).unwrap(),
        by: crate::DeviceId("dev_a".into()),
        entry: Ulid::generate(),
        via: None,
        drop,
    };
    if let Some(one) = state.tasks.get_mut(&said) {
        one.resolved = Some(resolved(false));
    }
    if let Some(one) = state.tasks.get_mut(&let_go) {
        one.resolved = Some(resolved(true));
    }
    if let Some(one) = state.tasks.get_mut(&done) {
        one.status = Status::Done;
    }
    if let Some(one) = state.docs.get_mut(&shelved) {
        one.archived = true;
    }
    let mut brain = world.brain();

    world.refreshed(&mut brain, &state);

    let standing = held(&brain, "COALESCE(state, '')");
    let of = |id: &str| standing.get(id).map(String::as_str);
    assert_eq!(of(&open.to_string()), Some("open"));
    assert_eq!(of(&said.to_string()), Some("said_done"));
    assert_eq!(of(&let_go.to_string()), Some("said_dropped"));
    assert_eq!(of(&done.to_string()), Some("done"));
    assert_eq!(of(&shelved_file), Some("archived"));
}

#[test]
fn a_task_s_page_named_before_it_arrives_already_reaches_the_task() {
    let world = World::new();
    let mut store = world.store();
    let later = Ulid::generate();
    let page = crate::task_docs::file_at(later, 1);
    let (_, notes) = doc(
        &world,
        &mut store,
        &format!("# Notes\n\n[soon](tisty:doc/{page})\n"),
        None,
        None,
    );
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());

    store
        .append(Op::TaskAdd {
            id: later,
            d: TaskAdd::new("arrives later", "a0"),
        })
        .unwrap();
    world.refreshed(&mut brain, &world.state());

    assert_eq!(
        edges(&brain),
        vec![tie(&notes, &later.to_string(), "mention")]
    );
    assert!(held(&brain, "kind").contains_key(&later.to_string()));
}

#[test]
fn a_page_taken_out_of_its_book_is_cited_by_the_line_left_behind() {
    let world = World::new();
    let mut store = world.store();
    let (book, book_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (page, page_file) = doc(&world, &mut store, "# Lisbon\n", Some(book), None);
    rewrite(
        &world,
        &book_file,
        &format!("# Trips\n\n![Lisbon](tisty:doc/{page_file})\n"),
    );
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    assert_eq!(
        edges(&brain),
        vec![tie(&page_file, &book_file, "structure")]
    );

    if let Some(one) = state.docs.get_mut(&page) {
        one.page_of = None;
    }
    world.refreshed(&mut brain, &state);

    assert_eq!(edges(&brain), vec![tie(&book_file, &page_file, "cite")]);
}

#[test]
fn a_book_that_arrives_after_its_page_is_hung_from() {
    let world = World::new();
    let mut store = world.store();
    let (book, book_file) = doc(&world, &mut store, "# Trips\n", None, None);
    let (_, page_file) = doc(&world, &mut store, "# Lisbon\n", Some(book), None);
    let whole = world.state();
    let mut early = whole.clone();
    early.docs.remove(&book);
    let mut brain = world.brain();
    world.refreshed(&mut brain, &early);
    assert!(edges(&brain).is_empty());

    world.refreshed(&mut brain, &whole);

    assert_eq!(
        edges(&brain),
        vec![tie(&page_file, &book_file, "structure")]
    );
}
