use std::collections::HashSet;

use super::super::likeness::CAPS;
use super::*;

fn caps(standout: f64) -> Caps {
    Caps { standout, ..CAPS }
}

fn meaning(lang: Option<&str>, vector: &[f32]) -> Item {
    Item {
        lang: lang.map(str::to_string),
        kin: Vec::new(),
        words: Vec::new(),
        meant: Some(Meant::of(vector)),
    }
}

const TWINS: [[f32; 8]; 2] = [
    [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1],
    [1.0, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1],
];

// Fillers point every which way off the twins' axis, so the twins stand far above the rest.
fn crowd(how_many: usize) -> Vec<Item> {
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed % 2001) as f32 / 1000.0 - 1.0
    };
    (0..how_many)
        .map(|_| {
            let mut vector = [0f32; 8];
            for one in vector.iter_mut().skip(1) {
                *one = next();
            }
            meaning(None, &vector)
        })
        .collect()
}

fn twins() -> Vec<Item> {
    TWINS.iter().map(|one| meaning(None, one)).collect()
}

fn drawn(items: &[Item], caps: &Caps, tied: &HashSet<(usize, usize)>) -> Vec<(usize, usize)> {
    let apart = |a: usize, b: usize| tied.contains(&(a.min(b), a.max(b)));
    let mut all: Vec<(usize, usize)> = by_meaning(items, &apart, caps)
        .0
        .into_iter()
        .map(|(_, a, b, _)| (a, b))
        .collect();
    all.sort_unstable();
    all
}

#[test]
fn two_that_mean_the_same_and_stand_out_from_the_crowd_are_alike() {
    let mut items = twins();
    items.extend(crowd(60));
    assert_eq!(drawn(&items, &CAPS, &HashSet::new()), [(0, 1)]);
}

#[test]
fn a_pair_that_does_not_stand_out_enough_for_both_ends_is_left() {
    let mut items = twins();
    items.extend(crowd(8));
    assert!(drawn(&items, &CAPS, &HashSet::new()).is_empty());
    assert!(drawn(&items, &caps(1.0), &HashSet::new()).contains(&(0, 1)));
}

#[test]
fn what_is_already_tied_is_never_guessed() {
    let mut items = twins();
    items.extend(crowd(60));
    let tied = HashSet::from([(0, 1)]);
    assert!(!drawn(&items, &CAPS, &tied).contains(&(0, 1)));
}

#[test]
fn another_language_has_its_own_slot_and_the_floor_of_its_own_pairs() {
    let mut items = vec![
        meaning(Some("es"), &[1.0, 0.0, 0.0, 0.1]),
        meaning(Some("en"), &[0.9, 0.1, 0.0, 0.1]),
        meaning(Some("es"), &[0.0, 1.0, 0.0, 0.0]),
        meaning(Some("en"), &[0.0, 0.0, 1.0, 0.0]),
    ];
    items.extend(crowd(4));
    let found = by_meaning(&items, &|_, _| false, &caps(100.0)).0;
    assert_eq!(found.len(), 1, "the same language finds no standout here");
    assert_eq!((found[0].1, found[0].2), (0, 1));
    assert!(found[0].0 == Tongue::Other);
}

#[test]
fn past_its_budget_each_one_meets_only_its_neighbours_around_the_base() {
    let mut items = twins();
    items.extend(crowd(60));
    let m = items.len() as u64;
    let tight = Caps {
        pairs: m * 5,
        ..caps(1.0)
    };
    let (found, compared) = by_meaning(&items, &|_, _| false, &tight);
    assert_eq!(compared, m * 5);
    assert!(found.iter().any(|(_, a, b, _)| (*a, *b) == (0, 1)));
    let (_, whole) = by_meaning(&items, &|_, _| false, &CAPS);
    assert_eq!(whole, m * (m - 1) / 2);
}

#[test]
fn nothing_meant_or_a_single_meaning_compares_nothing() {
    let silent = Item {
        lang: None,
        kin: Vec::new(),
        words: Vec::new(),
        meant: None,
    };
    let empty = meaning(None, &[0.0, 0.0]);
    let one = meaning(None, &[1.0, 0.0]);
    assert_eq!(by_meaning(&[silent, empty, one], &|_, _| false, &CAPS).1, 0);
}

#[test]
fn a_node_that_scores_the_same_with_everything_stands_out_with_nothing() {
    let mut flat = Standing::default();
    for _ in 0..10 {
        flat.add(0.5);
    }
    assert!(flat.standout(0.5).abs() < f64::EPSILON);
    let mut lonely = Standing::default();
    lonely.add(0.9);
    assert!(lonely.standout(0.9).abs() < f64::EPSILON);
}

#[test]
fn the_closest_are_kept_in_order_and_a_tie_goes_to_the_earlier() {
    let mut list = Vec::new();
    for (by, said) in [(5, 0.2), (3, 0.9), (4, 0.5), (2, 0.5)] {
        if admits(&list, 3, by, said) {
            offer(&mut list, 3, by, said);
        }
    }
    assert_eq!(list, [(3, 0.9), (2, 0.5), (4, 0.5)]);
    assert!(!admits(&list, 3, 9, 0.5));
    assert!(admits(&list, 3, 1, 0.5));
    assert!(!admits(&[], 0, 1, 1.0));
}

#[test]
fn two_alone_are_compared_whole_whatever_the_budget() {
    let none = Caps {
        pairs: 0,
        ..caps(1.0)
    };
    assert_eq!(by_meaning(&twins(), &|_, _| false, &none).1, 1);
}
