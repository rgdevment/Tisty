use super::*;

#[test]
fn a_shelf_with_nothing_owed_sends_no_history_ahead() {
    for fake in crate::fakes::every() {
        let remote: &dyn Remote = &*fake;
        let room = tempfile::tempdir().unwrap();
        let mut index = Index::default();
        let mut shelf = Shelf::over(remote, &mut index).history_first(room.path());

        assert!(shelf.history_sent().is_ok(), "{}", fake.who());
        assert!(shelf.history_sent().is_ok(), "{}", fake.who());
        assert_eq!(shelf.early(), 0, "{}", fake.who());
    }
}
