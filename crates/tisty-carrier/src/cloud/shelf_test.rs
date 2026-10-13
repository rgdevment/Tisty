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

#[test]
fn a_full_drive_while_the_history_goes_first_stops_the_round_as_full() {
    for fake in crate::fakes::every() {
        let who = fake.who();
        let remote: &dyn Remote = &*fake;
        let room = tempfile::tempdir().unwrap();
        let segment = room.path().join("store").join("dev_a");
        std::fs::create_dir_all(&segment).unwrap();
        std::fs::write(
            segment.join("0001.jsonl"),
            b"{}
",
        )
        .unwrap();
        let mut index = Index::default();
        fake.fail_next(Hitch::Full);
        let mut shelf = Shelf::over(remote, &mut index).history_first(room.path());

        let sent = shelf.history_sent();

        assert!(
            matches!(sent, Err(Trouble::CloudFull { .. })),
            "{who}: {sent:?}"
        );
    }
}
