use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::fakes::{self, Counting, Fake};
use super::*;

fn local(room: &Path, body: &[u8]) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let at = room.join(format!("local-{}", NEXT.fetch_add(1, Ordering::Relaxed)));
    std::fs::write(&at, body).unwrap();
    at
}

fn put(
    remote: &dyn Remote,
    room: &Path,
    name: &str,
    body: &[u8],
    expect: Expect,
) -> Result<Seen, Hitch> {
    remote.put(name, &local(room, body), expect)
}

fn body_of(remote: &dyn Remote, name: &str, from: u64) -> Result<Vec<u8>, Hitch> {
    let mut got = Vec::new();
    remote.fetch(name, from, &mut got)?;
    Ok(got)
}

fn names(mut seen: Vec<Seen>) -> Vec<String> {
    seen.sort_by(|one, other| one.name.cmp(&other.name));
    seen.into_iter().map(|one| one.name).collect()
}

fn on_each(check: impl Fn(&dyn Counting, &Path)) {
    for fake in fakes::every() {
        let room = tempfile::tempdir().unwrap();
        check(fake.as_ref(), room.path());
    }
}

fn known_after(known: &mut BTreeMap<String, Seen>, told: Changes) -> String {
    match told {
        Changes::Whole { seen, cursor } => {
            *known = seen
                .into_iter()
                .map(|one| (one.name.clone(), one))
                .collect();
            cursor
        }
        Changes::Since {
            changed,
            gone,
            cursor,
        } => {
            for one in changed {
                known.insert(one.name.clone(), one);
            }
            for name in gone {
                known.remove(&name);
            }
            cursor
        }
    }
}

fn listing(remote: &dyn Remote) -> BTreeMap<String, Seen> {
    remote
        .list("")
        .unwrap()
        .into_iter()
        .map(|one| (one.name.clone(), one))
        .collect()
}

#[test]
fn a_name_is_one_file_however_often_it_is_written() {
    on_each(|remote, room| {
        let who = remote.who();
        let first = put(remote, room, "docs/a.md", b"uno", Expect::Absent).unwrap();
        let second = put(remote, room, "docs/a.md", b"dos", Expect::Any).unwrap();

        assert_eq!(names(remote.list("").unwrap()), ["docs/a.md"], "{who}");
        assert_eq!(body_of(remote, "docs/a.md", 0).unwrap(), b"dos", "{who}");
        assert_ne!(
            first.revision, second.revision,
            "{who}: a write is a new revision"
        );
        assert_eq!(second.bytes, 3, "{who}");
    });
}

#[test]
fn creating_what_is_already_there_is_refused_and_leaves_it_whole() {
    on_each(|remote, room| {
        put(remote, room, "x", b"primero", Expect::Absent).unwrap();

        let again = put(remote, room, "x", b"segundo", Expect::Absent);

        assert_eq!(again, Err(Hitch::Changed("x".into())), "{}", remote.who());
        assert_eq!(
            body_of(remote, "x", 0).unwrap(),
            b"primero",
            "{}",
            remote.who()
        );
    });
}

#[test]
fn replacing_takes_the_revision_the_caller_saw_and_nothing_older() {
    on_each(|remote, room| {
        let who = remote.who();
        let seen = put(remote, room, "x", b"uno", Expect::Absent).unwrap();
        let next = put(
            remote,
            room,
            "x",
            b"dos",
            Expect::Revision(seen.revision.clone()),
        )
        .unwrap();

        let stale = put(remote, room, "x", b"tres", Expect::Revision(seen.revision));

        assert_eq!(stale, Err(Hitch::Changed("x".into())), "{who}");
        assert_eq!(body_of(remote, "x", 0).unwrap(), b"dos", "{who}");
        let current = put(
            remote,
            room,
            "x",
            b"cuatro",
            Expect::Revision(next.revision),
        );
        assert!(
            current.is_ok(),
            "{who}: the revision just given is the one to name"
        );
    });
}

#[test]
fn what_another_deleted_is_missing_not_changed() {
    on_each(|remote, room| {
        let seen = put(remote, room, "x", b"uno", Expect::Absent).unwrap();
        remote.delete("x", None).unwrap();

        let after = put(remote, room, "x", b"dos", Expect::Revision(seen.revision));

        assert_eq!(after, Err(Hitch::Missing("x".into())), "{}", remote.who());
        assert!(remote.list("").unwrap().is_empty(), "{}", remote.who());
    });
}

