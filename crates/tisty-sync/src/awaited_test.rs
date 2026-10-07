use super::*;

const HOUR: u64 = LANDING.as_secs();
const DAY: u64 = REMEMBERED.as_secs();

fn alive() -> Vec<String> {
    vec!["doc-0001".to_string()]
}

#[test]
fn a_body_waits_an_hour_from_the_first_time_it_was_seen() {
    let mut awaited = Awaited::default();

    assert!(awaited.still_landing("doc-0001", 1_000));
    assert!(awaited.still_landing("doc-0001", 1_000 + HOUR - 1));
    assert!(!awaited.still_landing("doc-0001", 1_000 + HOUR));
}

#[test]
fn seeing_it_again_never_starts_the_wait_over() {
    let mut awaited = Awaited::default();

    awaited.still_landing("doc-0001", 1_000);
    awaited.still_landing("doc-0001", 1_000 + HOUR / 2);

    assert!(!awaited.still_landing("doc-0001", 1_000 + HOUR));
}

#[test]
fn a_clock_turned_back_holds_the_body_no_longer_than_an_hour() {
    let mut awaited = Awaited::default();

    awaited.still_landing("doc-0001", 10_000);

    assert!(awaited.still_landing("doc-0001", 5_000));
    assert!(!awaited.still_landing("doc-0001", 5_000 + HOUR));
}

#[test]
fn once_it_arrives_the_next_one_waits_afresh() {
    let mut awaited = Awaited::default();

    awaited.still_landing("doc-0001", 1_000);
    awaited.arrived("doc-0001");

    assert!(awaited.still_landing("doc-0001", 1_000 + HOUR * 2));
}

#[test]
fn a_body_planted_again_within_the_day_is_asked_about_at_once() {
    let mut awaited = Awaited::default();
    awaited.still_landing("doc-0001", 1_000);
    let mut later = Awaited {
        since: awaited.since.clone(),
        checked: BTreeSet::new(),
    };

    later.forget_settled(&alive(), 1_000 + HOUR * 3);

    assert!(!later.still_landing("doc-0001", 1_000 + HOUR * 3));
}

#[test]
fn a_document_settled_long_ago_gets_its_hour_again() {
    let mut awaited = Awaited::default();
    awaited.still_landing("doc-0001", 1_000);
    let mut later = Awaited {
        since: awaited.since.clone(),
        checked: BTreeSet::new(),
    };

    later.forget_settled(&alive(), 1_000 + DAY);

    assert!(later.still_landing("doc-0001", 1_000 + DAY));
}

#[test]
fn one_still_waiting_is_never_forgotten_however_long_it_waits() {
    let mut awaited = Awaited::default();
    awaited.still_landing("doc-0001", 1_000);
    awaited.still_landing("doc-0001", 1_000 + DAY * 2);

    awaited.forget_settled(&alive(), 1_000 + DAY * 2);

    assert!(awaited.knows("doc-0001"));
}

#[test]
fn what_it_remembers_outlives_the_round_and_forgets_what_is_gone() {
    let room = tempfile::tempdir().unwrap();
    let mut awaited = Awaited::default();
    awaited.still_landing("doc-0001", 1_000);
    awaited.still_landing("doc-0002", 2_000);
    awaited.forget_settled(&["doc-0002".to_string()], 2_000);
    awaited.save(room.path());

    let read = Awaited::read(room.path());

    assert_eq!(read.since, awaited.since);
    assert_eq!(read.since.len(), 1);

    Awaited::default().save(room.path());
    assert!(!room.path().join("awaited").exists());
}
