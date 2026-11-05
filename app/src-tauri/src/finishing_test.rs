use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use tisty_sharing::{Finished, Unfinished};

use super::{Errand, Finishers, Heard, waited};

const AT_ONCE: Duration = Duration::from_millis(1);
const AMPLY: Duration = Duration::from_secs(5);

fn asked() -> Vec<Unfinished> {
    vec![Unfinished {
        name: "a".into(),
        html: "<p>a</p>".into(),
    }]
}

fn answer() -> Finished {
    Finished {
        name: "a".into(),
        html: "<p>a</p>".into(),
        not_drawn: 0,
    }
}

fn waiting() -> (Finishers, Receiver<Heard>) {
    let finishers = Finishers::default();
    let (told, heard) = channel();
    finishers.held().insert(
        "finisher-1".into(),
        Errand {
            asked: asked(),
            told,
        },
    );
    (finishers, heard)
}

#[test]
fn only_the_window_opened_for_it_is_told_what_to_finish_however_often_it_loads() {
    let (finishers, heard) = waiting();
    assert!(finishers.asked_of("main").is_none());
    assert!(heard.try_recv().is_err());
    assert_eq!(finishers.asked_of("finisher-1").unwrap(), asked());
    assert_eq!(heard.try_recv().unwrap(), Heard::Began);
    assert_eq!(finishers.asked_of("finisher-1").unwrap(), asked());
}

#[test]
fn an_answer_reaches_who_waits_and_a_stranger_answers_nothing() {
    let (finishers, heard) = waiting();
    finishers.answered("main", Some(vec![]));
    assert!(heard.try_recv().is_err());

    finishers.answered("finisher-1", Some(vec![answer()]));
    assert_eq!(
        heard.try_recv().unwrap(),
        Heard::Ended(Some(vec![answer()]))
    );
    assert!(finishers.held().is_empty());
}

#[test]
fn a_window_that_could_not_finish_ends_the_wait() {
    let (finishers, heard) = waiting();
    finishers.answered("finisher-1", None);
    assert_eq!(waited(&heard, AMPLY, AMPLY), None);
}

#[test]
fn a_page_that_never_asks_ends_the_wait_without_the_long_one() {
    let (_finishers, heard) = waiting();
    let began = std::time::Instant::now();
    assert_eq!(waited(&heard, AT_ONCE, Duration::from_secs(600)), None);
    assert!(began.elapsed() < Duration::from_secs(60));
}

#[test]
fn a_window_that_began_gets_the_long_wait_and_its_answer_arrives() {
    let (finishers, heard) = waiting();
    finishers.asked_of("finisher-1");
    finishers.answered("finisher-1", Some(vec![answer()]));
    assert_eq!(waited(&heard, AT_ONCE, AMPLY), Some(vec![answer()]));
}

#[test]
fn a_page_that_loads_again_may_still_answer() {
    let (finishers, heard) = waiting();
    finishers.asked_of("finisher-1");
    finishers.asked_of("finisher-1");
    finishers.answered("finisher-1", Some(vec![answer()]));
    assert_eq!(waited(&heard, AMPLY, AMPLY), Some(vec![answer()]));
}

#[test]
fn a_page_that_keeps_loading_gets_no_more_time_for_it() {
    let (finishers, heard) = waiting();
    finishers.asked_of("finisher-1");
    finishers.asked_of("finisher-1");
    assert_eq!(waited(&heard, AMPLY, AT_ONCE), None);
}

#[test]
fn a_window_that_began_and_never_ended_is_not_waited_for_past_its_time() {
    let (finishers, heard) = waiting();
    finishers.asked_of("finisher-1");
    assert_eq!(waited(&heard, AMPLY, AT_ONCE), None);
}
