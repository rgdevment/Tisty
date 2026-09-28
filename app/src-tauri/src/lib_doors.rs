use super::offering;

#[test]
fn a_machine_without_an_assistant_is_never_offered_one() {
    assert!(!offering(0, 0));
}

#[test]
fn an_assistant_that_is_found_and_not_yet_wired_is_worth_offering() {
    assert!(offering(1, 0));
    assert!(offering(6, 0));
}

#[test]
fn somebody_who_already_wired_one_knows_where_the_door_is() {
    assert!(!offering(1, 1));
    assert!(!offering(6, 6));
}

#[test]
fn one_wired_assistant_speaks_for_the_rest() {
    assert!(!offering(6, 1));
}
