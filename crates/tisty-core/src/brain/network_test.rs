use jiff::Timestamp;

use super::super::tests::{World, doc, task};
use super::{
    CAPS, Caps, Held, LeftOut, Network, Neuron, TIES, card_of, dates, kept_out, network,
    touched_last, woven,
};

fn neuron(id: &str, kind: &str, state: Option<&str>) -> Held {
    (
        Neuron {
            id: id.into(),
            kind: kind.into(),
            title: id.into(),
            group: None,
            state: state.map(Into::into),
            when: None,
        },
        None,
    )
}

fn tie(source: &str, target: &str, kind: &str) -> (String, String, String) {
    (source.into(), target.into(), kind.into())
}

fn named(network: &Network) -> Vec<&str> {
    network.nodes.iter().map(|one| one.id.as_str()).collect()
}

fn lines(network: &Network) -> Vec<(&str, &str, &str)> {
    let mut all: Vec<(&str, &str, &str)> = network
        .edges
        .iter()
        .map(|(from, to, kind)| {
            (
                network.nodes[*from as usize].id.as_str(),
                network.nodes[*to as usize].id.as_str(),
                TIES[usize::from(*kind)],
            )
        })
        .collect();
    all.sort_unstable();
    all
}

fn whole(world: &World, shown: impl Fn(&str) -> bool, near: &[String]) -> Network {
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());
    brain.network(shown, near).unwrap().expect("a built base")
}

#[test]
fn every_document_task_and_tag_comes_with_its_lines() {
    let world = World::new();
    let mut store = world.store();
    let party = task(&mut store, "the party", &["home"]).to_string();
    let (plans_id, plans) = doc(&world, &mut store, "# Plans\n\nNothing yet.\n", None, None);
    let body = format!(
        "# Notes\n\n![the party](tisty:task/{party})\n\nSee [the plans](tisty:doc/{plans_id}).\n"
    );
    let (_, notes) = doc(&world, &mut store, &body, None, None);

    let network = whole(&world, |_| true, &[]);

    let mut wanted = vec![
        (party.as_str(), "#home", "tag"),
        (notes.as_str(), party.as_str(), "cite"),
        (notes.as_str(), plans.as_str(), "mention"),
    ];
    wanted.sort_unstable();
    assert_eq!(network.nodes.len(), 4, "{:?}", named(&network));
    assert_eq!(lines(&network), wanted);
    assert_eq!(network.left_out, None);
}

#[test]
fn a_hidden_task_is_neither_a_node_nor_the_end_of_a_line() {
    let world = World::new();
    let mut store = world.store();
    let secret = task(&mut store, "a surprise party", &["surprise"]);
    let body = format!("# Notes\n\n![the party](tisty:task/{secret})\n");
    let (_, notes) = doc(&world, &mut store, &body, None, None);
    let hidden = secret.to_string();

    let network = whole(&world, |id| id != hidden, std::slice::from_ref(&hidden));

    assert_eq!(named(&network), [notes.as_str()]);
    assert!(network.edges.is_empty());
    assert_eq!(network.near, None);
}

#[test]
fn the_network_opens_on_the_first_named_node_it_shows() {
    let world = World::new();
    let mut store = world.store();
    let hidden = task(&mut store, "kept out", &[]).to_string();
    let open = task(&mut store, "in sight", &[]).to_string();

    let network = whole(
        &world,
        |id| id != hidden,
        &["gone".into(), hidden.clone(), open.clone()],
    );

    let near = network.near.expect("a node to open on") as usize;
    assert_eq!(network.nodes[near].id, open);
}

#[test]
fn a_base_never_built_has_no_network_yet() {
    let world = World::new();
    let brain = world.brain();

    assert_eq!(brain.network(|_| true, &[]).unwrap(), None);
    assert_eq!(network(&World::new().paths, |_| true, &[]), None);
}

