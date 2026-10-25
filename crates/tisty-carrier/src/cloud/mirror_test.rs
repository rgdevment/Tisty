use super::*;
use crate::cloud::budget::{Class, Metered};
use crate::fakes::{Counting, Fake};

fn refused_up_there(tree: &Path, fake: &Fake, docs: usize) {
    let local = tree.with_extension("theirs");
    std::fs::write(&local, b"lo de otra maquina").unwrap();
    std::fs::create_dir_all(tree.join(PAPERS)).unwrap();
    for at in 0..docs {
        let name = format!("{PAPERS}/dev_a-{at:04}.md");
        fake.put(&name, &local, Expect::Absent).unwrap();
        std::fs::write(tree.join(&name), b"lo de esta maquina").unwrap();
    }
    std::fs::remove_file(local).unwrap();
}

#[test]
fn a_push_whose_every_file_is_refused_and_written_down_is_never_taken_for_a_loop() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    let fake = Fake::drive();
    refused_up_there(&tree, &fake, 150);
    let meter = Metered::over(&fake, room.path(), Class::Attachments);
    let mut index = Index::default();

    let (pushed, hitch) = push(&meter, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));

    assert_eq!(hitch, None, "a push of refused files was stopped as a loop");
    assert_eq!(pushed.left, 150);
    assert_eq!(
        index.owed.len(),
        150,
        "what refused the writes was not written down"
    );
}

#[test]
fn the_same_refusal_asked_again_and_again_still_stops_as_a_loop() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    let fake = Fake::drive();
    refused_up_there(&tree, &fake, 1);
    let meter = Metered::over(&fake, room.path(), Class::Attachments);
    let mut index = Index::default();

    let mut pushes = 0;
    let stopped = loop {
        let (_, hitch) = push(&meter, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));
        if let Some(hitch) = hitch {
            break hitch;
        }
        pushes += 1;
        assert!(
            pushes < 1_000,
            "asking the same refused file forever was never stopped"
        );
    };

    assert!(matches!(stopped, Hitch::Troubled(_)), "{stopped:?}");
}

fn echoed_while_owed(fake: Fake) {
    let room = tempfile::tempdir().unwrap();
    let (tree, data) = (room.path().join("tree"), room.path().join("data"));
    let id = "dev_a-0001";
    let name = format!("{PAPERS}/{id}.md");
    let mut index = Index::default();
    pull(&fake, &tree, &mut index, Some(&data), false, &mut |_| {}).unwrap();
    std::fs::create_dir_all(tree.join(PAPERS)).unwrap();
    std::fs::write(tree.join(&name), b"lo primero\n").unwrap();
    let (pushed, _) = push(&fake, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));
    assert_eq!(pushed.put, 1);

    let echo = room.path().join("echo");
    std::fs::write(&echo, b"lo primero\n").unwrap();
    fake.put(&name, &echo, Expect::Any).unwrap();
    let ours = "lo primero\ny lo que vino despues\n";
    std::fs::write(tree.join(&name), ours).unwrap();
    let print = tisty_core::attach::printed(ours.as_bytes());
    std::fs::create_dir_all(&data).unwrap();
    let mut said = tisty_core::docs::Carried::read(&data);
    said.keep(id, &print);
    said.save(&data).unwrap();
    tisty_core::docs::keep_carried(&data, id, ours).unwrap();

    pull(&fake, &tree, &mut index, Some(&data), false, &mut |_| {}).unwrap();

    assert_eq!(
        tisty_core::docs::Carried::read(&data).of(id),
        Some(print.as_str()),
        "our own write coming back took the document's base away"
    );
    assert_eq!(
        tisty_core::docs::read_carried(&data, id).as_deref(),
        Some(ours)
    );
    assert_eq!(std::fs::read_to_string(tree.join(&name)).unwrap(), ours);
    let (pushed, hitch) = push(&fake, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));
    assert_eq!(
        (pushed.put, hitch),
        (1, None),
        "the change owed did not go up"
    );
    let mut up_there = Vec::new();
    fake.fetch(&name, 0, &mut up_there).unwrap();
    assert_eq!(up_there, ours.as_bytes());
}

#[test]
fn our_own_write_coming_back_leaves_a_change_still_owed_with_its_base() {
    echoed_while_owed(Fake::drive());
}

#[test]
fn our_own_write_coming_back_without_a_hash_leaves_a_change_still_owed_with_its_base() {
    echoed_while_owed(Fake::drive().without_hashes());
}