#[test]
fn anything_creates_and_replaces() {
    on_each(|remote, room| {
        put(remote, room, "x", b"uno", Expect::Any).unwrap();
        put(remote, room, "x", b"dos", Expect::Any).unwrap();

        assert_eq!(body_of(remote, "x", 0).unwrap(), b"dos", "{}", remote.who());
    });
}

#[test]
fn fetching_gives_the_whole_or_the_tail_and_says_what_it_fetched() {
    on_each(|remote, room| {
        let who = remote.who();
        let landed = put(remote, room, "x", b"abcdef", Expect::Absent).unwrap();

        let mut tail = Vec::new();
        let seen = remote.fetch("x", 4, &mut tail).unwrap();

        assert_eq!(tail, b"ef", "{who}");
        assert_eq!(seen, landed, "{who}");
        assert_eq!(
            body_of(remote, "x", 6).unwrap(),
            b"",
            "{who}: nothing is left past the end"
        );
        assert!(
            matches!(body_of(remote, "x", 7), Err(Hitch::Broke(_))),
            "{who}: asking past the end is a mistake"
        );
        assert_eq!(
            body_of(remote, "nada", 0),
            Err(Hitch::Missing("nada".into())),
            "{who}"
        );
    });
}

#[test]
fn deleting_by_revision_only_deletes_what_the_caller_saw() {
    on_each(|remote, room| {
        let who = remote.who();
        let seen = put(remote, room, "x", b"uno", Expect::Absent).unwrap();

        assert_eq!(
            remote.delete("x", Some("r0")),
            Err(Hitch::Changed("x".into())),
            "{who}"
        );
        assert_eq!(names(remote.list("").unwrap()), ["x"], "{who}");
        assert_eq!(remote.delete("x", Some(&seen.revision)), Ok(()), "{who}");
        assert_eq!(
            remote.delete("x", None),
            Err(Hitch::Missing("x".into())),
            "{who}"
        );
        put(remote, room, "y", b"dos", Expect::Absent).unwrap();
        assert_eq!(remote.delete("y", None), Ok(()), "{who}");
        assert!(remote.list("").unwrap().is_empty(), "{who}");
    });
}

#[test]
fn listing_stays_under_the_prefix_asked_and_goes_all_the_way_down() {
    on_each(|remote, room| {
        let who = remote.who();
        for name in ["a/x", "ab/y", "a/b/z", "top"] {
            put(remote, room, name, b"1", Expect::Absent).unwrap();
        }

        assert_eq!(names(remote.list("a").unwrap()), ["a/b/z", "a/x"], "{who}");
        assert_eq!(names(remote.list("a/").unwrap()), ["a/b/z", "a/x"], "{who}");
        assert_eq!(names(remote.list("").unwrap()).len(), 4, "{who}");
        assert!(remote.list("nada").unwrap().is_empty(), "{who}");
    });
}

#[test]
fn a_name_with_spaces_and_accents_comes_back_as_written() {
    on_each(|remote, room| {
        let name = "attachments/ab/fotó de playa-1a2b3c4d.png";
        put(remote, room, name, b"imagen", Expect::Absent).unwrap();

        assert_eq!(
            names(remote.list("attachments").unwrap()),
            [name],
            "{}",
            remote.who()
        );
        assert_eq!(
            body_of(remote, name, 0).unwrap(),
            b"imagen",
            "{}",
            remote.who()
        );
    });
}

#[test]
fn what_is_not_a_name_is_refused_and_writes_nothing() {
    on_each(|remote, room| {
        for bad in ["", "/x", "x/", "a//b", "../x", "a/./b", "a\\b", "a/\u{7}"] {
            let refused = put(remote, room, bad, b"1", Expect::Any);

            assert!(
                matches!(refused, Err(Hitch::Broke(_))),
                "{}: {bad:?} is not a name",
                remote.who()
            );
        }
        assert!(remote.list("").unwrap().is_empty(), "{}", remote.who());
    });
}

#[test]
fn the_hash_follows_the_content_and_is_the_one_the_provider_reports() {
    on_each(|remote, room| {
        let who = remote.who();
        let one = local(room, b"mismo");
        let same = local(room, b"mismo");
        let other = local(room, b"otro");

        assert_eq!(
            remote.hash_of(&one).unwrap(),
            remote.hash_of(&same).unwrap(),
            "{who}"
        );
        assert_ne!(
            remote.hash_of(&one).unwrap(),
            remote.hash_of(&other).unwrap(),
            "{who}"
        );
        let landed = remote.put("x", &one, Expect::Absent).unwrap();
        assert_eq!(
            landed.hash,
            remote.hash_of(&one).unwrap(),
            "{who}: a landing is checked without downloading"
        );
        assert_eq!(listing(remote)["x"].hash, landed.hash, "{who}");
        assert!(remote.hash_of(&room.join("no-such-file")).is_err(), "{who}");
    });
}