#[test]
fn a_card_is_read_from_the_base_on_disk_and_never_for_what_is_kept_out() {
    let world = World::new();
    let today = jiff::civil::date(2026, 11, 19);
    assert_eq!(card_of(&world.paths, "x", today, |_| true), None);
    let mut store = world.store();
    let party = task(&mut store, "the party", &["home"]).to_string();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());

    let card = card_of(&world.paths, &party, today, |_| true).expect("the task's card");
    assert_eq!(card.title, "the party");
    assert_eq!(card.around.tags, ["home"]);
    assert_eq!(card_of(&world.paths, &party, today, |id| id != party), None);
}

#[test]
fn a_line_aimed_at_a_document_by_its_id_reaches_the_document() {
    let held = vec![
        (
            Neuron {
                id: "plans.md".into(),
                kind: "doc".into(),
                title: "Plans".into(),
                group: None,
                state: Some("open".into()),
                when: None,
            },
            Some("01ABC".into()),
        ),
        neuron("notes.md", "doc", Some("open")),
    ];
    let ties = [
        tie("notes.md", "01ABC", "mention"),
        tie("notes.md", "plans.md", "mention"),
        tie("notes.md", "gone.md", "cite"),
        tie("notes.md", "notes.md", "cite"),
        tie("notes.md", "plans.md", "guessed"),
    ];

    let network = woven(held, &ties, &[], CAPS);

    assert_eq!(lines(&network), [("notes.md", "plans.md", "mention")]);
}

#[test]
fn a_tag_or_file_that_nothing_in_sight_holds_is_left_out() {
    let held = vec![
        neuron("#home", "tag", None),
        neuron("#alone", "tag", None),
        neuron("pics/a.png", "file", None),
        neuron("notes.md", "doc", Some("open")),
    ];
    let ties = [tie("notes.md", "#home", "tag")];

    let network = woven(held, &ties, &[], CAPS);

    assert_eq!(named(&network), ["#home", "notes.md"]);
}

#[test]
fn past_the_cap_it_keeps_where_it_opens_then_documents_then_work_under_way() {
    let held = vec![
        neuron("a-closed", "task", Some("done")),
        neuron("b-near", "task", Some("done")),
        neuron("c-beside", "task", Some("dropped")),
        neuron("d-open", "task", Some("open")),
        neuron("e-doc", "doc", Some("archived")),
    ];
    let ties = [
        tie("b-near", "c-beside", "structure"),
        tie("a-closed", "e-doc", "mention"),
        tie("d-open", "e-doc", "cite"),
    ];
    let caps = Caps { nodes: 4, edges: 1 };

    let network = woven(held, &ties, &["b-near".into()], caps);

    assert_eq!(named(&network), ["b-near", "c-beside", "d-open", "e-doc"]);
    assert_eq!(lines(&network), [("b-near", "c-beside", "structure")]);
    assert_eq!(network.left_out, Some(LeftOut { nodes: 1, edges: 2 }));
    assert_eq!(network.nodes[network.near.unwrap() as usize].id, "b-near");
}

#[test]
fn a_tag_left_without_its_lines_by_the_cap_goes_with_them() {
    let held = vec![
        neuron("#home", "tag", None),
        neuron("notes.md", "doc", Some("open")),
        neuron("plans.md", "doc", Some("open")),
    ];
    let ties = [
        tie("notes.md", "plans.md", "cite"),
        tie("notes.md", "#home", "tag"),
    ];

    let network = woven(
        held,
        &ties,
        &[],
        Caps {
            nodes: 10,
            edges: 1,
        },
    );

    assert_eq!(named(&network), ["notes.md", "plans.md"]);
    assert_eq!(network.left_out, Some(LeftOut { nodes: 1, edges: 1 }));
}

#[test]
fn the_tasks_the_person_hid_or_let_go_are_kept_out() {
    let world = World::new();
    let mut store = world.store();
    let hidden = task(&mut store, "kept out", &[]);
    let dropped = task(&mut store, "let go", &[]);
    let shown = task(&mut store, "in sight", &[]);
    store.append(crate::Op::TaskHide { id: hidden }).unwrap();
    store.append(crate::Op::TaskDrop { id: dropped }).unwrap();

    let out = kept_out(&world.paths.docs(), &world.state());

    assert!(out.contains(&hidden.to_string()));
    assert!(out.contains(&dropped.to_string()));
    assert!(!out.contains(&shown.to_string()));
}

