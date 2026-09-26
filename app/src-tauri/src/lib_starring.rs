use super::{Asking, CLOSED_ENOUGH, PAPERS_ENOUGH, asking};

fn at(text: &str) -> jiff::Timestamp {
    text.parse().expect("a timestamp")
}

fn now() -> jiff::Timestamp {
    at("2026-09-22T12:00:00Z")
}

const LONG_AGO: &str = "2026-01-01T00:00:00Z";

#[test]
fn a_copy_that_was_asked_once_is_never_asked_again() {
    assert_eq!(
        asking(Some(true), Some(at(LONG_AGO)), now(), 10_000, || 10_000),
        Asking::Wait
    );
}

#[test]
fn a_copy_with_no_mark_lays_one_and_says_nothing_yet() {
    assert_eq!(asking(None, None, now(), 10_000, || 10_000), Asking::Start);
}

#[test]
fn a_mark_from_a_clock_that_was_ahead_is_laid_again_rather_than_waited_on_for_ever() {
    assert_eq!(
        asking(
            None,
            Some(at("2030-01-01T00:00:00Z")),
            now(),
            10_000,
            || { 10_000 }
        ),
        Asking::Start
    );
}

#[test]
fn a_fortnight_is_the_floor_and_the_day_before_it_is_not() {
    assert_eq!(
        asking(
            None,
            Some(at("2026-09-08T12:00:01Z")),
            now(),
            10_000,
            || { 10_000 }
        ),
        Asking::Wait
    );
    assert_eq!(
        asking(
            None,
            Some(at("2026-09-08T12:00:00Z")),
            now(),
            10_000,
            || { 10_000 }
        ),
        Asking::Now
    );
}

#[test]
fn either_floor_opens_it_and_neither_alone_being_short_does() {
    let long_ago = Some(at(LONG_AGO));
    assert_eq!(
        asking(None, long_ago, now(), PAPERS_ENOUGH - 1, || CLOSED_ENOUGH
            - 1),
        Asking::Wait
    );
    assert_eq!(
        asking(None, long_ago, now(), PAPERS_ENOUGH, || CLOSED_ENOUGH - 1),
        Asking::Now
    );
    assert_eq!(
        asking(None, long_ago, now(), PAPERS_ENOUGH - 1, || CLOSED_ENOUGH),
        Asking::Now
    );
}

#[test]
fn the_archive_is_not_walked_when_the_answer_is_known_without_it() {
    let walked = std::cell::Cell::new(0);
    let count = || {
        walked.set(walked.get() + 1);
        10_000
    };
    assert_eq!(asking(Some(true), None, now(), 0, count), Asking::Wait);
    assert_eq!(walked.get(), 0);

    let count = || {
        walked.set(walked.get() + 1);
        10_000
    };
    assert_eq!(asking(None, Some(now()), now(), 0, count), Asking::Wait);
    assert_eq!(walked.get(), 0);

    let count = || {
        walked.set(walked.get() + 1);
        10_000
    };
    assert_eq!(
        asking(None, Some(at(LONG_AGO)), now(), PAPERS_ENOUGH, count),
        Asking::Now
    );
    assert_eq!(walked.get(), 0);
}
