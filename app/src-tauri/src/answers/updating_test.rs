use super::*;

#[test]
fn a_look_counts_as_heard_only_when_the_store_or_the_catalog_answered() {
    assert!(!answered(&shop::Shelf::Silent, &update::Ahead::Unheard));
    assert!(answered(&shop::Shelf::Current, &update::Ahead::Unheard));
    assert!(answered(&shop::Shelf::Silent, &update::Ahead::Nothing));
    assert!(answered(
        &shop::Shelf::Queued,
        &update::Ahead::Is("1.50.0".into())
    ));
}
