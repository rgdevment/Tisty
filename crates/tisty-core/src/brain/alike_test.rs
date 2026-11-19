use std::collections::BTreeMap;

use ulid::Ulid;

use super::super::tests::{World, doc, held, task};
use super::*;
use crate::Store;

fn caps() -> Caps {
    Caps {
        common: 1,
        common_at_least: 100,
        steps: u64::MAX,
        ..CAPS
    }
}

fn everyone(_: &str) -> bool {
    true
}

fn worded_ids(brain: &Brain) -> BTreeMap<String, (String, Option<String>, String)> {
    let mut asked = brain
        .db
        .prepare("SELECT id, stamp, lang, said FROM words")
        .unwrap();
    asked
        .query_map([], |row| {
            Ok((row.get(0)?, (row.get(1)?, row.get(2)?, row.get(3)?)))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn rows(brain: &Brain) -> Vec<(String, String, String, String)> {
    let mut asked = brain
        .db
        .prepare("SELECT a, b, shared, method FROM alike ORDER BY a, b")
        .unwrap();
    asked
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn pair(one: &str, other: &str) -> (String, String) {
    let (a, b) = if one < other {
        (one, other)
    } else {
        (other, one)
    };
    (a.to_string(), b.to_string())
}

fn ends(brain: &Brain) -> Vec<(String, String)> {
    rows(brain).into_iter().map(|(a, b, _, _)| (a, b)).collect()
}

fn turn(brain: &Brain) -> Option<u64> {
    turn_of(&brain.db, TURN).unwrap()
}

fn fillers(store: &mut Store) -> Vec<Ulid> {
    [
        "water the plants",
        "call the plumber",
        "book the dentist",
        "iron shirts",
    ]
    .iter()
    .map(|title| task(store, title, &[]))
    .collect()
}

const BUDGET: &str = "# Presupuesto familiar\n\nRevisar los gastos de diciembre.\n";
const SPENDING: &str = "# Gastos del mes\n\nEl presupuesto familiar sube en diciembre.\n";

#[test]
fn every_document_and_task_keeps_its_words_with_its_stamp_and_nothing_else_does() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "renew the passport", &["errands"]);
    let (_, file) = doc(
        &world,
        &mut store,
        "# Trips\n\nLisbon in spring\n",
        None,
        None,
    );
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let words = worded_ids(&brain);
    let stamps = held(&brain, "stamp");
    let ids: Vec<&String> = words.keys().collect();
    let mut wanted = vec![&file, &errand.to_string()]
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    wanted.sort();
    assert_eq!(
        ids,
        wanted.iter().collect::<Vec<_>>(),
        "no tag and no file has words"
    );
    for (id, (stamp, _, _)) in &words {
        assert_eq!(
            Some(stamp),
            stamps.get(id),
            "{id} keeps its words with its stamp"
        );
    }
    assert!(words[&errand.to_string()].2.contains("passport:1"));
    assert!(words[&file].2.contains("lisbon:1"));
    assert!(
        words[&file].2.contains("trips:1"),
        "a title is counted once"
    );
    assert_eq!(turn(&brain), Some(1));

    world.refreshed(&mut brain, &state);
    assert_eq!(
        turn(&brain),
        Some(1),
        "a round that writes no words leaves the turn"
    );
    assert!(matches!(
        brain.weigh_alike(&caps(), None).unwrap(),
        Passed::Ran { .. }
    ));
    assert_eq!(brain.weigh_alike(&caps(), None).unwrap(), Passed::Idle);
}

#[test]
fn an_empty_base_has_nothing_to_weigh() {
    let world = World::new();
    let mut brain = world.brain();
    assert_eq!(brain.weigh_alike(&caps(), None).unwrap(), Passed::Idle);
}

#[test]
fn two_notes_sharing_two_weighty_words_are_alike_and_say_why() {
    let world = World::new();
    let mut store = world.store();
    let (budget_id, budget) = doc(&world, &mut store, BUDGET, None, None);
    let (_, spending) = doc(&world, &mut store, SPENDING, None, None);
    fillers(&mut store);
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    assert!(matches!(
        brain.weigh_alike(&caps(), None).unwrap(),
        Passed::Ran {
            rows: 1,
            written: 1,
            ..
        }
    ));
    let (a, b) = pair(&budget, &spending);
    let all = rows(&brain);
    assert_eq!(all.len(), 1);
    assert_eq!((&all[0].0, &all[0].1), (&a, &b));
    let shared: Vec<&str> = all[0].2.split(' ').collect();
    for word in ["presupuesto", "familiar", "diciembre", "gastos"] {
        assert!(shared.contains(&word), "{shared:?} misses {word}");
    }
    assert_eq!(all[0].3, "words");

    let from_one = brain.alike(&budget, everyone).unwrap();
    assert_eq!(from_one.len(), 1);
    assert_eq!(from_one[0].id, spending);
    assert!(from_one[0].score > 0.0);
    assert_eq!(brain.alike(&spending, everyone).unwrap()[0].id, budget);
    assert_eq!(
        brain.alike(&budget_id.to_string(), everyone).unwrap()[0].id,
        spending,
        "a document is found by its id as well as its file"
    );
    assert!(brain.alike("nobody", everyone).unwrap().is_empty());
}

#[test]
fn a_hidden_task_or_a_diary_page_never_gets_alike_rows_nor_shows_through_a_reader() {
    let world = World::new();
    let mut store = world.store();
    let plain = task(&mut store, "Presupuesto familiar de diciembre", &[]);
    let secret = task(
        &mut store,
        "Revisar el presupuesto familiar de diciembre",
        &[],
    );
    let (book, _) = doc(
        &world,
        &mut store,
        "---\ndiary: on\n---\n\n# Diario 2026\n",
        None,
        None,
    );
    let (_, page) = doc(
        &world,
        &mut store,
        "# Noviembre\n\nEl presupuesto familiar de diciembre.\n",
        Some(book),
        None,
    );
    fillers(&mut store);
    let mut state = world.state();
    state.docs.get_mut(&book).unwrap().diary = true;
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.weigh_alike(&caps(), None).unwrap();

    let (plain_id, secret_id) = (plain.to_string(), secret.to_string());
    assert!(
        !worded_ids(&brain).contains_key(&page),
        "the diary keeps its words"
    );
    assert_eq!(ends(&brain), [pair(&plain_id, &secret_id)]);
    assert!(
        brain
            .alike(&plain_id, |id| id != secret_id)
            .unwrap()
            .is_empty()
    );
    assert!(
        brain
            .alike(&secret_id, |id| id != secret_id)
            .unwrap()
            .is_empty()
    );
    assert_eq!(brain.alike(&plain_id, everyone).unwrap().len(), 1);

    state.tasks.get_mut(&secret).unwrap().hidden = true;
    world.refreshed(&mut brain, &state);

    assert!(
        rows(&brain).is_empty(),
        "hiding takes its rows away in the same round"
    );
    assert!(!worded_ids(&brain).contains_key(&secret_id));
    assert!(brain.alike(&plain_id, everyone).unwrap().is_empty());
    brain.weigh_alike(&caps(), None).unwrap();
    assert!(rows(&brain).is_empty(), "nor does a pass bring them back");
}

#[test]
fn a_book_that_becomes_a_diary_takes_its_pages_out_though_they_did_not_change() {
    let world = World::new();
    let mut store = world.store();
    let (book, book_file) = doc(&world, &mut store, "# Notas 2026\n", None, None);
    let (_, page) = doc(
        &world,
        &mut store,
        "# Noviembre\n\nEl presupuesto familiar de diciembre.\n",
        Some(book),
        None,
    );
    let (_, budget) = doc(&world, &mut store, BUDGET, None, None);
    fillers(&mut store);
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.weigh_alike(&caps(), None).unwrap();
    assert_eq!(ends(&brain), [pair(&page, &budget)]);

    std::fs::write(
        world.paths.docs().join(format!("{book_file}.md")),
        "---\ndiary: on\n---\n\n# Diario 2026\n",
    )
    .unwrap();
    state.docs.get_mut(&book).unwrap().diary = true;
    world.refreshed(&mut brain, &state);

    assert!(rows(&brain).is_empty());
    assert!(!worded_ids(&brain).contains_key(&page));
}

#[test]
fn what_is_tied_already_is_not_guessed() {
    let world = World::new();
    let mut store = world.store();
    let (_, cited) = doc(&world, &mut store, BUDGET, None, None);
    let (_, citing) = doc(
        &world,
        &mut store,
        &format!("# Gastos del mes\n\nEl presupuesto familiar sube.\n\n![x](tisty:doc/{cited})\n"),
        None,
        None,
    );
    let (_, titled) = doc(
        &world,
        &mut store,
        "# Huerto urbano\n\nTomates y albahaca.\n",
        None,
        None,
    );
    let (_, naming) = doc(
        &world,
        &mut store,
        "# Riego\n\nRegar tomates y albahaca del [[Huerto urbano]].\n",
        None,
        None,
    );
    let whole = task(&mut store, "Mudanza a Valparaíso con cajas", &[]);
    let part = task(&mut store, "Comprar cajas para la mudanza", &[]);
    let (_, one) = doc(
        &world,
        &mut store,
        "# Router\n\nReiniciar el router wifi.\n",
        None,
        None,
    );
    let (_, other) = doc(
        &world,
        &mut store,
        "# Wifi lento\n\nEl router wifi falla.\n",
        None,
        None,
    );
    fillers(&mut store);
    let mut state = world.state();
    state.tasks.get_mut(&part).unwrap().part_of = Some(whole);
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.weigh_alike(&caps(), None).unwrap();

    let found = ends(&brain);
    assert!(found.contains(&pair(&one, &other)), "{found:?}");
    for (a, b) in [
        (cited.clone(), citing.clone()),
        (titled.clone(), naming.clone()),
        (whole.to_string(), part.to_string()),
    ] {
        assert!(
            !found.contains(&pair(&a, &b)),
            "{a} and {b} are tied already"
        );
    }
}

#[test]
fn pages_of_one_book_and_languages_of_one_document_are_kin() {
    let world = World::new();
    let mut store = world.store();
    let (book, book_file) = doc(
        &world,
        &mut store,
        "# Huerto urbano\n\nTomates y albahaca.\n",
        None,
        None,
    );
    let (_, first) = doc(
        &world,
        &mut store,
        "# Riego\n\nTomates y albahaca al sol.\n",
        Some(book),
        None,
    );
    let (_, second) = doc(
        &world,
        &mut store,
        "# Abono\n\nTomates y albahaca con compost.\n",
        Some(book),
        None,
    );
    let (spanish, spanish_file) = doc(
        &world,
        &mut store,
        "# Router\n\nRouter wifi Tisty.\n",
        None,
        None,
    );
    let (english, english_file) = doc(
        &world,
        &mut store,
        "# Router\n\nRouter wifi Tisty.\n",
        None,
        None,
    );
    fillers(&mut store);
    let mut state = world.state();
    state.docs.get_mut(&spanish).unwrap().lang = Some("es-CL".into());
    let english_kept = state.docs.get_mut(&english).unwrap();
    english_kept.lang = Some("en".into());
    english_kept.translation_of = Some(spanish);
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.weigh_alike(&caps(), None).unwrap();

    assert_eq!(worded_ids(&brain)[&spanish_file].1.as_deref(), Some("es"));
    let found = ends(&brain);
    for (a, b) in [
        (first.clone(), second.clone()),
        (book_file.clone(), first.clone()),
        (spanish_file.clone(), english_file.clone()),
    ] {
        assert!(
            !found.contains(&pair(&a, &b)),
            "{a} and {b} are kin: {found:?}"
        );
    }
}

#[test]
fn a_task_and_its_page_are_one_node_with_one_count_of_words() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "renew the passport", &[]);
    let (_, page) = doc(
        &world,
        &mut store,
        "# Notes\n\nThe consulate wants photos.\n",
        None,
        Some(errand),
    );
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let words = worded_ids(&brain);
    assert!(
        !words.contains_key(&page),
        "a task's page is no node of its own"
    );
    let said = &words[&errand.to_string()].2;
    assert!(
        said.contains("consulate:1") && said.contains("passport:1"),
        "{said}"
    );
}

