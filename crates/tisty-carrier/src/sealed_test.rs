use super::*;
use crate::fakes::Fake;
use tisty_share_proto::seed::Seed;

fn key(byte: u8) -> CloudKey {
    CloudKey::of(&Seed::from_bytes(&[byte; 32]))
}

fn over_drive() -> Sealed<Fake> {
    Sealed::new(Fake::drive(), key(7), "store_one")
}

fn put(remote: &dyn Remote, name: &str, body: &[u8]) -> Seen {
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path().join("body");
    std::fs::write(&at, body).unwrap();
    remote.put(name, &at, Expect::Any).unwrap()
}

fn fetched(remote: &dyn Remote, name: &str) -> Result<Vec<u8>, Hitch> {
    let mut got = Vec::new();
    remote.fetch(name, 0, &mut got)?;
    Ok(got)
}

fn names(many: Vec<Seen>) -> Vec<String> {
    let mut names: Vec<String> = many.into_iter().map(|one| one.name).collect();
    names.sort();
    names
}

#[test]
fn a_file_comes_back_as_it_went_and_the_provider_holds_only_sealed_bytes() {
    let sealed = over_drive();

    let sent = put(&sealed, "docs/a.md", b"hola");

    assert_eq!(sent.name, "docs/a.md");
    assert_eq!(sent.bytes, 4);
    let plain = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(plain.path(), b"hola").unwrap();
    assert_eq!(sent.hash, sealed.hash_of(plain.path()).unwrap());
    let up_there = fetched(&sealed.inner, "sealed/docs/a.md").unwrap();
    assert!(up_there.starts_with(b"\xffTCS"));
    assert!(!up_there.windows(4).any(|part| part == b"hola"));
    assert_eq!(fetched(&sealed, "docs/a.md").unwrap(), b"hola");
}

#[test]
fn the_three_files_every_build_must_read_stay_plain_and_in_place() {
    let sealed = over_drive();
    for (name, body) in [
        (NAMED, b"shape = 3\n".as_slice()),
        (tisty_core::moved::NAMED, b"{}".as_slice()),
        ("store/.store-id", b"store_one".as_slice()),
    ] {
        put(&sealed, name, body);

        assert_eq!(fetched(&sealed.inner, name).unwrap(), body, "{name}");
        assert_eq!(fetched(&sealed, name).unwrap(), body, "{name}");
    }
}

#[test]
fn only_what_is_sealed_and_the_plain_three_are_seen() {
    let sealed = over_drive();
    put(&sealed.inner, "docs/old.md", b"in the clear");
    put(&sealed.inner, "sealed/sealed.mark", b"mark");
    put(&sealed, "docs/a.md", b"hola");
    put(&sealed, "store/dev_a/1.tisty", b"segment");
    put(&sealed, NAMED, b"shape = 3\n");
    put(&sealed, "store/.store-id", b"store_one");

    let seen = sealed.list("").unwrap();

    assert_eq!(
        names(seen.clone()),
        ["docs/a.md", "store/.store-id", "store/dev_a/1.tisty", NAMED]
    );
    let held = seen.iter().find(|one| one.name == "docs/a.md").unwrap();
    assert_eq!(held.bytes, 4);
    assert!(held.hash.is_empty());
    assert_eq!(names(sealed.list("docs").unwrap()), ["docs/a.md"]);
    assert_eq!(
        names(sealed.list("store").unwrap()),
        ["store/.store-id", "store/dev_a/1.tisty"]
    );
}

#[test]
fn another_phrase_does_not_open_it() {
    let sealed = over_drive();
    put(&sealed, "docs/a.md", b"hola");
    let other = Sealed::new(sealed.inner, key(8), "store_one");

    assert!(matches!(fetched(&other, "docs/a.md"), Err(Hitch::Broke(_))));
}

#[test]
fn a_sealed_file_copied_under_another_name_does_not_open() {
    let sealed = over_drive();
    put(&sealed, "docs/a.md", b"hola");
    let bytes = fetched(&sealed.inner, "sealed/docs/a.md").unwrap();
    put(&sealed.inner, "sealed/docs/b.md", &bytes);

    assert!(matches!(
        fetched(&sealed, "docs/b.md"),
        Err(Hitch::Broke(_))
    ));
    assert_eq!(fetched(&sealed, "docs/a.md").unwrap(), b"hola");
}