#[test]
fn two_names_that_differ_only_in_case_are_never_merged_in_silence() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "docs/Nota", b"primera", Expect::Absent).unwrap();

        let second = put(remote, room, "docs/nota", b"segunda", Expect::Absent);

        match remote.folds_case() {
            true => {
                assert_eq!(second, Err(Hitch::Changed("docs/nota".into())), "{who}");
                assert_eq!(names(remote.list("").unwrap()), ["docs/Nota"], "{who}");
                assert_eq!(
                    body_of(remote, "docs/Nota", 0).unwrap(),
                    b"primera",
                    "{who}"
                );
                let found = remote.about("docs/NOTA").unwrap().unwrap();
                assert_eq!(
                    found.name, "docs/Nota",
                    "{who}: it answers as it was first written"
                );
            }
            false => {
                assert!(second.is_ok(), "{who}");
                assert_eq!(
                    names(remote.list("").unwrap()),
                    ["docs/Nota", "docs/nota"],
                    "{who}"
                );
                assert_eq!(
                    body_of(remote, "docs/Nota", 0).unwrap(),
                    b"primera",
                    "{who}"
                );
                assert_eq!(remote.about("docs/NOTA").unwrap(), None, "{who}");
            }
        }
    });
}

#[test]
fn asking_about_a_name_answers_what_listing_would() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "a/x", b"1", Expect::Absent).unwrap();

        assert_eq!(
            remote.about("a/x").unwrap(),
            Some(listing(remote)["a/x"].clone()),
            "{who}"
        );
        assert_eq!(remote.about("a/y").unwrap(), None, "{who}");
        assert_eq!(remote.about("b").unwrap(), None, "{who}");
    });
}

#[test]
fn changes_always_leave_the_caller_knowing_what_the_remote_holds() {
    on_each(|remote, room| {
        let who = remote.who();
        let mut known = BTreeMap::new();
        put(remote, room, "keep", b"1", Expect::Absent).unwrap();
        put(remote, room, "edit", b"1", Expect::Absent).unwrap();
        put(remote, room, "drop", b"1", Expect::Absent).unwrap();
        let cursor = known_after(&mut known, remote.changes(None).unwrap());
        assert_eq!(known, listing(remote), "{who}: the first look is the whole");

        put(remote, room, "new", b"1", Expect::Absent).unwrap();
        put(remote, room, "edit", b"22", Expect::Any).unwrap();
        remote.delete("drop", None).unwrap();
        let told = remote.changes(Some(&cursor)).unwrap();
        match (&told, remote.native_changes()) {
            (Changes::Since { changed, gone, .. }, true) => {
                assert_eq!(names(changed.clone()), ["edit", "new"], "{who}");
                assert_eq!(gone, &["drop".to_string()], "{who}");
            }
            (Changes::Whole { .. }, false) => {}
            _ => panic!("{who}: the shape of the answer says what the provider has"),
        }
        let next = known_after(&mut known, told);
        assert_eq!(known, listing(remote), "{who}");

        let quiet = remote.changes(Some(&next)).unwrap();
        known_after(&mut known, quiet);
        assert_eq!(known, listing(remote), "{who}");
    });
}

#[test]
fn a_quiet_round_asks_once_and_moves_no_content() {
    on_each(|remote, room| {
        let who = remote.who();
        for number in 0..70 {
            put(
                remote,
                room,
                &format!("store/dev_a/{number:04}.jsonl"),
                b"{}",
                Expect::Absent,
            )
            .unwrap();
        }
        let Changes::Whole { cursor, .. } = remote.changes(None).unwrap() else {
            panic!("{who}: the first look is the whole");
        };
        remote.forget_counts();

        let told = remote.changes(Some(&cursor)).unwrap();

        let spent = remote.counts();
        assert_eq!(spent.requests, 1, "{who}");
        assert_eq!((spent.sent, spent.received), (0, 0), "{who}");
        if let Changes::Since { changed, gone, .. } = told {
            assert!(changed.is_empty() && gone.is_empty(), "{who}");
        }
    });
}