#[test]
fn a_round_that_writes_words_meanwhile_leaves_the_pass_to_the_next_one() {
    let world = World::new();
    let mut store = world.store();
    let (_, budget) = doc(&world, &mut store, BUDGET, None, None);
    let (_, spending) = doc(&world, &mut store, SPENDING, None, None);
    let errands = fillers(&mut store);
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let weighed = brain
        .weighed(&caps(), None)
        .unwrap()
        .expect("words to weigh");
    state.tasks.get_mut(&errands[0]).unwrap().title = "water the ferns".into();
    world.refreshed(&mut brain, &state);
    assert_eq!(brain.kept_alike(weighed).unwrap(), Passed::Overtaken);
    assert!(rows(&brain).is_empty());

    assert!(matches!(
        brain.weigh_alike(&caps(), None).unwrap(),
        Passed::Ran { .. }
    ));
    assert_eq!(ends(&brain), [pair(&budget, &spending)]);
}

#[test]
fn a_pass_writes_only_what_changed() {
    let world = World::new();
    let mut store = world.store();
    let (_, budget) = doc(&world, &mut store, BUDGET, None, None);
    doc(&world, &mut store, SPENDING, None, None);
    let errands = fillers(&mut store);
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.weigh_alike(&caps(), None).unwrap();

    state.tasks.get_mut(&errands[0]).unwrap().title = "water the ferns".into();
    world.refreshed(&mut brain, &state);
    assert!(matches!(
        brain.weigh_alike(&caps(), None).unwrap(),
        Passed::Ran {
            rows: 1,
            written: 0,
            ..
        }
    ));

    let kept = state.papers().find(|one| one.file == budget).unwrap().id;
    state.docs.remove(&kept);
    world.refreshed(&mut brain, &state);
    assert!(rows(&brain).is_empty(), "what is gone takes its rows along");
    assert!(!worded_ids(&brain).contains_key(&budget));
    assert!(matches!(
        brain.weigh_alike(&caps(), None).unwrap(),
        Passed::Ran {
            rows: 0,
            written: 0,
            ..
        }
    ));
}

