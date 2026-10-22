use std::ops::Range;

use super::*;

fn sample() -> Nearby {
    Nearby::of(vec![30..32, 0..100, 10..12, 50..51, 200..210])
}

#[test]
fn ranges_touching_a_piece_are_every_one_that_overlaps_it_even_a_long_one_before() {
    let touching = |within: Range<usize>| {
        let mut found: Vec<Range<usize>> = sample().touching(&within).cloned().collect();
        found.sort_by_key(|one| one.start);
        found
    };

    assert_eq!(touching(40..60), vec![0..100, 50..51]);
    assert_eq!(touching(5..11), vec![0..100, 10..12]);
    assert_eq!(touching(150..190), Vec::<Range<usize>>::new());
    assert_eq!(touching(205..300), vec![200..210]);
}

#[test]
fn a_piece_is_held_only_by_a_range_that_holds_all_of_it() {
    let nearby = sample();

    assert!(nearby.holding(&(40..60)));
    assert!(!nearby.holding(&(95..105)));
    assert!(nearby.holds(201));
    assert!(!nearby.holds(150));
    assert!(!nearby.holds(210));
}

#[test]
fn what_lies_outside_skips_every_silent_range_once_in_order() {
    let silent = Nearby::of(vec![12..14, 2..4, 3..6, 20..30]);

    assert_eq!(
        outside(0..25, silent.near(&(0..25))),
        vec![0..2, 6..12, 14..20]
    );
    assert_eq!(outside(7..11, silent.near(&(7..11))), vec![7..11]);
    assert_eq!(
        outside(4..5, silent.near(&(4..5))),
        Vec::<Range<usize>>::new()
    );
}
