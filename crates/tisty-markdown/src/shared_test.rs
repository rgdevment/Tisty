use super::shared;

fn longest(a: &[u8], b: &[u8]) -> usize {
    let mut kept = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for i in 0..a.len() {
        for j in 0..b.len() {
            kept[i + 1][j + 1] = match a[i] == b[j] {
                true => kept[i][j] + 1,
                false => kept[i][j + 1].max(kept[i + 1][j]),
            };
        }
    }
    kept[a.len()][b.len()]
}

fn rolled(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *seed >> 33
}

fn some(seed: &mut u64) -> Vec<u8> {
    (0..rolled(seed) % 24)
        .map(|_| (rolled(seed) % 5) as u8)
        .collect()
}

#[test]
fn the_pairs_are_a_longest_run_both_hold_in_order() {
    let mut seed = 7;
    for _ in 0..800 {
        let (a, b) = (some(&mut seed), some(&mut seed));
        let pairs = shared(&a, &b, usize::MAX).unwrap();

        assert_eq!(pairs.len(), longest(&a, &b), "{a:?} against {b:?}");
        assert!(pairs.iter().all(|(i, j)| a[*i] == b[*j]), "{pairs:?}");
        assert!(
            pairs
                .windows(2)
                .all(|two| two[0].0 < two[1].0 && two[0].1 < two[1].1),
            "{pairs:?}"
        );
    }
}

#[test]
fn what_both_open_and_close_with_costs_no_cell() {
    let mut a: Vec<u32> = (0..50_000).collect();
    let mut b = a.clone();
    a[25_000] = 99_999_999;
    b.insert(25_001, 88_888_888);

    let pairs = shared(&a, &b, 16).unwrap();

    assert_eq!(pairs.len(), 49_999);
    assert_eq!(pairs[24_999], (24_999, 24_999));
    assert_eq!(pairs[25_000], (25_001, 25_002));
}

#[test]
fn two_that_say_the_same_are_never_too_long() {
    let a: Vec<u32> = (0..100_000).collect();

    assert_eq!(shared(&a, &a, 0).map(|pairs| pairs.len()), Some(100_000));
}

#[test]
fn past_the_ceiling_between_what_they_share_nothing_is_told() {
    let (a, b): (Vec<u32>, Vec<u32>) = ((0..10).collect(), (100..110).collect());

    assert_eq!(shared(&a, &b, 120), None);
    assert_eq!(shared(&a, &b, 121), Some(Vec::new()));
}

#[test]
fn one_side_empty_shares_nothing_and_costs_nothing() {
    let a: Vec<u32> = (0..1_000_000).collect();

    assert_eq!(shared(&a, &[], 0), Some(Vec::new()));
    assert_eq!(shared(&[], &a, 0), Some(Vec::new()));
}
