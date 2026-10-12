use super::*;

#[test]
fn a_number_of_days_no_person_could_choose_is_refused() {
    assert!(since(0).is_err());
    assert!(since(MOST_DAYS + 1).is_err());
    assert!(since(u64::MAX).is_err());
}

#[test]
fn the_days_chosen_reach_back_from_now() {
    let thirty = since(30).unwrap();
    let back = SystemTime::now().duration_since(thirty).unwrap();

    assert!(back >= Duration::from_secs(30 * A_DAY));
    assert!(back < Duration::from_secs(30 * A_DAY + 60));
}
