use super::*;
use tisty_core::standing::DUE_SOON_DAYS;

#[test]
fn the_rules_say_the_same_threshold_the_core_uses() {
    let days = DUE_SOON_DAYS;

    assert!(
        TAUGHT.contains(&format!("more than {days} days before")),
        "underway"
    );
    assert!(
        TAUGHT.contains(&format!("({days} days or less")),
        "due soon"
    );
}