#[test]
fn a_node_missing_its_words_gets_them_at_the_next_round() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "renew the passport", &[]);
    let errands = fillers(&mut store);
    let mut state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    let id = errand.to_string();
    brain
        .db
        .execute("DELETE FROM words WHERE id = ?", [&id])
        .unwrap();

    state.tasks.get_mut(&errands[0]).unwrap().title = "water the ferns".into();
    world.refreshed(&mut brain, &state);
    assert!(worded_ids(&brain).contains_key(&id));
}

#[test]
fn the_pass_waits_for_the_words_to_hold_still_and_runs_only_when_they_moved() {
    let world = World::new();
    let mut weighing = Weighing::default();
    assert_eq!(
        weighing.kept(&world.paths, None),
        None,
        "a swept store is left alone"
    );
    let mut store = world.store();
    let (_, budget) = doc(&world, &mut store, BUDGET, None, None);
    doc(&world, &mut store, SPENDING, None, None);
    fillers(&mut store);
    drop(store);
    super::super::kept_up(&world.paths, super::super::Round::Starting);
    assert_eq!(weighing.kept(&world.paths, None), Some(Passed::Settling));
    assert!(matches!(
        weighing.kept(&world.paths, None),
        Some(Passed::Ran { .. })
    ));
    assert_eq!(weighing.kept(&world.paths, None), Some(Passed::Idle));

    let file = world.paths.docs().join(format!("{budget}.md"));
    let mut body = std::fs::read_to_string(&file).unwrap();
    body.push_str("\nY la luz.\n");
    std::fs::write(&file, body).unwrap();
    super::super::kept_up(&world.paths, super::super::Round::Nudged);
    assert_eq!(weighing.kept(&world.paths, None), Some(Passed::Settling));
    assert!(matches!(
        weighing.kept(&world.paths, None),
        Some(Passed::Ran { .. })
    ));
}

