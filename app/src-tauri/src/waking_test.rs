use super::{HUSHED, UNREACH, told, unreaching};

fn said(args: &[&str]) -> Vec<String> {
    args.iter().map(|one| one.to_string()).collect()
}

#[test]
fn unreach_is_heard_only_when_it_is_all_that_was_said() {
    assert!(unreaching(&said(&["tisty-gui", UNREACH])));

    for args in [
        &["tisty-gui"][..],
        &["tisty-gui", "tisty://open?path=a.md", UNREACH],
        &["tisty-gui", UNREACH, "tisty://open"],
        &["tisty-gui", "--", UNREACH],
        &["tisty-gui", "--unreach "],
    ] {
        assert!(!unreaching(&said(args)), "{args:?}");
    }
}

#[test]
fn a_flag_handed_over_after_the_separator_is_not_one() {
    assert!(told(&said(&["tisty-gui", HUSHED]), HUSHED));
    assert!(!told(&said(&["tisty-gui", "--", HUSHED]), HUSHED));
    assert!(!told(&said(&[HUSHED]), HUSHED));
}
