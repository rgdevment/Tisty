use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

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

fn across(fakes: Vec<Box<dyn Counting>>, check: impl Fn(&dyn Counting, &Path)) {
    for fake in fakes {
        let room = tempfile::tempdir().unwrap();
        check(fake.as_ref(), room.path());
    }
}

fn on_each(check: impl Fn(&dyn Counting, &Path)) {
    across(fakes::every(), check);
}

fn cursor_of(told: Changes) -> String {
    match told {
        Changes::Whole { cursor, .. } | Changes::Since { cursor, .. } => cursor,
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
        let rev = Expect::Revision(seen.revision.clone());
        let next = put(remote, room, "x", b"dos", rev).unwrap();

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
fn a_revision_is_never_given_twice_even_for_a_name_deleted_and_made_again() {
    on_each(|remote, room| {
        let who = remote.who();
        let first = put(remote, room, "x", b"uno", Expect::Absent).unwrap();
        remote.delete("x", None).unwrap();
        let again = put(remote, room, "x", b"dos", Expect::Absent).unwrap();

        assert_ne!(first.revision, again.revision, "{who}");
        let old = put(
            remote,
            room,
            "x",
            b"tres",
            Expect::Revision(first.revision.clone()),
        );
        assert_eq!(old, Err(Hitch::Changed("x".into())), "{who}");
        assert_eq!(
            remote.delete("x", Some(&first.revision)),
            Err(Hitch::Changed("x".into())),
            "{who}"
        );
        assert_eq!(body_of(remote, "x", 0).unwrap(), b"dos", "{who}");
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
fn an_empty_file_is_a_file() {
    on_each(|remote, room| {
        let who = remote.who();

        let seen = put(remote, room, "vacio", b"", Expect::Absent).unwrap();

        assert_eq!(seen.bytes, 0, "{who}");
        assert_eq!(body_of(remote, "vacio", 0).unwrap(), b"", "{who}");
        assert_eq!(
            remote.hash_of(&local(room, b"")).unwrap(),
            seen.hash,
            "{who}"
        );
        assert_eq!(names(remote.list("").unwrap()), ["vacio"], "{who}");
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
fn a_long_listing_costs_a_request_per_page_and_misses_nothing() {
    across(
        fakes::every_with(|fake| fake.with_page(10)),
        |remote, room| {
            let who = remote.who();
            let wanted: Vec<String> = (0..25).map(|number| format!("f/{number:02}")).collect();
            for name in &wanted {
                put(remote, room, name, b"1", Expect::Absent).unwrap();
            }
            remote.forget_counts();

            let listed = names(remote.list("").unwrap());

            let spent = remote.counts();
            assert_eq!(listed, wanted, "{who}");
            assert_eq!(spent.requests, 3, "{who}");
            assert_eq!(spent.units, 3 * remote.limits().costs.list, "{who}");
        },
    );
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
fn what_is_not_a_name_is_refused_by_every_verb_and_writes_nothing() {
    on_each(|remote, room| {
        let who = remote.who();
        for bad in ["", "/x", "x/", "a//b", "../x", "a/./b", "a\\b", "a/\u{7}"] {
            let refused = |result: Option<Hitch>| {
                assert!(
                    matches!(result, Some(Hitch::Broke(_))),
                    "{who}: {bad:?} is not a name"
                );
            };
            refused(put(remote, room, bad, b"1", Expect::Any).err());
            refused(body_of(remote, bad, 0).err());
            refused(remote.delete(bad, None).err());
            refused(remote.about(bad).err());
        }
        assert!(remote.list("").unwrap().is_empty(), "{who}");
    });
}

#[test]
fn the_hash_follows_the_content_and_is_the_one_the_provider_reports() {
    on_each(|remote, room| {
        let who = remote.who();
        let one = local(room, b"mismo");
        let same = local(room, b"mismo");
        let other = local(room, b"otro");

        let hashed = remote.hash_of(&one).unwrap();

        assert_eq!(hashed, remote.hash_of(&same).unwrap(), "{who}");
        assert_ne!(hashed, remote.hash_of(&other).unwrap(), "{who}");
        let landed = remote.put("x", &one, Expect::Absent).unwrap();
        assert_eq!(landed.hash, hashed, "{who}: a landing is checked unread");
        assert_eq!(listing(remote)["x"].hash, landed.hash, "{who}");
        assert!(remote.hash_of(&room.join("no-such-file")).is_err(), "{who}");
    });
}

#[test]
fn a_second_spelling_is_refused_where_the_provider_folds_case_and_kept_apart_where_not() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "docs/Nota", b"primera", Expect::Absent).unwrap();

        let second = put(remote, room, "docs/nota", b"segunda", Expect::Absent);

        match remote.limits().folds_case {
            true => {
                assert_eq!(second, Err(Hitch::Changed("docs/nota".into())), "{who}");
                assert_eq!(names(remote.list("").unwrap()), ["docs/Nota"], "{who}");
                assert_eq!(
                    body_of(remote, "docs/Nota", 0).unwrap(),
                    b"primera",
                    "{who}"
                );
                let found = remote.about("docs/NOTA").unwrap().unwrap();
                assert_eq!(found.name, "docs/Nota", "{who}: as first written");
            }
            false => {
                assert!(second.is_ok(), "{who}");
                let both = ["docs/Nota", "docs/nota"];
                assert_eq!(names(remote.list("").unwrap()), both, "{who}");
                assert_eq!(remote.about("docs/NOTA").unwrap(), None, "{who}");
            }
        }
    });
}

#[test]
fn where_the_provider_folds_case_every_spelling_reaches_the_one_file() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "docs/Nota", b"primera", Expect::Absent).unwrap();

        let over = put(remote, room, "docs/NOTA", b"segunda", Expect::Any).unwrap();

        match remote.limits().folds_case {
            true => {
                assert_eq!(over.name, "docs/Nota", "{who}: the spelling it holds");
                assert_eq!(names(remote.list("").unwrap()), ["docs/Nota"], "{who}");
                assert_eq!(
                    body_of(remote, "docs/nota", 0).unwrap(),
                    b"segunda",
                    "{who}"
                );
                remote.delete("DOCS/nota", None).unwrap();
                assert!(remote.list("").unwrap().is_empty(), "{who}");
            }
            false => {
                assert_eq!(over.name, "docs/NOTA", "{who}");
                assert_eq!(
                    body_of(remote, "docs/Nota", 0).unwrap(),
                    b"primera",
                    "{who}"
                );
                remote.delete("docs/NOTA", None).unwrap();
                assert_eq!(names(remote.list("").unwrap()), ["docs/Nota"], "{who}");
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
        for name in ["keep", "edit", "drop"] {
            put(remote, room, name, b"1", Expect::Absent).unwrap();
        }
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

        known_after(&mut known, remote.changes(Some(&next)).unwrap());
        assert_eq!(known, listing(remote), "{who}");
    });
}

#[test]
fn changes_follow_a_name_deleted_and_made_again_under_any_spelling() {
    on_each(|remote, room| {
        let who = remote.who();
        let mut known = BTreeMap::new();
        put(remote, room, "docs/Nota", b"1", Expect::Absent).unwrap();
        let mut cursor = known_after(&mut known, remote.changes(None).unwrap());

        remote.delete("docs/Nota", None).unwrap();
        put(remote, room, "docs/nota", b"2", Expect::Absent).unwrap();
        cursor = known_after(&mut known, remote.changes(Some(&cursor)).unwrap());
        assert_eq!(known, listing(remote), "{who}: another spelling");

        remote.delete("docs/nota", None).unwrap();
        put(remote, room, "docs/nota", b"3", Expect::Absent).unwrap();
        cursor = known_after(&mut known, remote.changes(Some(&cursor)).unwrap());
        assert_eq!(known, listing(remote), "{who}: the same spelling");

        put(remote, room, "de-paso", b"4", Expect::Absent).unwrap();
        remote.delete("de-paso", None).unwrap();
        known_after(&mut known, remote.changes(Some(&cursor)).unwrap());
        assert_eq!(known, listing(remote), "{who}: came and went between looks");
    });
}

#[test]
fn a_quiet_round_asks_once_where_changes_are_kept_and_moves_no_content() {
    on_each(|remote, room| {
        let who = remote.who();
        for number in 0..70 {
            let name = format!("store/dev_a/{number:04}.jsonl");
            put(remote, room, &name, b"{}", Expect::Absent).unwrap();
        }
        let cursor = cursor_of(remote.changes(None).unwrap());
        remote.forget_counts();

        let told = remote.changes(Some(&cursor)).unwrap();

        let spent = remote.counts();
        assert_eq!((spent.sent, spent.received), (0, 0), "{who}");
        match (told, remote.native_changes()) {
            (Changes::Since { changed, gone, .. }, true) => {
                assert!(changed.is_empty() && gone.is_empty(), "{who}");
                assert_eq!(spent.requests, 1, "{who}");
            }
            (Changes::Whole { seen, .. }, false) => {
                assert_eq!(seen.len(), 70, "{who}");
                assert_eq!(
                    spent.requests,
                    remote.limits().requests_to_list(70),
                    "{who}"
                );
            }
            _ => panic!("{who}: the shape of the answer says what the provider has"),
        }
    });
}

#[test]
fn a_cursor_the_provider_does_not_know_gets_the_whole_listing_and_a_new_cursor() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "a", b"1", Expect::Absent).unwrap();
        let old = cursor_of(remote.changes(None).unwrap());
        put(remote, room, "b", b"1", Expect::Absent).unwrap();
        remote.forget_feed();

        for unknown in [old.as_str(), "not a cursor", "999999"] {
            let mut known = BTreeMap::new();
            let told = remote.changes(Some(unknown)).unwrap();
            assert!(matches!(told, Changes::Whole { .. }), "{who}: {unknown:?}");
            known_after(&mut known, told);
            assert_eq!(known, listing(remote), "{who}: {unknown:?}");
        }
        let mut known = BTreeMap::new();
        let fresh = known_after(&mut known, remote.changes(Some(&old)).unwrap());
        put(remote, room, "c", b"1", Expect::Absent).unwrap();
        known_after(&mut known, remote.changes(Some(&fresh)).unwrap());
        assert_eq!(known, listing(remote), "{who}: the new cursor works");
    });
}