#[test]
fn a_change_to_a_hidden_task_moves_no_turn() {
    let world = World::new();
    let mut store = world.store();
    let secret = task(&mut store, "a surprise party", &[]);
    fillers(&mut store);
    let mut state = world.state();
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    let before = turn(&brain);

    state.tasks.get_mut(&secret).unwrap().title = "a bigger surprise party".into();
    world.refreshed(&mut brain, &state);
    assert_eq!(turn(&brain), before);
}

#[test]
fn only_documents_pages_and_tasks_in_sight_are_owed_words() {
    let world = World::new();
    let mut store = world.store();
    let errand = task(&mut store, "renew the passport", &["errands"]);
    let secret = task(&mut store, "a surprise party", &[]);
    let (_, file) = doc(
        &world,
        &mut store,
        "# Trips\n\nLisbon in spring\n",
        None,
        None,
    );
    let mut state = world.state();
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.db.execute("DELETE FROM words", []).unwrap();

    let docs = Docs::on_disk(&world.paths.docs());
    let ground = Ground::new(&state, &docs);
    let wanted = super::super::nodes::nodes(&state, &docs, &ground);
    let out = kept_out(&state, &docs);
    let tx = brain.db.unchecked_transaction().unwrap();
    let mut owed: Vec<&String> = unworded(&tx, &wanted, &out, &[]).unwrap();
    owed.sort();
    let errand_id = errand.to_string();
    let mut expected = vec![&file, &errand_id];
    expected.sort();
    assert_eq!(owed, expected, "no tag, no hidden task");

    let fresh = [Rewritten {
        id: &file,
        stamp: "s",
        counted: Counted {
            lang: None,
            guess: None,
            kin: Vec::new(),
            said: String::new(),
        },
    }];
    assert_eq!(unworded(&tx, &wanted, &out, &fresh).unwrap(), [&errand_id]);
}