#[test]
fn a_stamp_already_up_there_goes_up_before_the_rest_when_the_place_is_raised() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(tree.join(PAPERS)).unwrap();
    let fake = Fake::drive();
    let meter = Metered::over(&fake, room.path(), Class::Attachments);
    let mut index = Index::default();
    std::fs::write(tree.join(NAMED), "shape = 2\n").unwrap();
    let (_, hitch) = push(&meter, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));
    assert_eq!(hitch, None);

    std::fs::write(tree.join(NAMED), "shape = 3\n").unwrap();
    std::fs::write(tree.join(PAPERS).join("dev_a-0001.md"), "# Turned\n").unwrap();
    fake.refusing(PAPERS, Hitch::Broke("not taken".into()));
    let (_, hitch) = push(&meter, &tree, &mut index, &mut Pace::new(None, &mut |_| {}));

    assert!(hitch.is_some(), "the refused document went up");
    let mut there = Vec::new();
    fake.fetch(NAMED, 0, &mut there).unwrap();
    assert_eq!(
        there, b"shape = 3\n",
        "older builds could take what was sent before learning the place was raised"
    );
}

#[test]
fn a_cloud_asking_for_a_later_tisty_brings_nothing_else_down() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(&tree).unwrap();
    let fake = Fake::drive();
    let said = room.path().join("said");
    std::fs::write(&said, "shape = 3\nneeds = \"999.0.0\"\n").unwrap();
    fake.put(NAMED, &said, Expect::Absent).unwrap();
    std::fs::write(&said, b"\xffTCS sealed by a later build").unwrap();
    fake.put(&format!("{PAPERS}/dev_b-0001.md"), &said, Expect::Absent)
        .unwrap();
    let mut index = Index::default();

    let pulled = pull(&fake, &tree, &mut index, None, false, &mut |_| {});

    assert_eq!(
        pulled,
        Err(Hitch::Barred(tisty_sync::Trouble::Needs(
            "999.0.0".to_string()
        )))
    );
    assert!(!tree.join(PAPERS).join("dev_b-0001.md").exists());
    assert!(index.owed.is_empty(), "{:?}", index.owed.keys());
    assert!(index.cursor.is_none(), "the listing was taken as read");
}

#[test]
fn a_place_whose_word_cannot_come_down_brings_nothing_else_either() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(&tree).unwrap();
    let fake = Fake::drive();
    let said = room.path().join("said");
    std::fs::write(&said, "shape = 3\nneeds = \"999.0.0\"\n").unwrap();
    fake.put(NAMED, &said, Expect::Absent).unwrap();
    fake.put(&format!("{PAPERS}/dev_b-0001.md"), &said, Expect::Absent)
        .unwrap();
    fake.truncate_next(1);
    let mut index = Index::default();

    let pulled = pull(&fake, &tree, &mut index, None, false, &mut |_| {});

    assert!(matches!(pulled, Err(Hitch::Troubled(_))), "{pulled:?}");
    assert!(!tree.join(PAPERS).join("dev_b-0001.md").exists());
    assert_eq!(index.owed.keys().collect::<Vec<_>>(), [NAMED]);
    assert!(index.cursor.is_none(), "the listing was taken as read");
}

fn word_up_there(fake: &Fake, room: &Path, needs: &str) {
    let said = room.join("said");
    std::fs::write(&said, format!("shape = 3\nneeds = \"{needs}\"\n")).unwrap();
    let expect = match fake.about(NAMED).unwrap() {
        Some(seen) => Expect::Revision(seen.revision),
        None => Expect::Absent,
    };
    fake.put(NAMED, &said, expect).unwrap();
}

#[test]
fn a_cloud_found_barred_is_asked_only_for_its_word_until_it_changes() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(&tree).unwrap();
    let fake = Fake::drive();
    word_up_there(&fake, room.path(), "999.0.0");
    let mut index = Index::default();
    let barred = pull(&fake, &tree, &mut index, None, false, &mut |_| {});
    assert!(matches!(barred, Err(Hitch::Barred(_))), "{barred:?}");
    let listed = fake.counts().lists;

    let again = pull(&fake, &tree, &mut index, None, false, &mut |_| {});

    assert!(matches!(again, Err(Hitch::Barred(_))), "{again:?}");
    assert_eq!(fake.counts().lists, listed, "the cloud was listed again");

    word_up_there(&fake, room.path(), "1.0.0");
    let freed = pull(&fake, &tree, &mut index, None, false, &mut |_| {});

    assert_eq!(freed, Ok(0));
    assert!(index.cursor.is_some());
}

#[test]
fn what_a_later_tisty_took_away_is_not_taken_away_here_while_barred() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(&tree).unwrap();
    let fake = Fake::drive();
    word_up_there(&fake, room.path(), "1.0.0");
    let doc = format!("{PAPERS}/dev_b-0001.md");
    std::fs::write(room.path().join("doc"), b"lo escrito").unwrap();
    fake.put(&doc, &room.path().join("doc"), Expect::Absent)
        .unwrap();
    let mut index = Index::default();
    pull(&fake, &tree, &mut index, None, false, &mut |_| {}).unwrap();
    assert!(tree.join(&doc).is_file());

    word_up_there(&fake, room.path(), "999.0.0");
    fake.delete(&doc, None).unwrap();
    let barred = pull(&fake, &tree, &mut index, None, false, &mut |_| {});

    assert!(matches!(barred, Err(Hitch::Barred(_))), "{barred:?}");
    assert!(tree.join(&doc).is_file(), "a deletion came through the bar");
    assert!(index.tree.contains_key(&doc));
}

