use std::sync::mpsc::channel;

use tisty_sharing::{Finished, Unfinished};

use super::{Errand, Finishers};

fn asked() -> Vec<Unfinished> {
    vec![Unfinished {
        name: "a".into(),
        html: "<p>a</p>".into(),
    }]
}

#[test]
fn only_the_window_opened_for_it_is_told_what_to_finish_and_only_once() {
    let finishers = Finishers::default();
    let (told, _heard) = channel();
    finishers.held().insert(
        "finisher-1".into(),
        Errand {
            asked: asked(),
            answer: told,
        },
    );
    assert!(finishers.asked_of("main").is_none());
    assert_eq!(finishers.asked_of("finisher-1").unwrap(), asked());
    assert!(finishers.asked_of("finisher-1").unwrap().is_empty());
}

#[test]
fn an_answer_reaches_who_waits_and_a_stranger_answers_nothing() {
    let finishers = Finishers::default();
    let (told, heard) = channel();
    finishers.held().insert(
        "finisher-1".into(),
        Errand {
            asked: asked(),
            answer: told,
        },
    );
    finishers.answered("main", Some(vec![]));
    assert!(heard.try_recv().is_err());

    let answer = Finished {
        name: "a".into(),
        html: "<p>a</p>".into(),
        not_drawn: 0,
    };
    finishers.answered("finisher-1", Some(vec![answer.clone()]));
    assert_eq!(heard.try_recv().unwrap(), Some(vec![answer]));
    assert!(finishers.held().is_empty());
}

#[test]
fn a_window_that_could_not_finish_ends_the_wait() {
    let finishers = Finishers::default();
    let (told, heard) = channel();
    finishers.held().insert(
        "finisher-1".into(),
        Errand {
            asked: asked(),
            answer: told,
        },
    );
    finishers.answered("finisher-1", None);
    assert_eq!(heard.try_recv().unwrap(), None);
}