#[test]
fn a_link_by_a_document_s_id_ties_it_and_a_title_two_documents_carry_ties_neither() {
    let world = World::new();
    let mut store = world.store();
    let (_, one) = doc(
        &world,
        &mut store,
        "# Huerto urbano\n\nTomates y albahaca al sol.\n",
        None,
        None,
    );
    let (_, other) = doc(
        &world,
        &mut store,
        "# Huerto urbano\n\nTomates y albahaca con compost.\n",
        None,
        None,
    );
    let (_, naming) = doc(
        &world,
        &mut store,
        "# Riego\n\nRegar tomates y albahaca del [[Huerto urbano]].\n",
        None,
        None,
    );
    let (router_id, router) = doc(
        &world,
        &mut store,
        "# Router\n\nReiniciar el router wifi.\n",
        None,
        None,
    );
    let (_, slow) = doc(
        &world,
        &mut store,
        &format!("# Wifi lento\n\nEl router wifi falla, ver [esto](tisty:doc/{router_id}).\n"),
        None,
        None,
    );
    fillers(&mut store);
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.weigh_alike(&caps(), None).unwrap();

    let found = ends(&brain);
    assert!(found.contains(&pair(&naming, &one)), "{found:?}");
    assert!(found.contains(&pair(&naming, &other)), "{found:?}");
    assert!(!found.contains(&pair(&router, &slow)), "{found:?}");
}

#[test]
fn a_pass_that_keeps_being_put_off_runs_once_its_patience_is_spent() {
    let world = World::new();
    let mut store = world.store();
    let (_, budget) = doc(&world, &mut store, BUDGET, None, None);
    doc(&world, &mut store, SPENDING, None, None);
    fillers(&mut store);
    drop(store);
    let mut weighing = Weighing::default();
    let file = world.paths.docs().join(format!("{budget}.md"));
    let written = || {
        let mut body = std::fs::read_to_string(&file).unwrap();
        body.push_str("\nOtra línea.\n");
        std::fs::write(&file, body).unwrap();
        super::super::kept_up(&world.paths, super::super::Round::Nudged);
    };
    let mut put_off = 0;
    loop {
        written();
        match weighing.kept(&world.paths, None) {
            Some(Passed::Settling) => put_off += 1,
            Some(Passed::Ran { .. }) => break,
            other => panic!("{other:?}"),
        }
        assert!(put_off <= PATIENCE, "it never ran");
    }
    assert_eq!(put_off, PATIENCE);

    written();
    assert_eq!(
        weighing.kept(&world.paths, None),
        Some(Passed::Settling),
        "its patience starts over"
    );
}

