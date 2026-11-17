use super::*;

fn noting(said: &Arc<Mutex<Vec<&'static str>>>, word: &'static str) -> Call {
    let said = Arc::clone(said);
    Box::new(move || said.lock().unwrap().push(word))
}

#[test]
fn keeps_what_is_asked_while_starting_and_hands_it_back_in_order() {
    let starting = Starting::default();
    let said = Arc::new(Mutex::new(Vec::new()));
    assert!(!starting.over());
    assert!(starting.keep(noting(&said, "snapshot")).is_none());
    assert!(starting.keep(noting(&said, "locale")).is_none());
    assert!(said.lock().unwrap().is_empty());

    for call in starting.finished() {
        call();
    }
    assert_eq!(*said.lock().unwrap(), ["snapshot", "locale"]);
    assert!(starting.over());
}

#[test]
fn gives_back_what_is_asked_once_the_start_is_over() {
    let starting = Starting::default();
    assert!(starting.finished().is_empty());
    let said = Arc::new(Mutex::new(Vec::new()));
    let now = starting.keep(noting(&said, "snapshot"));

    now.expect("a call after the start runs at once")();
    assert_eq!(*said.lock().unwrap(), ["snapshot"]);
    assert!(starting.finished().is_empty());
}
