use super::*;

#[test]
fn a_full_drive_while_the_history_goes_first_is_told_as_full() {
    for fake in crate::fakes::every() {
        let who = fake.who();
        let remote: &dyn Remote = &*fake;
        let room = tempfile::tempdir().unwrap();
        let segment = room.path().join("store").join("dev_a");
        std::fs::create_dir_all(&segment).unwrap();
        std::fs::write(segment.join("000001.tisty"), b"{}").unwrap();
        let mut index = Index::default();
        fake.fail_next(Hitch::Full);
        let mut shelf = Shelf::over(remote, &mut index).history_first(room.path());

        let hitch = shelf.history_sent();

        assert!(matches!(hitch, Some(Hitch::Full)), "{who}: {hitch:?}");
        assert!(
            shelf.history_sent().is_none(),
            "{who}: the history goes ahead only once"
        );
    }
}

#[test]
fn a_passing_hitch_on_the_history_ahead_is_left_to_the_round_s_own_push() {
    for fake in crate::fakes::every() {
        let who = fake.who();
        let remote: &dyn Remote = &*fake;
        let room = tempfile::tempdir().unwrap();
        let segment = room.path().join("store").join("dev_a");
        std::fs::create_dir_all(&segment).unwrap();
        std::fs::write(segment.join("000001.tisty"), b"{}").unwrap();
        let mut index = Index::default();
        fake.fail_next(Hitch::Broke("a moment".into()));
        let mut shelf = Shelf::over(remote, &mut index).history_first(room.path());

        assert!(shelf.history_sent().is_none(), "{who}");
    }
}