#[test]
fn appending_leaves_the_whole_content_and_sends_only_the_tail_where_it_can() {
    on_each(|remote, room| {
        let who = remote.who();
        let base = vec![b'a'; 1000];
        let seen = put(
            remote,
            room,
            "store/dev_a/0001.jsonl",
            &base,
            Expect::Absent,
        )
        .unwrap();
        let grown = [base.as_slice(), &[b'b'; 40]].concat();
        remote.forget_counts();

        let after = remote
            .append(
                "store/dev_a/0001.jsonl",
                &local(room, &grown),
                1000,
                Expect::Revision(seen.revision.clone()),
            )
            .unwrap();

        assert_eq!(
            body_of(remote, "store/dev_a/0001.jsonl", 0).unwrap(),
            grown,
            "{who}"
        );
        assert_eq!(after.bytes, 1040, "{who}");
        let sent = remote.counts().sent;
        assert_eq!(
            sent,
            if remote.native_append() { 40 } else { 1040 },
            "{who}"
        );
        let stale = remote.append(
            "store/dev_a/0001.jsonl",
            &local(room, &grown),
            1040,
            Expect::Revision(seen.revision),
        );
        assert_eq!(
            stale,
            Err(Hitch::Changed("store/dev_a/0001.jsonl".into())),
            "{who}"
        );
    });
}

#[test]
fn every_provider_takes_the_biggest_attachment_tisty_allows() {
    on_each(|remote, _| {
        let limits = remote.limits();

        assert!(
            limits.most_per_file >= tisty_core::attach::COPIED_IN_DOC,
            "{}",
            remote.who()
        );
        assert!(
            limits.chunk > 0 && limits.units_a_day > 0 && limits.bytes_a_day > 0,
            "{}",
            remote.who()
        );
        let costs = limits.costs;
        assert!(
            [
                costs.list,
                costs.fetch,
                costs.put,
                costs.delete,
                costs.changes
            ]
            .iter()
            .all(|cost| *cost > 0),
            "{}: nothing is free, or it would never be counted",
            remote.who()
        );
    });
}

#[test]
fn an_upload_costs_a_request_per_chunk_and_a_refused_one_sends_nothing() {
    on_each(|remote, room| {
        let who = remote.who();
        let limits = remote.limits();
        let body = vec![0u8; usize::try_from(limits.chunk * 2 + 1).unwrap()];
        let big = local(room, &body);
        remote.forget_counts();

        remote.put("big", &big, Expect::Absent).unwrap();
        let refused = remote.put("big", &big, Expect::Absent);

        let spent = remote.counts();
        assert_eq!(refused, Err(Hitch::Changed("big".into())), "{who}");
        assert_eq!(spent.requests, 3 + 1, "{who}");
        assert_eq!(spent.sent, body.len() as u64, "{who}");
        assert_eq!(spent.units, 4 * limits.costs.put, "{who}");
    });
}

#[test]
fn the_provider_that_holds_two_of_a_name_shows_one_the_same_one_every_time() {
    let drive = Fake::drive();
    let room = tempfile::tempdir().unwrap();
    let first = put(
        &drive,
        room.path(),
        "docs/a.md",
        b"la primera",
        Expect::Absent,
    )
    .unwrap();
    drive.plant("docs/a.md", b"la otra");

    assert_eq!(drive.copies_of("docs/a.md"), 2);
    for _ in 0..3 {
        assert_eq!(drive.list("").unwrap(), std::slice::from_ref(&first));
        assert_eq!(drive.about("docs/a.md").unwrap(), Some(first.clone()));
        assert_eq!(body_of(&drive, "docs/a.md", 0).unwrap(), b"la primera");
    }
    assert_eq!(
        put(&drive, room.path(), "docs/a.md", b"y otra", Expect::Absent),
        Err(Hitch::Changed("docs/a.md".into()))
    );
}

#[test]
fn only_the_provider_that_can_be_told_hears_and_it_hears_once_per_change() {
    let room = tempfile::tempdir().unwrap();
    for fake in fakes::every() {
        let hears = fake.hears();
        assert_eq!(hears.is_some(), fake.who() == "dropbox", "{}", fake.who());
        let Some(mut hears) = hears else { continue };
        let quiet = std::time::Duration::from_millis(1);

        assert!(!hears.told(quiet).unwrap());
        put(fake.as_ref(), room.path(), "x", b"1", Expect::Absent).unwrap();
        assert!(hears.told(quiet).unwrap());
        assert!(!hears.told(quiet).unwrap());
    }
}

#[test]
fn a_link_is_lent_for_what_exists_and_only_where_the_provider_lends() {
    let room = tempfile::tempdir().unwrap();
    for fake in fakes::every() {
        put(fake.as_ref(), room.path(), "x", b"1", Expect::Absent).unwrap();

        let lent = fake.lends("x").unwrap();

        assert_eq!(lent.is_some(), fake.who() != "bare", "{}", fake.who());
        assert_eq!(fake.lends("nada").unwrap(), None, "{}", fake.who());
    }
}