#[test]
fn a_read_costs_what_the_limits_say_and_moves_the_bytes_it_moves() {
    on_each(|remote, room| {
        let who = remote.who();
        let costs = remote.limits().costs;
        put(remote, room, "x", b"hola", Expect::Absent).unwrap();

        let spent = || {
            let now = remote.counts();
            remote.forget_counts();
            now
        };
        remote.forget_counts();
        remote.list("").unwrap();
        assert_eq!(spent().units, costs.list, "{who}: list");
        body_of(remote, "x", 0).unwrap();
        let fetched = spent();
        assert_eq!(
            (fetched.units, fetched.received),
            (costs.fetch, 4),
            "{who}: fetch"
        );
        remote.about("x").unwrap();
        let asked = spent();
        let about = if remote.native_changes() {
            costs.about
        } else {
            costs.list
        };
        assert_eq!((asked.units, asked.received), (about, 0), "{who}: about");
        remote.changes(None).unwrap();
        let looked = spent();
        let changes = if remote.native_changes() {
            costs.changes
        } else {
            costs.list
        };
        assert_eq!((looked.units, looked.sent), (changes, 0), "{who}: changes");
    });
}

#[test]
fn a_write_costs_what_the_limits_say_and_a_refused_one_sends_nothing() {
    across(
        fakes::every_with(|fake| fake.with_chunk(1000)),
        |remote, room| {
            let who = remote.who();
            let put_cost = remote.limits().costs.put;
            let small = local(room, &[0u8; 500]);
            let big = local(room, &[0u8; 2500]);
            remote.forget_counts();

            remote.put("small", &small, Expect::Absent).unwrap();
            assert_eq!(remote.counts().requests, 1, "{who}: one chunk");
            remote.put("big", &big, Expect::Absent).unwrap();
            assert_eq!(
                remote.counts().requests,
                1 + 4,
                "{who}: a session and three chunks"
            );
            let refused = remote.put("big", &big, Expect::Absent);

            let spent = remote.counts();
            assert_eq!(refused, Err(Hitch::Changed("big".into())), "{who}");
            assert_eq!(spent.requests, 1 + 4 + 1, "{who}");
            assert_eq!(spent.sent, 3000, "{who}");
            assert_eq!(spent.units, 6 * put_cost, "{who}");
            remote.forget_counts();
            remote.delete("big", None).unwrap();
            assert_eq!(remote.counts().units, remote.limits().costs.delete, "{who}");
        },
    );
}

