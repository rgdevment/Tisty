use super::{NETWORK, QUIET_STARTS_OVER, free_at, hold, press, take_back, waiting_until};
use crate::folio_limits::FolioLimits;

const WAITS: &[u64] = &[1, 1, 1, 2, 4, 8, 15];

#[test]
fn a_press_taken_back_leaves_the_next_one_free() {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        upload_waits_minutes: Some(vec![1, 2]),
        ..FolioLimits::default()
    };
    press(home.path(), "d", 1000, Some(&limits)).unwrap();
    assert!(press(home.path(), "d", 1001, Some(&limits)).is_err());

    take_back(home.path(), "d", 1000);

    assert_eq!(press(home.path(), "d", 1001, Some(&limits)), Ok(()));
}

#[test]
fn only_the_press_named_is_taken_back() {
    let home = tempfile::tempdir().unwrap();
    press(home.path(), "d", 1000, None).unwrap();
    let limits = FolioLimits {
        upload_waits_minutes: Some(vec![1]),
        ..FolioLimits::default()
    };

    take_back(home.path(), "d", 999);
    take_back(home.path(), "other", 1000);

    assert!(press(home.path(), "d", 1001, Some(&limits)).is_err());
}

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

#[test]
fn a_ladder_of_one_step_makes_every_press_wait_that_long() {
    let home = tempfile::tempdir().unwrap();
    let limits = FolioLimits {
        upload_waits_minutes: Some(vec![5]),
        uploads_of_a_root_an_hour: Some(10),
        ..FolioLimits::default()
    };
    let at = 10_000;

    assert_eq!(waiting_until(home.path(), "d", at, Some(&limits)), None);
    for step in 0..4 {
        let now = at + step * 300;
        assert_eq!(press(home.path(), "d", now, Some(&limits)), Ok(()));
        assert_eq!(
            press(home.path(), "d", now + 299, Some(&limits)),
            Err(now + 300)
        );
        assert_eq!(
            waiting_until(home.path(), "d", now + 1, Some(&limits)),
            Some(now + 300)
        );
    }
    assert_eq!(
        waiting_until(home.path(), "d", at + 1, None),
        None,
        "only the signed limits make it wait"
    );
}

#[test]
fn what_folio_holds_back_stops_a_press_here_until_then() {
    let home = tempfile::tempdir().unwrap();

    hold(home.path(), "d", 1_300, 1_000);

    assert_eq!(press(home.path(), "d", 1_100, None), Err(1_300));
    assert_eq!(waiting_until(home.path(), "d", 1_100, None), Some(1_300));
    assert_eq!(press(home.path(), "other", 1_100, None), Ok(()));
    assert_eq!(press(home.path(), "d", 1_300, None), Ok(()));
}

#[test]
fn a_network_held_back_holds_every_document() {
    let home = tempfile::tempdir().unwrap();

    hold(home.path(), NETWORK, 5_000, 1_000);

    assert_eq!(press(home.path(), "a", 2_000, None), Err(5_000));
    assert_eq!(waiting_until(home.path(), "b", 2_000, None), Some(5_000));
}

#[test]
fn a_hold_keeps_the_later_end_and_forgets_what_ended() {
    let home = tempfile::tempdir().unwrap();

    hold(home.path(), "d", 1_500, 1_000);
    hold(home.path(), "d", 1_200, 1_000);
    assert_eq!(waiting_until(home.path(), "d", 1_100, None), Some(1_500));

    hold(home.path(), "e", 9_000, 2_000);
    let kept = std::fs::read_to_string(home.path().join("folio-held-until.json")).unwrap();
    assert!(!kept.contains("\"d\""), "{kept}");
}
