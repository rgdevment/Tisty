use super::*;

#[test]
fn a_write_waits_for_the_writing_to_settle() {
    assert_eq!(due(true, Duration::from_secs(1)), None);
    assert_eq!(due(true, SETTLES), Some(Round::Nudged));
}

#[test]
fn a_write_after_a_long_quiet_is_never_taken_on_trust() {
    assert_eq!(due(true, EVERY), Some(Round::Nudged));
}

#[test]
fn quiet_rounds_come_on_their_own_and_only_then() {
    assert_eq!(due(false, SETTLES), None);
    assert_eq!(due(false, EVERY), Some(Round::Looking));
}

#[test]
fn the_dictionary_is_looked_for_where_the_bundler_puts_resources() {
    let exe = Path::new("install").join("Tisty.exe");
    let found = beside(&exe);
    assert!(found.ends_with(DICTIONARY));
    assert!(found.starts_with("install"));
}
