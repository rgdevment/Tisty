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
        let mut shelf = Shelf::over(remote, &mut index).history_first(room.path(), remote);
        let mut quiet = |_| {};
        let mut pace = Pace::new(None, &mut quiet);

        let hitch = shelf.history_sent(&mut pace);

        assert!(matches!(hitch, Some(Hitch::Full)), "{who}: {hitch:?}");
        assert!(
            shelf.history_sent(&mut pace).is_none(),
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
        let mut shelf = Shelf::over(remote, &mut index).history_first(room.path(), remote);
        let mut quiet = |_| {};

        assert!(
            shelf
                .history_sent(&mut Pace::new(None, &mut quiet))
                .is_none(),
            "{who}"
        );
    }
}

#[test]
fn what_an_older_build_left_on_shelves_up_there_moves_to_the_one_flat_folder() {
    for fake in crate::fakes::every() {
        let who = fake.who();
        let remote: &dyn Remote = &*fake;
        let room = tempfile::tempdir().unwrap();
        let mut index = Index::default();
        for (name, body) in [
            ("attachments/23/plano-0006bf09.png", &b"plano"[..]),
            ("attachments/ab/acta-1234abcd.pdf", b"acta"),
            ("attachments/acta-1234abcd.pdf", b"acta"),
        ] {
            let local = room.path().join(name.replace('/', "-"));
            std::fs::write(&local, body).unwrap();
            let seen = remote.put(name, &local, Expect::Absent).unwrap();
            index.shelved(name, seen);
        }

        assert_eq!(flattened(remote, &mut index), 2, "{who}");

        let names: Vec<&str> = index.shelf.keys().map(String::as_str).collect();
        assert_eq!(
            names,
            [
                "attachments/acta-1234abcd.pdf",
                "attachments/plano-0006bf09.png"
            ],
            "{who}"
        );
        let mut up: Vec<String> = remote
            .list("attachments")
            .unwrap()
            .into_iter()
            .map(|one| one.name)
            .collect();
        up.sort();
        assert_eq!(
            up,
            [
                "attachments/acta-1234abcd.pdf",
                "attachments/plano-0006bf09.png"
            ],
            "{who}"
        );
    }
}