#[test]
fn a_document_whose_file_cannot_be_read_still_counts_its_title() {
    let world = World::new();
    let mut store = world.store();
    let (_, file) = doc(
        &world,
        &mut store,
        "# Lisbon trip\n\nIn spring\n",
        None,
        None,
    );
    std::fs::remove_file(world.paths.docs().join(format!("{file}.md"))).unwrap();
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let said = &worded_ids(&brain)[&file].2;
    assert!(
        said.contains("lisbon:1") && !said.contains("spring"),
        "{said}"
    );
}

#[test]
fn a_title_a_hidden_diary_page_also_carries_still_ties_the_one_every_reader_sees() {
    let world = World::new();
    let mut store = world.store();
    let (book, _) = doc(
        &world,
        &mut store,
        "---\ndiary: on\n---\n\n# Diario 2026\n",
        None,
        None,
    );
    doc(
        &world,
        &mut store,
        "# Huerto urbano\n\nHoy regué.\n",
        Some(book),
        None,
    );
    let (_, garden) = doc(
        &world,
        &mut store,
        "# Huerto urbano\n\nTomates y albahaca al sol.\n",
        None,
        None,
    );
    let (_, naming) = doc(
        &world,
        &mut store,
        "# Riego\n\nRegar tomates y albahaca del [[Huerto urbano]].\n",
        None,
        None,
    );
    fillers(&mut store);
    let mut state = world.state();
    state.docs.get_mut(&book).unwrap().diary = true;
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    brain.weigh_alike(&caps(), None).unwrap();
    assert!(
        !ends(&brain).contains(&pair(&naming, &garden)),
        "{:?}",
        ends(&brain)
    );
}

#[test]
fn a_file_that_leaves_moves_no_turn() {
    let world = World::new();
    let mut store = world.store();
    fillers(&mut store);
    let mut state = world.state();
    state
        .kept
        .insert("files/plan.png".into(), ("ab12".into(), 3));
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    let before = turn(&brain);

    state.kept.remove("files/plan.png");
    world.refreshed(&mut brain, &state);
    assert_eq!(turn(&brain), before);
}

fn guesses(brain: &Brain) -> BTreeMap<String, Option<String>> {
    let mut asked = brain.db.prepare("SELECT id, guess FROM words").unwrap();
    asked
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn dictionary_at(world: &World) -> std::path::PathBuf {
    let at = world.paths.cache().join("dictionary.tsv");
    std::fs::write(&at, "agua\twater\ncuenta\tbill\npagar\tpay\n").unwrap();
    at
}

#[test]
fn the_round_keeps_the_language_each_one_seems_written_in() {
    let world = World::new();
    let mut store = world.store();
    let spanish = task(&mut store, "Pagar la cuenta del agua", &[]).to_string();
    let english = task(&mut store, "Pay the water bill", &[]).to_string();
    let unknown = task(&mut store, "Router", &[]).to_string();
    let (declared, declared_file) =
        doc(&world, &mut store, "# Router\n\nRouter wifi.\n", None, None);
    let (other, other_file) = doc(
        &world,
        &mut store,
        "# Notes\n\nLa casa y el jardín.\n",
        None,
        None,
    );
    let mut state = world.state();
    state.docs.get_mut(&declared).unwrap().lang = Some("en-GB".into());
    state.docs.get_mut(&other).unwrap().lang = Some("pt".into());
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);

    let guessed = guesses(&brain);
    assert_eq!(guessed[&spanish].as_deref(), Some("es"));
    assert_eq!(guessed[&english].as_deref(), Some("en"));
    assert_eq!(guessed[&unknown], None);
    assert_eq!(
        guessed[&declared_file].as_deref(),
        Some("en"),
        "a declared language wins"
    );
    assert_eq!(
        guessed[&other_file].as_deref(),
        Some("es"),
        "a language the dictionary does not speak is guessed from the text"
    );
}

