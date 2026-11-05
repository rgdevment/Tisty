use super::{HUSHED, UNREACH, came_back, told, unreaching};
use crate::update::Route;

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
fn only_the_plugin_brings_a_hidden_copy_back_in_view() {
    assert!(came_back(Route::Download, Some("1.42.0"), "1.42.0"));
    assert!(came_back(Route::Brew, Some("1.42.0"), "1.42.0"));

    assert!(!came_back(Route::Store, Some("1.42.0"), "1.42.0"));
    assert!(!came_back(Route::Download, Some("1.42.0"), "1.41.1"));
    assert!(!came_back(Route::Download, None, "1.42.0"));
}

#[test]
fn a_flag_handed_over_after_the_separator_is_not_one() {
    assert!(told(&said(&["tisty-gui", HUSHED]), HUSHED));
    assert!(!told(&said(&["tisty-gui", "--", HUSHED]), HUSHED));
    assert!(!told(&said(&[HUSHED]), HUSHED));
}