#[test]
fn what_was_touched_last_is_the_newest_writing_in_sight() {
    let world = World::new();
    let mut store = world.store();
    let (notes_id, notes) = doc(&world, &mut store, "# Notes\n", None, None);
    let busy = task(&mut store, "busy", &[]);
    let quiet = task(&mut store, "quiet", &[]);
    let mut state = world.state();
    let at = |s: i64| Timestamp::from_second(s).unwrap();
    state.docs.get_mut(&notes_id).unwrap().wrote = Some(at(4_000_000_000));
    state.tasks.get_mut(&busy).unwrap().completed_at = Some(at(4_100_000_000));
    state.tasks.get_mut(&quiet).unwrap().grown_at = Some(at(3_900_000_000));
    let out = |state: &crate::State| kept_out(&world.paths.docs(), state);
    assert_eq!(touched_last(&state, &out(&state)), Some(busy.to_string()));

    state.tasks.get_mut(&busy).unwrap().hidden = true;
    assert_eq!(touched_last(&state, &out(&state)), Some(notes.clone()));

    state.tasks.get_mut(&busy).unwrap().hidden = false;
    state.tasks.get_mut(&busy).unwrap().status = crate::Status::Dropped;
    assert_eq!(touched_last(&state, &out(&state)), Some(notes));

    state.docs.clear();
    assert_eq!(touched_last(&state, &out(&state)), Some(quiet.to_string()));
}

#[test]
fn past_the_cap_a_tag_takes_no_room_unless_something_kept_holds_it() {
    let held = vec![
        neuron("#lost", "tag", None),
        neuron("a-closed", "task", Some("done")),
        neuron("b-doc", "doc", Some("open")),
        neuron("c-closed", "task", Some("done")),
    ];
    let ties = [
        tie("a-closed", "#lost", "tag"),
        tie("c-closed", "b-doc", "mention"),
    ];
    let caps = Caps {
        nodes: 2,
        edges: 10,
    };

    let network = woven(held, &ties, &[], caps);

    assert_eq!(named(&network), ["a-closed", "b-doc"]);
}

#[test]
fn a_node_named_like_another_s_id_keeps_its_own_lines() {
    let held = vec![
        (
            Neuron {
                id: "plans.md".into(),
                kind: "doc".into(),
                title: "Plans".into(),
                group: None,
                state: Some("open".into()),
                when: None,
            },
            Some("zz.md".into()),
        ),
        neuron("zz.md", "doc", Some("open")),
        neuron("notes.md", "doc", Some("open")),
    ];
    let ties = [tie("notes.md", "zz.md", "cite")];

    let network = woven(held, &ties, &[], CAPS);

    assert_eq!(lines(&network), [("notes.md", "zz.md", "cite")]);
}

#[test]
fn past_the_cap_a_tag_whose_holder_comes_later_still_finds_room() {
    let held = vec![
        neuron("#home", "tag", None),
        neuron("a-closed", "task", Some("done")),
        neuron("b-closed", "task", Some("done")),
    ];
    let ties = [tie("a-closed", "#home", "tag")];
    let caps = Caps {
        nodes: 2,
        edges: 10,
    };

    let network = woven(held, &ties, &[], caps);

    assert_eq!(named(&network), ["#home", "a-closed"]);
}

