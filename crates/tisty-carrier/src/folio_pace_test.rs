use super::{QUIET_STARTS_OVER, free_at, press};
use crate::folio_limits::FolioLimits;

const WAITS: &[u64] = &[1, 1, 1, 2, 4, 8, 15];

#[test]
fn a_first_press_never_waits() {
    assert_eq!(free_at(&[], 1_000, WAITS, Some(10)), None);
}

#[test]
fn the_wait_grows_with_each_press_as_the_limits_say() {
    let at = 10_000;
    assert_eq!(free_at(&[at], at, WAITS, None), Some(at + 60));
    let four: Vec<u64> = (0..4).map(|one| at + one * 60).collect();
    assert_eq!(free_at(&four, at + 180, WAITS, None), Some(at + 180 + 120));
}

#[test]
fn past_the_last_wait_it_keeps_the_longest() {
    let many: Vec<u64> = (0..12).map(|one| 10_000 + one * 60).collect();
    let last = *many.last().unwrap();

    assert_eq!(free_at(&many, last, WAITS, None), Some(last + 15 * 60));
}

#[test]
fn thirty_quiet_minutes_start_the_waits_over() {
    let last = 10_000;
    let later = last + QUIET_STARTS_OVER;

    assert_eq!(
        free_at(&[last - 120, last - 60, last], later, WAITS, None),
        None
    );
    assert_eq!(
        free_at(&[last, later], later, WAITS, None),
        Some(later + 60)
    );
}

#[test]
fn the_hourly_most_holds_until_the_oldest_press_leaves_the_hour() {
    let at = 100_000;
    let presses: Vec<u64> = (0..3).map(|one| at + one * 600).collect();
    let now = at + 1_300;

    assert_eq!(free_at(&presses, now, &[], Some(3)), Some(at + 3_600));
    assert_eq!(free_at(&presses, now, &[], Some(4)), None);
}

#[test]
fn it_counts_each_press_and_remembers_it() {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        upload_waits_minutes: Some(vec![1]),
        ..FolioLimits::default()
    };

    assert_eq!(press(home.path(), "a", 1_000, Some(&limits)), Ok(()));
    assert_eq!(press(home.path(), "a", 1_010, Some(&limits)), Err(1_060));
    assert_eq!(press(home.path(), "b", 1_010, Some(&limits)), Ok(()));
    assert_eq!(press(home.path(), "a", 1_060, Some(&limits)), Ok(()));
}

#[test]
fn with_no_word_from_folio_nothing_is_made_up() {
    let home = tempfile::tempdir().unwrap();

    assert_eq!(press(home.path(), "a", 1_000, None), Ok(()));
    assert_eq!(press(home.path(), "a", 1_000, None), Ok(()));
}