#[test]
fn what_every_provider_declares_has_to_add_up() {
    on_each(|remote, _| {
        let who = remote.who();
        let limits = remote.limits();
        let costs = limits.costs;

        assert!(
            limits.most_per_file >= tisty_core::attach::COPIED_IN_DOC,
            "{who}"
        );
        assert!(
            limits.bytes_a_day >= tisty_core::attach::COPIED_IN_DOC,
            "{who}"
        );
        assert!(
            limits.chunk > 0 && limits.page > 0 && limits.units_a_day > 0,
            "{who}"
        );
        assert!(!limits.poll_every.is_zero(), "{who}");
        let all = [
            costs.list,
            costs.fetch,
            costs.put,
            costs.delete,
            costs.about,
            costs.changes,
        ];
        assert!(all.iter().all(|cost| *cost > 0), "{who}: nothing is free");
        let polls = 86_400 / limits.poll_every.as_secs();
        assert!(
            polls * costs.changes <= limits.units_a_day / 2,
            "{who}: polling at the shortest interval leaves half the day for the work"
        );
    });
}

#[test]
fn a_refusal_comes_back_as_the_provider_gave_it_and_changes_nothing() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "keep", b"1", Expect::Absent).unwrap();
        let hitches = [
            Hitch::Limited {
                wait: Duration::from_secs(7),
            },
            Hitch::Full,
            Hitch::Lost,
            Hitch::Elsewhere {
                found: "otra".into(),
            },
            Hitch::Unreachable("sin red".into()),
            Hitch::Broke("raro".into()),
        ];
        for hitch in hitches {
            remote.forget_counts();
            remote.fail_next(hitch.clone());

            let refused = put(remote, room, "new", b"22", Expect::Absent);

            assert_eq!(refused, Err(hitch), "{who}");
            assert_eq!(remote.counts().requests, 1, "{who}: it was asked");
            assert_eq!(remote.counts().sent, 0, "{who}: and nothing went");
            assert_eq!(names(remote.list("").unwrap()), ["keep"], "{who}");
        }
    });
}