#[test]
fn every_document_and_task_carries_the_day_it_was_last_touched() {
    let world = World::new();
    let mut store = world.store();
    let (notes_id, notes) = doc(&world, &mut store, "# Notes\n", None, None);
    let errand = task(&mut store, "errand", &[]);
    let mut state = world.state();
    state.docs.get_mut(&notes_id).unwrap().wrote =
        Some(Timestamp::from_second(1_800_000_000).unwrap());
    state.tasks.get_mut(&errand).unwrap().completed_at =
        Some(Timestamp::from_second(4_000_000_000).unwrap());
    let mut network = whole(&world, |_| true, &[]);

    network.dated(&dates(&state));

    let when = |id: &str| {
        network
            .nodes
            .iter()
            .find(|one| one.id == id)
            .and_then(|one| one.when.clone())
    };
    assert_eq!(when(&notes).as_deref(), Some("2027-01-15"));
    assert_eq!(when(&errand.to_string()).as_deref(), Some("2096-10-02"));
}

fn guessed_between(brain: &super::Brain, rows: &[(&str, &str, f64, &str, &str)]) {
    for (a, b, score, method, shared) in rows {
        brain
            .db
            .execute(
                "INSERT INTO alike(a, b, score, shared, method) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![a, b, score, shared, method],
            )
            .unwrap();
    }
}

#[test]
fn a_guess_comes_apart_from_the_lines_and_only_between_two_nodes_in_sight() {
    let world = World::new();
    let mut store = world.store();
    let budget = task(&mut store, "Presupuesto familiar", &[]).to_string();
    let spending = task(&mut store, "Gastos de diciembre", &[]).to_string();
    let secret = task(&mut store, "Regalo sorpresa", &[]).to_string();
    let typed = task(&mut store, "Ntoas del mes", &[]).to_string();
    let (_, notes) = doc(&world, &mut store, "# Notes\n\nNothing yet.\n", None, None);
    let mut brain = world.brain();
    world.refreshed(&mut brain, &world.state());
    guessed_between(
        &brain,
        &[
            (&budget, &spending, 0.4234, "words", "presupuesto familiar"),
            (&notes, &spending, 0.91, "words", "diciembre notas"),
            (&notes, &budget, 0.95, "meaning", ""),
            (&budget, &secret, 0.8, "words", "regalo"),
            (&budget, "gone", 0.8, "words", "nada"),
            (&budget, &notes, 0.5, "a method not known yet", ""),
            (&spending, &secret, 0.7, "dictionary", "regalo≈gift"),
        ],
    );
    guessed_between(&brain, &[(&notes, &typed, 0.3, "typo", "notas≈ntoas")]);

    let network = brain
        .network(|id| id != secret, &[])
        .unwrap()
        .expect("a built base");

    let place = |id: &str| network.nodes.iter().position(|one| one.id == id).unwrap() as u32;
    let mut guesses = network.alike.clone();
    guesses.sort_unstable();
    let mut wanted = vec![
        (
            place(&budget),
            place(&spending),
            423,
            "presupuesto familiar".to_string(),
        ),
        (
            place(&notes),
            place(&spending),
            910,
            "diciembre notas".to_string(),
        ),
    ];
    wanted.push((place(&notes), place(&typed), 300, "notas≈ntoas".to_string()));
    wanted.sort_unstable();
    assert_eq!(
        guesses, wanted,
        "shared words, a typo or a translation make a guess"
    );
    assert!(network.edges.is_empty(), "a guess is never a written line");
    assert!(!network.nodes.iter().any(|one| one.id == secret));
}

#[test]
fn a_network_with_no_guess_says_nothing_of_them() {
    let world = World::new();
    let mut store = world.store();
    task(&mut store, "the party", &[]);

    let network = whole(&world, |_| true, &[]);

    assert!(network.alike.is_empty());
    let said = serde_json::to_value(&network).unwrap();
    assert!(said.get("alike").is_none(), "{said}");
}

fn a_task_with_a_hung_page(
    world: &World,
    store: &mut crate::Store,
    title: &str,
) -> (ulid::Ulid, String) {
    let first = task(store, title, &[]);
    let (book, _) = doc(
        world,
        store,
        &format!("# {title}\n\nIts page.\n"),
        None,
        Some(first),
    );
    let (_, hung) = doc(
        world,
        store,
        "# Secret hung page\n\nWords.\n",
        Some(book),
        None,
    );
    (first, hung)
}

