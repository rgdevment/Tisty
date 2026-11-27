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

#[test]
fn a_window_inside_the_room_stays_where_it_is() {
    assert_eq!(within(100, 400, 0, 1000), 100);
}

#[test]
fn a_window_overflowing_the_end_moves_back_to_fit() {
    assert_eq!(within(800, 400, 0, 1000), 600);
}

#[test]
fn a_window_before_the_start_moves_to_the_start() {
    assert_eq!(within(-50, 400, 0, 1000), 0);
}

#[test]
fn a_second_monitor_left_of_the_primary_keeps_its_negative_start() {
    assert_eq!(within(-1800, 400, -1920, 1920), -1800);
    assert_eq!(within(-2000, 400, -1920, 1920), -1920);
    assert_eq!(within(-300, 400, -1920, 1920), -400);
}

#[test]
fn a_window_larger_than_the_room_ends_at_the_start() {
    assert_eq!(within(50, 1200, 0, 1000), 0);
    assert_eq!(within(50, 1200, -1920, 1000), -1920);
}

#[test]
fn a_tall_window_on_a_screen_with_a_taskbar_moves_up_into_the_work_area() {
    assert_eq!(within(72, 820, 0, 852), 32);
}
