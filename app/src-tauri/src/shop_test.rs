use super::*;

#[test]
fn the_download_fills_the_bar_on_its_own() {
    assert_eq!(step(0.0), ("getting", 0));
    assert_eq!(step(0.4), ("getting", 50));
    assert_eq!(step(0.79), ("getting", 98));
}

#[test]
fn what_the_store_counts_as_the_install_is_shown_as_installing() {
    assert_eq!(step(0.8), ("installing", 100));
    assert_eq!(step(0.9), ("installing", 100));
    assert_eq!(step(1.0), ("installing", 100));
}

#[test]
fn a_figure_that_makes_no_sense_still_leaves_a_bar_that_does() {
    assert_eq!(step(-1.0), ("getting", 0));
    assert_eq!(step(2.0), ("installing", 100));
}
