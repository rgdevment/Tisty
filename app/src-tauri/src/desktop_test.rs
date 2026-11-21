use super::*;

#[test]
fn a_parting_the_window_holds_waits_for_it_instead_of_leaving_on_its_own() {
    let parting = Parting::default();
    let round = parting.asked_again();
    assert!(parting.leaves_alone(round));

    parting.hold();

    assert!(!parting.leaves_alone(round));
}

#[test]
fn quitting_again_after_staying_leaves_on_its_own_unless_held_again() {
    let parting = Parting::default();
    let first = parting.asked_again();
    parting.hold();

    let second = parting.asked_again();

    assert!(!parting.leaves_alone(first));
    assert!(parting.leaves_alone(second));
    parting.hold();
    assert!(!parting.leaves_alone(second));
}