#[test]
fn a_file_that_was_never_sealed_inside_the_folder_does_not_pass_as_plain_text() {
    let sealed = over_drive();
    put(&sealed.inner, "sealed/docs/a.md", b"plain text");

    assert!(matches!(
        fetched(&sealed, "docs/a.md"),
        Err(Hitch::Broke(_))
    ));
}

#[test]
fn a_move_seals_the_file_again_under_its_new_name() {
    let sealed = over_drive();
    put(&sealed, "docs/a.md", b"hola");

    sealed.renamed("docs/a.md", "docs/b.md").unwrap();

    assert_eq!(fetched(&sealed, "docs/b.md").unwrap(), b"hola");
    assert!(sealed.about("docs/a.md").unwrap().is_none());
    assert_eq!(names(sealed.list("").unwrap()), ["docs/b.md"]);
}

#[test]
fn a_delete_reaches_the_sealed_copy() {
    let sealed = over_drive();
    put(&sealed, "docs/a.md", b"hola");

    sealed.delete("docs/a.md", None).unwrap();

    assert!(sealed.about("docs/a.md").unwrap().is_none());
    assert!(sealed.list("").unwrap().is_empty());
}

#[test]
fn about_and_changes_speak_of_the_name_and_size_the_file_has_in_the_clear() {
    let sealed = over_drive();
    put(&sealed, "docs/a.md", b"hola");

    let seen = sealed.about("docs/a.md").unwrap().unwrap();
    assert_eq!((seen.name.as_str(), seen.bytes), ("docs/a.md", 4));
    let Changes::Whole { seen, cursor } = sealed.changes(None).unwrap() else {
        panic!("the first look is the whole listing");
    };
    assert_eq!(names(seen), ["docs/a.md"]);
    put(&sealed, "docs/b.md", b"adios");
    let Changes::Since { changed, .. } = sealed.changes(Some(&cursor)).unwrap() else {
        panic!("asking from a cursor brings only what moved");
    };
    assert_eq!(names(changed), ["docs/b.md"]);
}

#[test]
fn a_sealed_file_can_be_fetched_from_the_middle_and_is_never_lent() {
    let sealed = over_drive();
    put(&sealed, "docs/a.md", b"hola");

    let mut tail = Vec::new();
    sealed.fetch("docs/a.md", 2, &mut tail).unwrap();

    assert_eq!(tail, b"la");
    assert_eq!(sealed.lends("docs/a.md").unwrap(), None);
}

#[test]
fn a_provider_ceiling_leaves_room_for_the_seal() {
    let sealed = over_drive();

    let (inner, ours) = (sealed.inner.limits(), sealed.limits());

    assert!(ours.most_per_file < inner.most_per_file);
    assert_eq!(ours.chunk, inner.chunk);
}

#[test]
fn the_size_in_the_clear_is_read_back_from_the_sealed_size() {
    let key = key(7);
    for plain in [0usize, 1, 100, 65_535, 65_536, 65_537, 131_072, 200_000] {
        let body = vec![5u8; plain];
        let sealed =
            tisty_share_proto::cloud::sealed(&key, "store_one", "docs/a.md", &body).unwrap();

        assert_eq!(
            plain_len(sealed.len() as u64),
            Some(plain as u64),
            "{plain}"
        );
        let plain = plain as u64;
        assert_eq!(
            sealed.len() as u64,
            HEADER + plain + TAG * plain.div_ceil(BLOCK).max(1),
            "{plain}"
        );
        assert_eq!(plain_room(sealed.len() as u64), plain, "{plain}");
    }
    assert_eq!(plain_len(23), None);
    assert_eq!(plain_len(HEADER + 5), None);
}

#[test]
fn the_name_of_the_phrase_mark_cannot_be_written_or_deleted_as_a_file() {
    let sealed = over_drive();
    put(&sealed.inner, "sealed/sealed.mark", b"mark");
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path().join("body");
    std::fs::write(&at, b"x").unwrap();

    assert!(sealed.put("sealed.mark", &at, Expect::Any).is_err());
    assert!(sealed.delete("sealed.mark", None).is_err());
    assert!(sealed.inner.about("sealed/sealed.mark").unwrap().is_some());
}

