use super::*;

fn said(newest: &str, others: &[&str], waiting: &[&str]) -> Answers {
    Answers {
        newest: Some(newest.into()),
        own: None,
        others: others.iter().map(|one| one.to_string()).collect(),
        waiting: waiting.iter().map(|one| one.to_string()).collect(),
    }
}

#[test]
fn a_body_held_by_a_waiting_machine_waits() {
    let says = said("la-ultima", &[], &["la-del-otro"]);
    assert_eq!(
        answered_for(Some(&"la-del-otro".to_string()), Some(&says), "doc-0001"),
        Answer::Waits
    );
}

#[test]
fn a_body_nobody_answers_for_is_still_put_to_the_person() {
    let says = said("la-ultima", &["una-vieja"], &["la-del-otro"]);
    assert_eq!(
        answered_for(Some(&"de-nadie".to_string()), Some(&says), "doc-0001"),
        Answer::No
    );
}

#[test]
fn what_a_trusted_history_says_comes_first() {
    let says = said("la-ultima", &["una-vieja"], &["la-ultima", "una-vieja"]);
    assert_eq!(
        answered_for(Some(&"la-ultima".to_string()), Some(&says), "doc-0001"),
        Answer::Yes
    );
    assert_eq!(
        answered_for(Some(&"una-vieja".to_string()), Some(&says), "doc-0001"),
        Answer::Doubtful
    );
}