#[test]
fn a_word_spoiled_here_is_brought_again_rather_than_barring_for_good() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(&tree).unwrap();
    let fake = Fake::drive();
    word_up_there(&fake, room.path(), "1.0.0");
    let mut index = Index::default();
    pull(&fake, &tree, &mut index, None, false, &mut |_| {}).unwrap();

    std::fs::write(tree.join(NAMED), b"\xff\xfe").unwrap();
    let pulled = pull(&fake, &tree, &mut index, None, false, &mut |_| {});

    assert_eq!(pulled, Ok(0));
    assert_eq!(tisty_sync::refuses(&tree), Ok(()));
}

#[test]
fn a_word_spoiled_here_and_gone_up_there_is_let_go_rather_than_barring_for_good() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(&tree).unwrap();
    let fake = Fake::drive();
    word_up_there(&fake, room.path(), "1.0.0");
    let mut index = Index::default();
    pull(&fake, &tree, &mut index, None, false, &mut |_| {}).unwrap();

    fake.delete(NAMED, None).unwrap();
    std::fs::write(tree.join(NAMED), b"\xff\xfe").unwrap();
    let pulled = pull(&fake, &tree, &mut index, None, true, &mut |_| {});

    assert!(!matches!(pulled, Err(Hitch::Barred(_))), "{pulled:?}");
    assert!(!tree.join(NAMED).exists());
}

#[test]
fn a_plain_reader_stops_at_a_sealed_place_and_a_sealed_one_at_a_plain_one() {
    let tree = tempfile::tempdir().unwrap();
    let plain = Fake::drive();
    let sealed = crate::fakes::sealed_over(Fake::drive());
    let mut heard = Index::default();
    heard.tree.insert(
        NAMED.to_string(),
        Mirrored {
            seen: Seen {
                name: NAMED.into(),
                bytes: 1,
                hash: String::new(),
                revision: "r1".into(),
            },
            len: 1,
            stamp: 0,
            digest: String::new(),
        },
    );
    let word = |said: &str| {
        std::fs::write(
            tree.path().join(NAMED),
            format!("shape = 3\nneeds = \"1.34.0\"\n{said}"),
        )
        .unwrap();
    };
    let code = |remote: &dyn Remote, index: &Index| {
        barred(remote, tree.path(), index).map(|one| one.code())
    };

    word("");
    assert_eq!(
        code(&sealed, &Index::default()),
        None,
        "a word written here and never sent was judged"
    );
    assert_eq!(code(&plain, &heard), None);
    assert_eq!(code(&sealed, &heard), Some("unsealed"));
    word("sealed = \"store_one\"\n");
    assert_eq!(code(&plain, &heard), Some("sealed"));
    assert_eq!(code(&sealed, &heard), None);
    word("sealed = \"store_one\"\nsealing = \"closed:dev_a:n1\"\n");
    assert_eq!(code(&sealed, &heard), Some("sealing"));
}

#[test]
fn a_listing_that_keeps_no_hash_keeps_the_one_this_machine_sent_for_the_same_revision() {
    let mut index = Index::default();
    let seen = |hash: &str, revision: &str| Seen {
        name: "attachments/x-1a2b.pdf".into(),
        bytes: 4,
        hash: hash.into(),
        revision: revision.into(),
    };

    index.shelved("attachments/x-1a2b.pdf", seen("abcd", "r1"));
    index.shelved("attachments/x-1a2b.pdf", seen("", "r1"));
    assert_eq!(index.shelf["attachments/x-1a2b.pdf"].hash, "abcd");
    index.shelved("attachments/x-1a2b.pdf", seen("", "r2"));
    assert_eq!(index.shelf["attachments/x-1a2b.pdf"].hash, "");
}

#[test]
fn a_plain_reader_that_brings_down_a_sealed_word_brings_nothing_else_and_writes_nothing() {
    let room = tempfile::tempdir().unwrap();
    let tree = room.path().join("tree");
    std::fs::create_dir_all(&tree).unwrap();
    let fake = Fake::drive();
    let said = room.path().join("said");
    std::fs::write(
        &said,
        "shape = 3\nneeds = \"1.35.0\"\nsealed = \"store_one\"\n",
    )
    .unwrap();
    fake.put(NAMED, &said, Expect::Absent).unwrap();
    std::fs::write(&said, b"\xffTCS sealed").unwrap();
    fake.put(
        &format!("sealed/{PAPERS}/dev_b-0001.md"),
        &said,
        Expect::Absent,
    )
    .unwrap();
    let mut index = Index::default();

    let pulled = pull(&fake, &tree, &mut index, None, false, &mut |_| {});

    assert!(
        matches!(pulled, Err(Hitch::Barred(tisty_sync::Trouble::Sealed(_)))),
        "{pulled:?}"
    );
    assert!(!tree.join("sealed").exists(), "something sealed came down");
    assert!(index.cursor.is_none(), "the listing was taken as read");
}