#[test]
fn every_verb_hears_the_refusal_once_and_works_again() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "keep", b"1", Expect::Absent).unwrap();
        let limited = Hitch::Limited {
            wait: Duration::from_secs(7),
        };
        let again = |refused: Option<Hitch>, verb: &str| {
            assert_eq!(refused, Some(limited.clone()), "{who}: {verb}");
            remote.fail_next(limited.clone());
        };
        remote.fail_next(limited.clone());

        again(remote.list("").err(), "list");
        again(body_of(remote, "keep", 0).err(), "fetch");
        again(remote.delete("keep", None).err(), "delete");
        again(remote.about("keep").err(), "about");
        again(remote.changes(None).err(), "changes");
        if remote.lends_links() {
            again(remote.lends("keep").err(), "lends");
        }
        remote.list("").unwrap_err();

        assert_eq!(
            names(remote.list("").unwrap()),
            ["keep"],
            "{who}: still there"
        );
        assert!(remote.about("keep").unwrap().is_some(), "{who}");
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
    drive.delete("docs/a.md", None).unwrap();
    assert_eq!(
        drive.copies_of("docs/a.md"),
        0,
        "a deleted name is gone, not shown again"
    );
    assert_eq!(drive.about("docs/a.md").unwrap(), None);
}

#[test]
fn a_watch_hears_every_change_after_the_cursor_it_was_given_and_only_where_pushed() {
    let room = tempfile::tempdir().unwrap();
    let quiet = Duration::from_millis(1);
    for fake in fakes::every() {
        let who = fake.who();
        let cursor = cursor_of(fake.changes(None).unwrap());

        let watch = fake.hears(&cursor);

        assert_eq!(watch.is_some(), fake.hears_pushed(), "{who}");
        let Some(mut watch) = watch else { continue };
        assert!(!watch.told(quiet).unwrap(), "{who}");
        put(fake.as_ref(), room.path(), "x", b"1", Expect::Absent).unwrap();
        assert!(watch.told(quiet).unwrap(), "{who}");
        assert!(!watch.told(quiet).unwrap(), "{who}: once per change");
        let mut late = fake.hears(&cursor).unwrap();
        assert!(
            late.told(quiet).unwrap(),
            "{who}: what came before it was made"
        );
    }
}

#[test]
fn a_link_is_lent_for_what_exists_and_only_where_the_provider_lends() {
    on_each(|remote, room| {
        let who = remote.who();
        put(remote, room, "x", b"1", Expect::Absent).unwrap();

        let lent = remote.lends("x").unwrap();

        assert_eq!(lent.is_some(), remote.lends_links(), "{who}");
        assert_eq!(remote.lends("nada").unwrap(), None, "{who}");
    });
}