#[test]
fn a_missing_file_is_named_as_it_was_asked_for() {
    let sealed = over_drive();

    assert_eq!(
        fetched(&sealed, "docs/zz.md").unwrap_err(),
        Hitch::Missing("docs/zz.md".into())
    );
}

#[test]
fn a_file_that_is_not_validly_sealed_keeps_the_size_it_has_up_there() {
    let sealed = over_drive();
    put(&sealed.inner, "sealed/docs/a.md", b"plain");

    let seen = sealed.about("docs/a.md").unwrap().unwrap();

    assert_eq!(seen.bytes, 5);
}

#[test]
fn the_phrase_mark_is_neither_told_about_nor_fetched_as_a_file() {
    let sealed = over_drive();
    put(&sealed.inner, "sealed/sealed.mark", b"mark");

    assert!(sealed.about("sealed.mark").unwrap().is_none());
    assert!(matches!(
        fetched(&sealed, "sealed.mark"),
        Err(Hitch::Missing(_))
    ));
}

#[test]
fn a_provider_that_folds_case_cannot_be_made_to_overwrite_the_mark() {
    let sealed = Sealed::new(Fake::onedrive(), key(7), "store_one");
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path().join("body");
    std::fs::write(&at, b"x").unwrap();

    assert!(sealed.put("Sealed.Mark", &at, Expect::Any).is_err());
    assert!(sealed.delete("SEALED.MARK", None).is_err());
}

#[test]
fn a_cloud_with_no_mark_says_none_and_a_mark_once_written_holds_for_its_phrase() {
    let sealed = over_drive();
    assert_eq!(sealed.marked().unwrap(), None);

    sealed.mark_it().unwrap();

    assert_eq!(sealed.marked().unwrap(), Some(Mark::Holds));
    assert!(sealed.list("").unwrap().is_empty());
}

#[test]
fn another_phrase_is_told_from_this_one_by_the_mark() {
    let sealed = over_drive();
    sealed.mark_it().unwrap();
    let other = Sealed::new(sealed.inner, key(8), "store_one");

    assert_eq!(other.marked().unwrap(), Some(Mark::OtherPhrase));
}

#[test]
fn marking_twice_with_one_phrase_is_the_same_as_once_and_another_phrase_cannot_mark_over_it() {
    let sealed = over_drive();
    sealed.mark_it().unwrap();

    sealed.mark_it().unwrap();
    assert_eq!(sealed.marked().unwrap(), Some(Mark::Holds));
    let other = Sealed::new(sealed.inner, key(8), "store_one");

    assert!(other.mark_it().is_err());
    assert_eq!(other.marked().unwrap(), Some(Mark::OtherPhrase));
}

#[test]
fn the_root_names_the_cloud_so_two_sealed_clouds_never_share_a_mirror() {
    let one = Sealed::new(Fake::drive(), key(7), "store_one");
    let two = Sealed::new(Fake::drive(), key(7), "store_two");

    assert_ne!(one.root(), two.root());
    assert!(!one.root().is_empty());
}

#[test]
fn a_name_with_capitals_comes_back_on_a_provider_that_folds_case() {
    let sealed = Sealed::new(Fake::onedrive(), key(7), "store_one");
    put(&sealed, "docs/Plan.md", b"hola");

    assert_eq!(fetched(&sealed, "docs/Plan.md").unwrap(), b"hola");
    sealed.renamed("docs/Plan.md", "docs/Otro.md").unwrap();
    assert_eq!(fetched(&sealed, "docs/Otro.md").unwrap(), b"hola");
}

#[test]
fn a_plain_name_planted_inside_the_sealed_folder_is_not_seen_twice() {
    let sealed = over_drive();
    put(&sealed, NAMED, b"shape = 3\n");
    put(&sealed.inner, &format!("sealed/{NAMED}"), b"planted");

    assert_eq!(names(sealed.list("").unwrap()), [NAMED]);
}

#[test]
fn renaming_only_the_capitals_on_a_provider_that_folds_case_is_refused_and_keeps_the_file() {
    let sealed = Sealed::new(Fake::onedrive(), key(7), "store_one");
    put(&sealed, "docs/Plan.md", b"hola");

    assert!(sealed.renamed("docs/Plan.md", "docs/plan.md").is_err());
    assert!(sealed.renamed("docs/Plan.md", "docs/Plan.md").is_ok());
    assert_eq!(fetched(&sealed, "docs/Plan.md").unwrap(), b"hola");
}
