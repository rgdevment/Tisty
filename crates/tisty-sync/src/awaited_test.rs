use super::*;

fn alive() -> Vec<String> {
    vec!["doc-0001".to_string()]
}

fn a_later_round(awaited: &Awaited) -> Awaited {
    Awaited {
        since: awaited.since.clone(),
        read: awaited.since.clone(),
        ..Awaited::default()
    }
}

#[test]
fn a_body_waits_an_hour_from_the_first_time_it_was_seen() {
    let mut awaited = Awaited::default();

    assert!(awaited.still_landing("doc-0001", 1_000));
    assert!(awaited.still_landing("doc-0001", 1_000 + LANDING - 1));
    assert!(!awaited.still_landing("doc-0001", 1_000 + LANDING));
}

#[test]
fn seeing_it_again_never_starts_the_wait_over() {
    let mut awaited = Awaited::default();

    awaited.still_landing("doc-0001", 1_000);
    awaited.still_landing("doc-0001", 1_000 + LANDING / 2);

    assert!(!awaited.still_landing("doc-0001", 1_000 + LANDING));
}

#[test]
fn a_clock_turned_back_holds_the_body_no_longer_than_an_hour() {
    let mut awaited = Awaited::default();

    awaited.still_landing("doc-0001", 10_000);

    assert!(awaited.still_landing("doc-0001", 5_000));
    assert!(!awaited.still_landing("doc-0001", 5_000 + LANDING));
}

#[test]
fn once_it_arrives_the_next_one_waits_afresh() {
    let mut awaited = Awaited::default();

    awaited.still_landing("doc-0001", 1_000);
    awaited.arrived(&alive());

    assert!(awaited.still_landing("doc-0001", 1_000 + LANDING * 2));
}

#[test]
fn a_body_planted_again_within_a_day_of_its_last_wait_is_asked_about_at_once() {
    let mut awaited = Awaited::default();
    awaited.still_landing("doc-0001", 1_000);
    awaited.still_landing("doc-0001", 1_000 + REMEMBERED * 2);
    let mut later = a_later_round(&awaited);

    later.forget_settled(&alive(), 1_000 + REMEMBERED * 2 + LANDING);

    assert!(!later.still_landing("doc-0001", 1_000 + REMEMBERED * 2 + LANDING));
}

#[test]
fn a_document_that_last_waited_long_ago_gets_its_hour_again() {
    let mut awaited = Awaited::default();
    awaited.still_landing("doc-0001", 1_000);
    let mut later = a_later_round(&awaited);

    later.forget_settled(&alive(), 1_000 + REMEMBERED);

    assert!(later.still_landing("doc-0001", 1_000 + REMEMBERED));
}

#[test]
fn one_still_waiting_is_never_forgotten_however_long_it_waits() {
    let mut awaited = Awaited::default();
    awaited.still_landing("doc-0001", 1_000);
    awaited.still_landing("doc-0001", 1_000 + REMEMBERED * 2);

    awaited.forget_settled(&alive(), 1_000 + REMEMBERED * 2);

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

    let mut emptied = read.clone();
    emptied.arrived(&["doc-0002".to_string()]);
    emptied.save(room.path());
    assert!(!room.path().join("awaited").exists());
}

#[test]
fn a_ledger_written_with_one_time_reads_as_first_and_last() {
    let room = tempfile::tempdir().unwrap();
    std::fs::write(room.path().join("awaited"), "doc-0001 0\n").unwrap();

    let mut read = Awaited::read(room.path());

    assert!(!read.still_landing("doc-0001", LANDING));
}

#[test]
fn a_ledger_that_cannot_be_read_is_never_written_over() {
    let room = tempfile::tempdir().unwrap();
    std::fs::create_dir(room.path().join("awaited")).unwrap();

    let mut read = Awaited::read(room.path());
    read.still_landing("doc-0001", 1_000);
    read.save(room.path());

    assert!(room.path().join("awaited").is_dir());
}