#[test]
fn a_page_hung_under_a_hidden_task_s_page_is_neither_a_node_nor_a_card() {
    let world = World::new();
    let mut store = world.store();
    let (secret, hung) = a_task_with_a_hung_page(&world, &mut store, "secret plan");
    let citing = format!("# Notes\n\nSee [the page](tisty:doc/{hung}).\n");
    let (_, notes) = doc(&world, &mut store, &citing, None, None);
    store.append(crate::Op::TaskHide { id: secret }).unwrap();
    let out = kept_out(&world.paths.docs(), &world.state());
    let today = jiff::civil::date(2026, 11, 20);

    let network = whole(&world, |id| !out.contains(id), &[]);

    assert!(out.contains(&hung), "{out:?}");
    assert!(named(&network).contains(&notes.as_str()));
    assert!(
        lines(&network)
            .iter()
            .all(|(from, to, _)| *from != hung && *to != hung),
        "{:?}",
        lines(&network)
    );
    let card = card_of(&world.paths, &notes, today, |id| !out.contains(id)).unwrap();
    let told = serde_json::to_string(&card).unwrap();
    assert!(
        !told.contains(&hung) && !told.contains("Secret hung page"),
        "{told}"
    );
    assert!(
        !named(&network).contains(&hung.as_str()),
        "{:?}",
        named(&network)
    );
    assert!(
        network
            .nodes
            .iter()
            .all(|one| one.title != "Secret hung page")
    );
    assert_eq!(
        card_of(&world.paths, &hung, today, |id| !out.contains(id)),
        None
    );
}

// Neither the task nor a page that names it by its place, nor a line or a card around them.
fn kept_whole_out(world: &World, gone: &[&str], citing: &str) {
    let out = kept_out(&world.paths.docs(), &world.state());
    let today = jiff::civil::date(2026, 11, 20);
    let network = whole(world, |id| !out.contains(id), &[]);
    for one in gone {
        assert!(out.contains(*one), "{one} not kept out: {out:?}");
        assert!(!named(&network).contains(one), "{:?}", named(&network));
        assert!(
            lines(&network)
                .iter()
                .all(|(from, to, _)| from != one && to != one),
            "{:?}",
            lines(&network)
        );
        assert_eq!(
            card_of(&world.paths, one, today, |id| !out.contains(id)),
            None
        );
    }
    assert!(named(&network).contains(&citing));
    let card = card_of(&world.paths, citing, today, |id| !out.contains(id)).unwrap();
    let told = serde_json::to_string(&card).unwrap();
    for one in gone {
        assert!(!told.contains(one), "{told}");
    }
}

#[test]
fn a_task_the_person_let_go_and_what_hangs_from_it_leave_their_network() {
    let world = World::new();
    let mut store = world.store();
    let (dropped, hung) = a_task_with_a_hung_page(&world, &mut store, "old plan");
    let citing =
        format!("# Notes\n\nSee [it](tisty:task/{dropped}) and [the page](tisty:doc/{hung}).\n");
    let (_, notes) = doc(&world, &mut store, &citing, None, None);
    store.append(crate::Op::TaskDrop { id: dropped }).unwrap();

    kept_whole_out(&world, &[&dropped.to_string(), &hung], &notes);
}

#[test]
fn the_diary_and_its_pages_leave_the_network() {
    let world = World::new();
    let mut store = world.store();
    let (book_id, book) = doc(
        &world,
        &mut store,
        "---\ndiary: on\n---\n\n# Diario 2026\n\nPaseo por el cerro.\n",
        None,
        None,
    );
    let (_, month) = doc(
        &world,
        &mut store,
        "# Noviembre\n\nAlgo mío.\n",
        Some(book_id),
        None,
    );
    let citing = format!("# Notes\n\nSee [the diary](tisty:doc/{book}).\n");
    let (_, notes) = doc(&world, &mut store, &citing, None, None);

    kept_whole_out(&world, &[&book, &month], &notes);
}
