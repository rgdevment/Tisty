use super::super::super::tests::{World, doc, task};
use super::super::tests::Words;
use super::*;

struct Deaf;

impl Meaning for Deaf {
    fn version(&self) -> &str {
        "deaf"
    }

    fn meaning(&mut self, _: &str) -> Option<Vec<f32>> {
        panic!("a refused model is never asked")
    }

    fn refused(&self) -> bool {
        true
    }
}

fn vectors_of(brain: &Brain) -> Vec<(String, String, String)> {
    let mut asked = brain
        .db
        .prepare("SELECT id, stamp, model FROM vector ORDER BY id")
        .unwrap();
    asked
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn worded(brain: &Brain) -> Vec<(String, String)> {
    let mut asked = brain
        .db
        .prepare("SELECT id, stamp FROM words ORDER BY id")
        .unwrap();
    asked
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect()
}

fn meant_turn(brain: &Brain) -> Option<u64> {
    turn_of(&brain.db, MEANT).unwrap()
}

fn base(world: &World) -> Brain {
    let mut store = world.store();
    task(&mut store, "call the plumber", &[]);
    task(&mut store, "book the dentist", &[]);
    doc(
        world,
        &mut store,
        "# Presupuesto\n\nEl presupuesto de diciembre.\n",
        None,
        None,
    );
    drop(store);
    let mut brain = world.brain();
    let state = world.state();
    world.refreshed(&mut brain, &state);
    brain
}

#[test]
fn every_node_with_words_gets_its_meaning_with_its_stamp_and_the_model_that_read_it() {
    let world = World::new();
    let mut brain = base(&world);
    let mut words = Words::new();

    let embedded = brain
        .embed(&world.paths, &mut words, BYTES_PER_PASS)
        .unwrap();
    assert_eq!(
        embedded,
        Some(Embedded {
            written: 3,
            left: 0
        })
    );
    let stamps = worded(&brain);
    let vectors = vectors_of(&brain);
    assert_eq!(vectors.len(), 3);
    for ((id, stamp, model), (worded_id, worded_stamp)) in vectors.iter().zip(&stamps) {
        assert_eq!((id, stamp), (worded_id, worded_stamp));
        assert_eq!(model, "words-v1");
    }
    assert_eq!(meant_turn(&brain), Some(1));
    assert_eq!(
        brain
            .embed(&world.paths, &mut words, BYTES_PER_PASS)
            .unwrap(),
        None,
        "nothing is read twice"
    );
    assert_eq!(words.read, 3);
}

#[test]
fn a_pass_reads_up_to_its_budget_and_says_what_is_left() {
    let world = World::new();
    let mut brain = base(&world);
    let mut words = Words::new();

    let first = brain.embed(&world.paths, &mut words, 1).unwrap().unwrap();
    assert_eq!(
        first,
        Embedded {
            written: 1,
            left: 2
        }
    );
    let second = brain.embed(&world.paths, &mut words, 1).unwrap().unwrap();
    assert_eq!(
        second,
        Embedded {
            written: 1,
            left: 1
        }
    );
    assert_eq!(
        words.rested, 0,
        "the model stays while there is more to read"
    );
    let third = brain.embed(&world.paths, &mut words, 1).unwrap().unwrap();
    assert_eq!(
        third,
        Embedded {
            written: 1,
            left: 0
        }
    );
    assert_eq!(words.rested, 1);
    assert_eq!(vectors_of(&brain).len(), 3);
}

#[test]
fn another_model_reads_everything_again() {
    let world = World::new();
    let mut brain = base(&world);
    brain
        .embed(&world.paths, &mut Words::new(), BYTES_PER_PASS)
        .unwrap();
    let mut newer = Words {
        version: "words-v2",
        ..Words::new()
    };
    assert_eq!(brain.unmeant("words-v2").unwrap().len(), 3);
    brain
        .embed(&world.paths, &mut newer, BYTES_PER_PASS)
        .unwrap();
    assert_eq!(newer.read, 3);
    assert!(
        vectors_of(&brain)
            .iter()
            .all(|(_, _, model)| model == "words-v2")
    );
    assert_eq!(meant_turn(&brain), Some(2));
}

#[test]
fn a_refused_model_reads_nothing_and_leaves_the_store_unread() {
    let world = World::new();
    let mut brain = base(&world);
    assert_eq!(
        brain
            .embed(&world.paths, &mut Deaf, BYTES_PER_PASS)
            .unwrap(),
        None
    );
    assert!(vectors_of(&brain).is_empty());
}

#[test]
fn a_model_that_gives_up_keeps_what_it_read_and_stops() {
    let world = World::new();
    let mut brain = base(&world);
    let mut tired = Words {
        gives_up_after: Some(1),
        ..Words::new()
    };
    let embedded = brain
        .embed(&world.paths, &mut tired, BYTES_PER_PASS)
        .unwrap();
    assert_eq!(
        embedded,
        Some(Embedded {
            written: 1,
            left: 0
        })
    );
}

#[test]
fn a_meaning_read_while_the_words_changed_is_not_kept() {
    let world = World::new();
    let mut brain = base(&world);
    let pending = brain.unmeant("words-v1").unwrap();
    let (fresh, _) = meant(&world.paths, &mut Words::new(), &pending, BYTES_PER_PASS);
    assert_eq!(fresh.len(), 3);
    brain
        .db
        .execute(
            "UPDATE words SET stamp = 'written meanwhile' WHERE id = ?",
            [&pending[0].0],
        )
        .unwrap();
    brain
        .db
        .execute("DELETE FROM words WHERE id = ?", [&pending[1].0])
        .unwrap();

    assert_eq!(brain.kept_meant(&fresh, "words-v1").unwrap(), 1);
    let kept: Vec<String> = vectors_of(&brain)
        .into_iter()
        .map(|(id, _, _)| id)
        .collect();
    assert_eq!(kept, [pending[2].0.clone()]);
}

#[test]
fn a_meaning_whose_node_lost_its_words_goes_too() {
    let world = World::new();
    let mut brain = base(&world);
    brain
        .embed(&world.paths, &mut Words::new(), BYTES_PER_PASS)
        .unwrap();
    let id = worded(&brain)[0].0.clone();
    brain
        .db
        .execute("DELETE FROM words WHERE id = ?", [&id])
        .unwrap();

    assert_eq!(
        brain
            .embed(&world.paths, &mut Words::new(), BYTES_PER_PASS)
            .unwrap(),
        Some(Embedded {
            written: 0,
            left: 0
        })
    );
    assert!(vectors_of(&brain).iter().all(|(one, _, _)| *one != id));
    assert_eq!(meant_turn(&brain), Some(2));
}

#[test]
fn reading_the_same_meaning_again_moves_no_turn() {
    let world = World::new();
    let mut brain = base(&world);
    let mut words = Words::new();
    brain
        .embed(&world.paths, &mut words, BYTES_PER_PASS)
        .unwrap();
    brain
        .db
        .execute("UPDATE vector SET stamp = 'older'", [])
        .unwrap();
    assert_eq!(brain.unmeant("words-v1").unwrap().len(), 3);

    brain
        .embed(&world.paths, &mut words, BYTES_PER_PASS)
        .unwrap();
    assert_eq!(words.read, 6);
    assert_eq!(meant_turn(&brain), Some(1));
    assert!(brain.unmeant("words-v1").unwrap().is_empty());
}

#[test]
fn a_store_that_cannot_be_read_gives_no_meanings() {
    let world = World::new();
    let mut brain = base(&world);
    std::fs::remove_dir_all(world.paths.store()).unwrap();
    std::fs::write(world.paths.store(), "not a folder").unwrap();
    assert_eq!(
        brain
            .embed(&world.paths, &mut Words::new(), BYTES_PER_PASS)
            .unwrap(),
        Some(Embedded {
            written: 0,
            left: 0
        })
    );
}

#[test]
fn a_long_text_is_cut_on_a_character_whole() {
    let long = "ñ".repeat(TEXT_AT_MOST);
    let short = cut(&long);
    assert!(short.len() <= TEXT_AT_MOST);
    assert!(short.len() >= TEXT_AT_MOST - 1);
    assert_eq!(cut("corto"), "corto");
}
