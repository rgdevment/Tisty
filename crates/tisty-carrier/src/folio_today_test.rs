use super::{DAY, counted, full_now, full_until, shared_today};

#[test]
fn each_new_document_counts_once_however_often_it_is_sent() {
    let home = tempfile::tempdir().unwrap();
    let at = 20_000 * DAY + 10;

    counted(home.path(), "a", at);
    counted(home.path(), "a", at + 60);
    counted(home.path(), "b", at + 120);

    assert_eq!(shared_today(home.path(), at + 180), 2);
}

#[test]
fn a_new_utc_day_starts_the_count_again() {
    let home = tempfile::tempdir().unwrap();
    let last_second = 20_001 * DAY - 1;

    counted(home.path(), "a", last_second);
    assert_eq!(shared_today(home.path(), last_second), 1);
    assert_eq!(shared_today(home.path(), last_second + 1), 0);

    counted(home.path(), "b", last_second + 1);
    assert_eq!(shared_today(home.path(), last_second + 2), 1);
}

#[test]
fn nothing_kept_counts_as_nothing_shared() {
    let home = tempfile::tempdir().unwrap();

    assert_eq!(shared_today(home.path(), 5 * DAY), 0);
}

#[test]
fn a_day_folio_said_is_full_stays_full_until_it_ends_and_survives_a_new_count() {
    let home = tempfile::tempdir().unwrap();
    let at = 20_000 * DAY + 100;
    let tomorrow = 20_001 * DAY;

    assert!(!full_now(home.path(), at));
    full_until(home.path(), tomorrow, at);
    counted(home.path(), "a", at + 10);

    assert!(full_now(home.path(), at + 20));
    assert_eq!(shared_today(home.path(), at + 20), 1);
    assert!(!full_now(home.path(), tomorrow));
}
