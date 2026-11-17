use super::*;

fn noting(said: &Arc<Mutex<Vec<&'static str>>>, word: &'static str) -> Call {
    let said = Arc::clone(said);
    Box::new(move || said.lock().unwrap().push(word))
}

#[test]
fn keeps_what_is_asked_while_starting_and_answers_it_in_order() {
    let starting = Starting::default();
    let said = Arc::new(Mutex::new(Vec::new()));
    assert!(!starting.over());
    assert!(starting.keep(noting(&said, "snapshot")).is_none());
    assert!(starting.keep(noting(&said, "locale")).is_none());
    assert!(said.lock().unwrap().is_empty());

    starting.finish();
    assert_eq!(*said.lock().unwrap(), ["snapshot", "locale"]);
    assert!(starting.over());
}

#[test]
fn answers_what_is_asked_while_the_kept_calls_run_after_them() {
    let starting = Arc::new(Starting::default());
    let said = Arc::new(Mutex::new(Vec::new()));
    let again = Arc::clone(&starting);
    let late = Arc::clone(&said);
    starting.keep(Box::new(move || {
        late.lock().unwrap().push("settle_in");
        let kept = again.keep(noting(&late, "snapshot"));
        assert!(
            kept.is_none(),
            "a call asked during the drain waits its turn"
        );
    }));
    starting.keep(noting(&said, "locale"));

    starting.finish();
    assert_eq!(*said.lock().unwrap(), ["settle_in", "locale", "snapshot"]);
}

#[test]
fn gives_back_what_is_asked_once_the_start_is_over() {
    let starting = Starting::default();
    starting.finish();
    let said = Arc::new(Mutex::new(Vec::new()));
    let now = starting.keep(noting(&said, "snapshot"));

    now.expect("a call after the start runs at once")();
    assert_eq!(*said.lock().unwrap(), ["snapshot"]);
}