#[test]
fn a_hidden_task_keeps_no_language_either() {
    let world = World::new();
    let mut store = world.store();
    let secret = task(&mut store, "Pagar la cuenta del agua", &[]);
    let mut state = world.state();
    state.tasks.get_mut(&secret).unwrap().hidden = true;
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    assert!(!guesses(&brain).contains_key(&secret.to_string()));
}

#[test]
fn a_spanish_task_and_its_english_twin_are_kept_alike_through_the_dictionary_file() {
    let world = World::new();
    let mut store = world.store();
    let spanish = task(&mut store, "Pagar la cuenta del agua", &[]).to_string();
    let english = task(&mut store, "Pay the water bill", &[]).to_string();
    fillers(&mut store);
    drop(store);
    super::super::kept_up(&world.paths, super::super::Round::Starting);
    let at = dictionary_at(&world);
    let mut weighing = Weighing::default();
    assert_eq!(
        weighing.kept(&world.paths, Some(&at)),
        Some(Passed::Settling)
    );
    assert!(matches!(
        weighing.kept(&world.paths, Some(&at)),
        Some(Passed::Ran { translated: 1, .. })
    ));

    let brain = world.brain();
    let all = rows(&brain);
    let (a, b) = pair(&spanish, &english);
    assert_eq!(all.len(), 1, "{all:?}");
    assert_eq!((&all[0].0, &all[0].1), (&a, &b));
    assert_eq!(all[0].3, "dictionary");
    for told in ["pagar≈pay", "cuenta≈bill", "agua≈water"] {
        let flipped = told.split('≈').rev().collect::<Vec<_>>().join("≈");
        assert!(
            all[0].2.contains(told) || all[0].2.contains(&flipped),
            "{} misses {told}",
            all[0].2
        );
    }
    assert_eq!(
        brain.alike(&spanish, everyone).unwrap()[0].method,
        "dictionary"
    );
}

#[test]
fn a_dictionary_that_cannot_be_read_waits_then_leaves_shared_words_and_warns_once() {
    let world = World::new();
    let mut store = world.store();
    task(&mut store, "Pagar la cuenta del agua", &[]);
    task(&mut store, "Pay the water bill", &[]);
    fillers(&mut store);
    drop(store);
    super::super::kept_up(&world.paths, super::super::Round::Starting);
    let missing = world.paths.cache().join("missing.tsv");
    let mut weighing = Weighing::default();
    for _ in 0..=PATIENCE {
        assert_eq!(
            weighing.kept(&world.paths, Some(&missing)),
            Some(Passed::Settling),
            "one bad read drops no guess"
        );
    }
    assert!(matches!(
        weighing.kept(&world.paths, Some(&missing)),
        Some(Passed::Ran {
            rows: 0,
            translated: 0,
            ..
        })
    ));
    assert_eq!(weighing.missed, PATIENCE + 1, "warned once");
    weighing.kept(&world.paths, Some(&missing));
    assert_eq!(weighing.missed, PATIENCE + 1);
}

#[test]
fn a_pair_that_changes_track_is_written_again() {
    let world = World::new();
    let mut store = world.store();
    let spanish = task(&mut store, "Pagar la cuenta del agua", &[]).to_string();
    let english = task(&mut store, "Pay the water bill", &[]).to_string();
    fillers(&mut store);
    let state = world.state();
    let mut brain = world.brain();
    world.refreshed(&mut brain, &state);
    let dictionary =
        Dictionary::parsed("agua\twater\ncuenta\tbill\npagar\tpay\n".to_string()).unwrap();
    brain.weigh_alike(&caps(), Some(&dictionary)).unwrap();
    let (a, b) = pair(&spanish, &english);
    brain
        .db
        .execute(
            "UPDATE alike SET method = 'words' WHERE a = ?1 AND b = ?2",
            [&a, &b],
        )
        .unwrap();
    brain
        .db
        .execute("DELETE FROM meta WHERE key = ?1", [PASSED])
        .unwrap();
    assert!(matches!(
        brain.weigh_alike(&caps(), Some(&dictionary)).unwrap(),
        Passed::Ran { written: 1, .. }
    ));
    assert_eq!(rows(&brain)[0].3, "dictionary");
}
