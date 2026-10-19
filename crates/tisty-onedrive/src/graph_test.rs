use std::collections::HashMap;
use std::io::Write;
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use tisty_carrier::testing::{Asked, Fresh, Kept, heard};
use tisty_carrier::{Changes, Expect, Hitch, Remote, Tokens};

use super::{GraphAt, OneDrive};

struct Said {
    status: &'static str,
    headers: String,
    body: String,
}

fn said(status: &'static str, body: &str) -> Said {
    Said {
        status,
        headers: String::new(),
        body: body.to_string(),
    }
}

fn said_with(status: &'static str, headers: &str, body: &str) -> Said {
    Said {
        status,
        headers: format!("{headers}\r\n"),
        body: body.to_string(),
    }
}

fn graph(scripted: impl FnOnce(&str) -> Vec<Said>) -> (GraphAt, JoinHandle<Vec<Asked>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let script = scripted(&base);
    let served = std::thread::spawn(move || {
        script
            .into_iter()
            .map(|one| {
                let (mut stream, _) = listener.accept().unwrap();
                let asked = heard(&mut stream);
                write!(
                    stream,
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
                    one.status,
                    one.headers,
                    one.body.len(),
                    one.body
                )
                .unwrap();
                asked
            })
            .collect()
    });
    (
        GraphAt {
            api: format!("{base}/v1.0"),
        },
        served,
    )
}

fn onedrive(at: GraphAt, home: &std::path::Path) -> OneDrive {
    let kept = Kept(Mutex::new(HashMap::from([(
        "onedrive-me".to_string(),
        "refresh".to_string(),
    )])));
    let tokens = Tokens::new(
        Arc::new(kept),
        Arc::new(Fresh(Mutex::new(0))),
        "onedrive-me",
        home,
    );
    OneDrive::new(Arc::new(tokens), at).unwrap()
}

fn header(asked: &Asked, name: &str) -> Option<String> {
    asked
        .head
        .lines()
        .find(|one| one.to_ascii_lowercase().starts_with(&format!("{name}:")))
        .map(|one| one.split_once(':').unwrap().1.trim().to_string())
}

fn written(home: &tempfile::TempDir, body: &[u8]) -> std::path::PathBuf {
    let local = home.path().join("local.bin");
    std::fs::write(&local, body).unwrap();
    local
}

const ROOT: &str = r#"{"id":"R"}"#;

fn folder(id: &str, name: &str, parent: &str) -> String {
    format!(
        r#"{{"id":"{id}","name":"{name}","folder":{{}},"parentReference":{{"id":"{parent}"}}}}"#
    )
}

fn file(id: &str, name: &str, parent: &str, etag: &str) -> String {
    format!(
        r#"{{"id":"{id}","name":"{name}","eTag":"{etag}","size":4,"file":{{"hashes":{{"quickXorHash":"Q{id}"}}}},"parentReference":{{"id":"{parent}"}}}}"#
    )
}

fn page(items: &[String], link: &str, cursor: &str) -> String {
    format!(r#"{{"value":[{}],"{link}":"{cursor}"}}"#, items.join(","))
}

fn whole(base: &str, items: &[String], cursor: &str) -> Said {
    said(
        "200 OK",
        &page(
            items,
            "@odata.deltaLink",
            &format!("{base}/v1.0/me/drive/special/approot/delta?token={cursor}"),
        ),
    )
}

#[test]
fn every_file_under_the_app_s_folder_is_named_by_its_path_across_pages() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            said(
                "200 OK",
                &page(
                    &[
                        folder("S", "store", "R"),
                        file("F", "active.tisty", "S", "e1"),
                    ],
                    "@odata.nextLink",
                    &format!("{base}/v1.0/me/drive/special/approot/delta?token=p2"),
                ),
            ),
            whole(base, &[file("T", "tisty.toml", "R", "e2")], "d1"),
        ]
    });
    let drive = onedrive(at, home.path());

    let listed = drive.list("").unwrap();
    let store = drive.shown("store");

    let names: Vec<&str> = listed.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["store/active.tisty", "tisty.toml"]);
    assert_eq!(listed[0].hash, "quickxor:QF");
    assert_eq!(listed[0].revision, "e1");
    assert_eq!(store.len(), 1);
    let asked = served.join().unwrap();
    assert!(
        asked[0]
            .line
            .starts_with("GET /v1.0/me/drive/special/approot?"),
        "{}",
        asked[0].line
    );
    assert!(
        asked[1].line.contains("/approot/delta?"),
        "{}",
        asked[1].line
    );
    assert!(asked[2].line.contains("token=p2"), "{}", asked[2].line);
    assert_eq!(
        header(&asked[1], "authorization").as_deref(),
        Some("Bearer access-1")
    );
}

#[test]
fn a_page_refused_for_a_moment_is_asked_again_and_the_pages_before_it_are_kept() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            said(
                "200 OK",
                &page(
                    &[
                        folder("S", "store", "R"),
                        file("F", "active.tisty", "S", "e1"),
                    ],
                    "@odata.nextLink",
                    &format!("{base}/v1.0/me/drive/special/approot/delta?token=p2"),
                ),
            ),
            said_with("429 Too Many Requests", "Retry-After: 1", "{}"),
            whole(base, &[file("T", "tisty.toml", "R", "e2")], "d1"),
        ]
    });
    let drive = onedrive(at, home.path());

    let listed = drive.list("").unwrap();

    let names: Vec<&str> = listed.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["store/active.tisty", "tisty.toml"]);
    let asked = served.join().unwrap();
    assert_eq!(
        asked.len(),
        4,
        "the listing began again from its first page"
    );
    assert!(asked[3].line.contains("token=p2"), "{}", asked[3].line);
}

#[test]
fn changes_from_the_cursor_held_bring_only_what_moved_and_a_renamed_folder_moves_its_files() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            whole(
                base,
                &[
                    folder("S", "store", "R"),
                    file("F", "a.tisty", "S", "e1"),
                    file("G", "b.tisty", "R", "e1"),
                ],
                "d1",
            ),
            whole(
                base,
                &[
                    folder("S", "kept", "R"),
                    r#"{"id":"G","name":"b.tisty","deleted":{}}"#.to_string(),
                ],
                "d2",
            ),
        ]
    });
    let drive = onedrive(at, home.path());

    let Changes::Whole { cursor, seen } = drive.changes(None).unwrap() else {
        panic!("a first look sees everything");
    };
    assert_eq!(seen.len(), 2);
    let Changes::Since {
        changed,
        mut gone,
        cursor: next,
    } = drive.changes(Some(&cursor)).unwrap()
    else {
        panic!("a cursor held is followed");
    };

    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].name, "kept/a.tisty");
    gone.sort();
    assert_eq!(gone, ["b.tisty", "store/a.tisty"]);
    assert!(next.ends_with("token=d2"));
    let asked = served.join().unwrap();
    assert!(asked[2].line.contains("token=d1"), "{}", asked[2].line);
}

#[test]
fn a_cursor_onedrive_let_lapse_lists_everything_again() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            whole(base, &[file("F", "a.tisty", "R", "e1")], "d1"),
            said(
                "410 Gone",
                r#"{"error":{"code":"resyncRequired","message":"gone"}}"#,
            ),
            said("200 OK", ROOT),
            whole(base, &[file("F", "a.tisty", "R", "e2")], "d2"),
        ]
    });
    let drive = onedrive(at, home.path());

    let Changes::Whole { cursor, .. } = drive.changes(None).unwrap() else {
        panic!("a first look sees everything");
    };
    let again = drive.changes(Some(&cursor)).unwrap();

    let Changes::Whole { seen, cursor } = again else {
        panic!("a lapsed cursor is never read as nothing changed");
    };
    assert_eq!(seen[0].revision, "e2");
    assert!(cursor.ends_with("token=d2"));
    assert_eq!(served.join().unwrap().len(), 5);
}

#[test]
fn a_folder_onedrive_made_anew_is_found_again_under_its_new_id() {
    let home = tempfile::tempdir().unwrap();
    let (at, _served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            whole(base, &[file("F", "a.tisty", "R", "e1")], "d1"),
            said("404 Not Found", r#"{"error":{"code":"itemNotFound"}}"#),
            said("200 OK", r#"{"id":"R2"}"#),
            whole(base, &[file("G", "a.tisty", "R2", "e9")], "d9"),
        ]
    });
    let drive = onedrive(at, home.path());

    let Changes::Whole { cursor, .. } = drive.changes(None).unwrap() else {
        panic!("a first look sees everything");
    };
    let Changes::Whole { seen, .. } = drive.changes(Some(&cursor)).unwrap() else {
        panic!("a cursor of a folder that went is never read as nothing changed");
    };

    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].revision, "e9");
}

#[test]
fn a_round_asking_from_an_older_cursor_still_hears_what_a_poll_passed_over() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            whole(base, &[file("F", "a.tisty", "R", "e1")], "d1"),
            whole(base, &[file("F", "a.tisty", "R", "e2")], "d2"),
            whole(base, &[], "d3"),
        ]
    });
    let drive = onedrive(at, home.path());

    let Changes::Whole { cursor: first, .. } = drive.changes(None).unwrap() else {
        panic!("a first look sees everything");
    };
    drive.changes(Some(&first)).unwrap();
    let Changes::Since { changed, gone, .. } = drive.changes(Some(&first)).unwrap() else {
        panic!("a cursor passed is still followed");
    };

    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].revision, "e2");
    assert!(gone.is_empty());
    let asked = served.join().unwrap();
    assert!(
        asked[3].line.contains("token=d2"),
        "never asked again from d1: {}",
        asked[3].line
    );
}

#[test]
fn every_ask_no_verb_pays_for_is_counted_once() {
    let home = tempfile::tempdir().unwrap();
    let (at, _served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            said(
                "200 OK",
                &page(
                    &[],
                    "@odata.nextLink",
                    &format!("{base}/v1.0/me/drive/special/approot/delta?token=p2"),
                ),
            ),
            whole(base, &[], "d1"),
        ]
    });
    let drive = onedrive(at, home.path());

    drive.list("").unwrap();

    assert_eq!(
        drive.spent_beyond(),
        2,
        "the folder's id and the second page"
    );
    assert_eq!(drive.spent_beyond(), 0);
}

#[test]
fn a_cursor_this_copy_never_held_lists_everything() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", ROOT),
            whole(base, &[file("F", "a.tisty", "R", "e1")], "d2"),
        ]
    });
    let drive = onedrive(at.clone(), home.path());

    let said = drive
        .changes(Some(&format!(
            "{}/me/drive/special/approot/delta?token=old",
            at.api
        )))
        .unwrap();

    assert!(matches!(said, Changes::Whole { .. }));
    let asked = served.join().unwrap();
    assert!(!asked.iter().any(|one| one.line.contains("token=old")));
}

#[test]
fn a_link_that_leaves_graph_is_never_followed_with_the_token() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![
            said("200 OK", ROOT),
            said(
                "200 OK",
                &page(&[], "@odata.nextLink", "https://elsewhere.example/steal"),
            ),
        ]
    });
    let drive = onedrive(at, home.path());

    assert!(matches!(drive.list(""), Err(Hitch::Broke(_))));
    assert_eq!(served.join().unwrap().len(), 2);
}

#[test]
fn a_file_is_brought_from_its_own_address_without_the_token_and_from_where_it_was_left() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said(
                "200 OK",
                &format!(
                    r#"{{"id":"F","name":"a.tisty","eTag":"e1","size":6,"file":{{}},"@microsoft.graph.downloadUrl":"{base}/dl/F"}}"#
                ),
            ),
            said_with("206 Partial Content", "Content-Range: bytes 2-5/6", "cdef"),
        ]
    });
    let drive = onedrive(at, home.path());
    let mut got = Vec::new();

    let seen = drive.fetch("store/a.tisty", 2, &mut got).unwrap();

    assert_eq!(got, b"cdef");
    assert_eq!(seen.revision, "e1");
    let asked = served.join().unwrap();
    assert!(
        asked[0]
            .line
            .starts_with("GET /v1.0/me/drive/special/approot:/store/a.tisty:"),
        "{}",
        asked[0].line
    );
    assert_eq!(asked[1].line, "GET /dl/F HTTP/1.1");
    assert_eq!(header(&asked[1], "range").as_deref(), Some("bytes=2-"));
    assert_eq!(header(&asked[1], "authorization"), None);
}

#[test]
fn a_piece_other_than_the_one_asked_for_is_never_written_after_what_is_here() {
    let home = tempfile::tempdir().unwrap();
    let (at, _served) = graph(|base| {
        vec![
            said(
                "200 OK",
                &format!(
                    r#"{{"id":"F","name":"a.tisty","eTag":"e1","size":6,"file":{{}},"@microsoft.graph.downloadUrl":"{base}/dl/F"}}"#
                ),
            ),
            said_with(
                "206 Partial Content",
                "Content-Range: bytes 0-5/6",
                "abcdef",
            ),
        ]
    });
    let drive = onedrive(at, home.path());
    let mut got = Vec::new();

    let refused = drive.fetch("a.tisty", 2, &mut got);

    assert!(matches!(refused, Err(Hitch::Broke(_))), "{refused:?}");
    assert!(got.is_empty());
}

#[test]
fn a_range_onedrive_ignored_passes_over_what_was_already_here() {
    let home = tempfile::tempdir().unwrap();
    let (at, _served) = graph(|base| {
        vec![
            said(
                "200 OK",
                &format!(
                    r#"{{"id":"F","name":"a.tisty","eTag":"e1","size":6,"file":{{}},"@microsoft.graph.downloadUrl":"{base}/dl/F"}}"#
                ),
            ),
            said("200 OK", "abcdef"),
        ]
    });
    let drive = onedrive(at, home.path());
    let mut got = Vec::new();

    drive.fetch("a.tisty", 2, &mut got).unwrap();

    assert_eq!(got, b"cdef");
}

#[test]
fn a_file_not_there_is_missing_when_brought_and_absent_when_asked_about() {
    let home = tempfile::tempdir().unwrap();
    let gone = r#"{"error":{"code":"itemNotFound","message":"no"}}"#;
    let (at, _served) = graph(|_| vec![said("404 Not Found", gone), said("404 Not Found", gone)]);
    let drive = onedrive(at, home.path());

    assert!(matches!(
        drive.fetch("a.tisty", 0, &mut Vec::new()),
        Err(Hitch::Missing(name)) if name == "a.tisty"
    ));
    assert_eq!(drive.about("a.tisty").unwrap(), None);
}

#[test]
fn a_small_file_whose_answer_was_lost_is_found_landed_rather_than_sent_twice() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"abcd");
    let hash = crate::quickxor::of_file(&local).unwrap();
    let (at, served) = graph(|_| {
        vec![
            said("503 Service Unavailable", "{}"),
            said(
                "200 OK",
                &format!(
                    r#"{{"id":"F","name":"a.tisty","eTag":"e1","size":4,"file":{{"hashes":{{"quickXorHash":"{hash}"}}}}}}"#
                ),
            ),
        ]
    });
    let drive = onedrive(at, home.path());

    let seen = drive.put("store/a.tisty", &local, Expect::Absent).unwrap();

    assert_eq!(seen.revision, "e1");
    assert_eq!(served.join().unwrap().len(), 2);
}

#[test]
fn a_file_of_the_same_size_another_machine_left_is_never_taken_for_this_write() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![
            said("503 Service Unavailable", "{}"),
            said(
                "200 OK",
                r#"{"id":"F","name":"a.tisty","eTag":"e1","size":4,"file":{"hashes":{"quickXorHash":"other"}}}"#,
            ),
            said("409 Conflict", r#"{"error":{"code":"nameAlreadyExists"}}"#),
        ]
    });
    let drive = onedrive(at, home.path());

    let refused = drive.put("store/a.tisty", &written(&home, b"abcd"), Expect::Absent);

    assert!(matches!(refused, Err(Hitch::Changed(_))));
    assert_eq!(served.join().unwrap().len(), 3);
}

#[test]
fn a_download_address_outside_microsoft_is_never_followed() {
    let home = tempfile::tempdir().unwrap();
    let (at, _served) = graph(|_| {
        vec![said(
            "200 OK",
            r#"{"id":"F","name":"a.tisty","eTag":"e1","size":6,"file":{},"@microsoft.graph.downloadUrl":"https://elsewhere.example/dl/F"}"#,
        )]
    });
    let drive = onedrive(at, home.path());

    let refused = drive.fetch("store/a.tisty", 0, &mut Vec::new());

    assert!(matches!(refused, Err(Hitch::Broke(_))));
}

#[test]
fn a_download_address_that_lapsed_is_asked_for_again_and_never_the_sign_in() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        let item = format!(
            r#"{{"id":"F","name":"a.tisty","eTag":"e1","size":3,"file":{{}},"@microsoft.graph.downloadUrl":"{base}/dl/F"}}"#
        );
        vec![
            said("200 OK", &item),
            said("401 Unauthorized", "{}"),
            said("200 OK", &item),
            said("200 OK", "abc"),
        ]
    });
    let drive = onedrive(at, home.path());
    let mut got = Vec::new();

    drive.fetch("store/a.tisty", 0, &mut got).unwrap();

    assert_eq!(got, b"abc");
    assert_eq!(served.join().unwrap().len(), 4);
}

#[test]
fn a_file_whose_fresh_address_is_refused_too_is_left_and_the_sign_in_still_stands() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        let item = format!(
            r#"{{"id":"F","name":"a.tisty","eTag":"e1","size":3,"file":{{}},"@microsoft.graph.downloadUrl":"{base}/dl/F"}}"#
        );
        vec![
            said("200 OK", &item),
            said("403 Forbidden", r#"{"error":{"code":"accessDenied"}}"#),
            said("200 OK", &item),
            said("403 Forbidden", r#"{"error":{"code":"accessDenied"}}"#),
        ]
    });
    let drive = onedrive(at, home.path());

    let refused = drive.fetch("store/a.tisty", 0, &mut Vec::new());

    assert!(
        matches!(&refused, Err(Hitch::Broke(why)) if why.contains("store/a.tisty")),
        "{refused:?}"
    );
    assert_eq!(served.join().unwrap().len(), 4);
}

#[test]
fn a_new_small_file_lands_whole_and_only_where_nothing_is() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| vec![said("201 Created", &file("F", "a b#c.tisty", "S", "e1"))]);
    let drive = onedrive(at, home.path());

    let seen = drive
        .put(
            "store/a b#c.tisty",
            &written(&home, b"abcd"),
            Expect::Absent,
        )
        .unwrap();

    assert_eq!(seen.name, "store/a b#c.tisty");
    assert_eq!(seen.revision, "e1");
    let asked = served.join().unwrap();
    assert!(
        asked[0]
            .line
            .starts_with("PUT /v1.0/me/drive/special/approot:/store/a%20b%23c.tisty:/content?"),
        "{}",
        asked[0].line
    );
    assert!(
        asked[0].line.contains("conflictBehavior=fail"),
        "{}",
        asked[0].line
    );
    assert_eq!(asked[0].body, b"abcd");
    assert_eq!(header(&asked[0], "if-match"), None);
}

#[test]
fn a_whole_write_whose_token_is_renewed_sends_the_file_again_from_its_start() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![
            said("401 Unauthorized", "{}"),
            said("201 Created", &file("F", "a.tisty", "R", "e1")),
        ]
    });
    let drive = onedrive(at, home.path());

    drive
        .put("a.tisty", &written(&home, b"abcd"), Expect::Absent)
        .unwrap();

    let asked = served.join().unwrap();
    assert_eq!(asked[0].body, b"abcd");
    assert_eq!(asked[1].body, b"abcd");
    assert_eq!(
        header(&asked[1], "authorization").as_deref(),
        Some("Bearer access-2")
    );
}

#[test]
fn a_new_file_that_finds_one_there_already_is_told_it_changed() {
    let home = tempfile::tempdir().unwrap();
    let (at, _served) = graph(|_| {
        vec![said(
            "409 Conflict",
            r#"{"error":{"code":"nameAlreadyExists","message":"taken"}}"#,
        )]
    });
    let drive = onedrive(at, home.path());

    assert!(matches!(
        drive.put("a.tisty", &written(&home, b"abcd"), Expect::Absent),
        Err(Hitch::Changed(_))
    ));
}

#[test]
fn a_write_over_a_revision_names_it_and_tells_a_newer_one_from_a_file_that_left() {
    let home = tempfile::tempdir().unwrap();
    let mismatch = r#"{"error":{"code":"resourceModified","message":"etag"}}"#;
    let (at, served) = graph(|_| {
        vec![
            said("412 Precondition Failed", mismatch),
            said("200 OK", &file("F", "a.tisty", "R", "e2")),
            said("412 Precondition Failed", mismatch),
            said("404 Not Found", r#"{"error":{"code":"itemNotFound"}}"#),
        ]
    });
    let drive = onedrive(at, home.path());
    let local = written(&home, b"abcd");

    let newer = drive.put("a.tisty", &local, Expect::Revision("\"e1\"".into()));
    let left = drive.put("a.tisty", &local, Expect::Revision("\"e1\"".into()));

    assert!(matches!(newer, Err(Hitch::Changed(_))));
    assert!(matches!(left, Err(Hitch::Missing(_))));
    let asked = served.join().unwrap();
    assert_eq!(header(&asked[0], "if-match").as_deref(), Some("\"e1\""));
    assert!(
        asked[0].line.contains("conflictBehavior=replace"),
        "{}",
        asked[0].line
    );
}

#[test]
fn a_large_file_goes_up_in_pieces_without_the_token_and_lands_on_the_last_one() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", &format!(r#"{{"uploadUrl":"{base}/up/1"}}"#)),
            said("202 Accepted", r#"{"nextExpectedRanges":["4-"]}"#),
            said("202 Accepted", r#"{"nextExpectedRanges":["8-"]}"#),
            said("201 Created", &file("F", "big.pdf", "A", "e1")),
        ]
    });
    let drive = onedrive(at, home.path()).with_chunk(4);

    let seen = drive
        .put(
            "attachments/big.pdf",
            &written(&home, b"0123456789"),
            Expect::Absent,
        )
        .unwrap();

    assert_eq!(seen.revision, "e1");
    let asked = served.join().unwrap();
    assert!(
        asked[0].line.contains(":/createUploadSession"),
        "{}",
        asked[0].line
    );
    assert!(String::from_utf8_lossy(&asked[0].body).contains(r#""fail""#));
    let ranges: Vec<String> = asked[1..]
        .iter()
        .map(|one| header(one, "content-range").unwrap())
        .collect();
    assert_eq!(ranges, ["bytes 0-3/10", "bytes 4-7/10", "bytes 8-9/10"]);
    assert_eq!(asked[3].body, b"89");
    assert!(
        asked[1..]
            .iter()
            .all(|one| header(one, "authorization").is_none())
    );
}

#[test]
fn an_upload_cut_short_carries_on_after_a_restart_from_where_onedrive_says_it_got() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", &format!(r#"{{"uploadUrl":"{base}/up/1"}}"#)),
            said("202 Accepted", r#"{"nextExpectedRanges":["4-"]}"#),
            said_with("429 Too Many Requests", "Retry-After: 1", "{}"),
            said("200 OK", r#"{"nextExpectedRanges":["4-"]}"#),
            said("202 Accepted", r#"{"nextExpectedRanges":["8-"]}"#),
            said("201 Created", &file("F", "big.pdf", "A", "e1")),
        ]
    });
    let local = written(&home, b"0123456789");
    let first = onedrive(at.clone(), home.path())
        .with_chunk(4)
        .keeping_at(home.path());

    let cut = first.put("attachments/big.pdf", &local, Expect::Absent);
    assert!(cut.is_err());
    assert!(home.path().join("uploading.json").exists());

    let again = onedrive(at, home.path())
        .with_chunk(4)
        .keeping_at(home.path());
    let seen = again
        .put("attachments/big.pdf", &local, Expect::Absent)
        .unwrap();

    assert_eq!(seen.revision, "e1");
    let asked = served.join().unwrap();
    assert_eq!(asked[3].line, "GET /up/1 HTTP/1.1");
    assert_eq!(
        header(&asked[4], "content-range").as_deref(),
        Some("bytes 4-7/10")
    );
    assert_eq!(
        header(&asked[5], "content-range").as_deref(),
        Some("bytes 8-9/10")
    );
    assert!(
        !home.path().join("uploading.json").exists(),
        "a finished upload was still written down as going on"
    );
}

#[test]
fn a_session_that_kept_every_byte_but_never_made_the_file_is_let_go_and_begun_again() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", &format!(r#"{{"uploadUrl":"{base}/up/1"}}"#)),
            said("202 Accepted", r#"{"nextExpectedRanges":["4-"]}"#),
            said_with("429 Too Many Requests", "Retry-After: 1", "{}"),
            said("200 OK", r#"{"nextExpectedRanges":[]}"#),
            said("404 Not Found", r#"{"error":{"code":"itemNotFound"}}"#),
            said("204 No Content", ""),
            said("200 OK", &format!(r#"{{"uploadUrl":"{base}/up/2"}}"#)),
            said("202 Accepted", r#"{"nextExpectedRanges":["4-"]}"#),
            said("202 Accepted", r#"{"nextExpectedRanges":["8-"]}"#),
            said("201 Created", &file("F", "big.pdf", "A", "e1")),
        ]
    });
    let local = written(&home, b"0123456789");
    let first = onedrive(at.clone(), home.path())
        .with_chunk(4)
        .keeping_at(home.path());
    assert!(
        first
            .put("attachments/big.pdf", &local, Expect::Absent)
            .is_err()
    );

    let again = onedrive(at, home.path())
        .with_chunk(4)
        .keeping_at(home.path());
    let seen = again
        .put("attachments/big.pdf", &local, Expect::Absent)
        .unwrap();

    assert_eq!(seen.revision, "e1");
    let asked = served.join().unwrap();
    assert_eq!(asked[5].line, "DELETE /up/1 HTTP/1.1");
    assert!(
        asked[6].line.contains(":/createUploadSession"),
        "{}",
        asked[6].line
    );
    assert_eq!(
        header(&asked[7], "content-range").as_deref(),
        Some("bytes 0-3/10")
    );
    assert!(!home.path().join("uploading.json").exists());
}

#[test]
fn an_upload_refused_on_its_last_piece_is_let_go_of_up_there() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![
            said("200 OK", &format!(r#"{{"uploadUrl":"{base}/up/1"}}"#)),
            said("202 Accepted", "{}"),
            said("409 Conflict", r#"{"error":{"code":"nameAlreadyExists"}}"#),
            said("204 No Content", ""),
        ]
    });
    let drive = onedrive(at, home.path()).with_chunk(4);

    let refused = drive.put("big.pdf", &written(&home, b"01234567"), Expect::Absent);

    assert!(matches!(refused, Err(Hitch::Changed(_))));
    let asked = served.join().unwrap();
    assert_eq!(asked[3].line, "DELETE /up/1 HTTP/1.1");
}

#[test]
fn a_file_is_moved_in_place_into_a_folder_that_is_there() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![
            said("200 OK", &file("F", "old.pdf", "A", "e1")),
            said("200 OK", &folder("B", "kept", "R")),
            said("200 OK", &file("F", "new.pdf", "B", "e2")),
        ]
    });
    let drive = onedrive(at, home.path());

    drive
        .renamed("attachments/old.pdf", "kept/new.pdf")
        .unwrap();

    let asked = served.join().unwrap();
    assert!(
        asked[2].line.starts_with("PATCH /v1.0/me/drive/items/F?"),
        "{}",
        asked[2].line
    );
    let body: serde_json::Value = serde_json::from_slice(&asked[2].body).unwrap();
    assert_eq!(body["name"], "new.pdf");
    assert_eq!(body["parentReference"]["id"], "B");
    assert_eq!(
        asked.len(),
        3,
        "the bytes went down and up instead of moving"
    );
}

#[test]
fn a_file_is_deleted_only_at_the_revision_named() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![
            said("200 OK", &file("F!1", "a.tisty", "R", "e1")),
            said("204 No Content", ""),
            said("200 OK", &file("F!1", "a.tisty", "R", "e2")),
        ]
    });
    let drive = onedrive(at, home.path());

    drive.delete("a.tisty", Some("e1")).unwrap();
    let newer = drive.delete("a.tisty", Some("e1"));

    assert!(matches!(newer, Err(Hitch::Changed(_))));
    let asked = served.join().unwrap();
    assert_eq!(asked.len(), 3, "a newer revision is never asked to go");
    assert_eq!(asked[1].line, "DELETE /v1.0/me/drive/items/F!1 HTTP/1.1");
    assert_eq!(header(&asked[1], "if-match").as_deref(), Some("e1"));
}

#[test]
fn a_file_is_deleted_at_its_content_tag_whatever_a_rename_did_to_its_etag() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![
            said(
                "200 OK",
                r#"{"id":"F","name":"a.tisty","eTag":"e9","cTag":"c1","size":4,"file":{}}"#,
            ),
            said("204 No Content", ""),
        ]
    });
    let drive = onedrive(at, home.path());

    drive.delete("a.tisty", Some("c1")).unwrap();

    let asked = served.join().unwrap();
    assert!(asked[0].line.contains("cTag"), "{}", asked[0].line);
    assert_eq!(header(&asked[1], "if-match").as_deref(), Some("c1"));
}

#[test]
fn the_room_left_is_what_the_drive_says_remains() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![said(
            "200 OK",
            r#"{"quota":{"total":100,"used":40,"remaining":60}}"#,
        )]
    });
    let drive = onedrive(at, home.path());

    assert_eq!(drive.room().unwrap(), Some(60));
    let asked = served.join().unwrap();
    assert!(
        asked[0].line.starts_with("GET /v1.0/me/drive?"),
        "{}",
        asked[0].line
    );
}

#[test]
fn each_refusal_onedrive_gives_is_read_as_what_it_means() {
    let home = tempfile::tempdir().unwrap();
    let (at, _served) = graph(|_| {
        vec![
            said_with("429 Too Many Requests", "Retry-After: 7", "{}"),
            said(
                "507 Insufficient Storage",
                r#"{"error":{"code":"quotaLimitReached"}}"#,
            ),
            said("403 Forbidden", r#"{"error":{"code":"accessDenied"}}"#),
            said("503 Service Unavailable", "{}"),
            said(
                "400 Bad Request",
                r#"{"error":{"code":"invalidRequest","innerError":{"code":"quotaLimitReached"}}}"#,
            ),
            said("423 Locked", r#"{"error":{"code":"resourceLocked"}}"#),
            said(
                "403 Forbidden",
                r#"{"error":{"code":"activityLimitReached"}}"#,
            ),
        ]
    });
    let drive = onedrive(at, home.path());

    assert_eq!(
        drive.about("a.tisty"),
        Err(Hitch::Limited {
            wait: Duration::from_secs(7)
        })
    );
    assert_eq!(drive.about("a.tisty"), Err(Hitch::Full));
    assert!(matches!(drive.about("a.tisty"), Err(Hitch::Broke(_))));
    assert!(matches!(drive.about("a.tisty"), Err(Hitch::Troubled(_))));
    assert_eq!(drive.about("a.tisty"), Err(Hitch::Full));
    assert!(matches!(drive.about("a.tisty"), Err(Hitch::Limited { .. })));
    assert!(matches!(drive.about("a.tisty"), Err(Hitch::Limited { .. })));
}

#[test]
fn a_token_onedrive_turns_down_is_renewed_once_and_then_the_access_is_lost() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![
            said("401 Unauthorized", "{}"),
            said("401 Unauthorized", "{}"),
        ]
    });
    let drive = onedrive(at, home.path());

    assert_eq!(drive.about("a.tisty"), Err(Hitch::Lost));
    let asked = served.join().unwrap();
    assert_eq!(
        header(&asked[0], "authorization").as_deref(),
        Some("Bearer access-1")
    );
    assert_eq!(
        header(&asked[1], "authorization").as_deref(),
        Some("Bearer access-2")
    );
}

#[test]
fn another_account_is_refused_before_anything_is_read() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| {
        vec![said(
            "200 OK",
            r#"{"id":"other","mail":null,"userPrincipalName":"otra@outlook.com"}"#,
        )]
    });
    let drive = onedrive(at, home.path()).for_account("mine");

    assert_eq!(
        drive.list(""),
        Err(Hitch::Elsewhere {
            found: "otra@outlook.com".into()
        })
    );
    assert_eq!(served.join().unwrap().len(), 1);
}

#[test]
fn a_name_onedrive_would_refuse_never_reaches_it() {
    let home = tempfile::tempdir().unwrap();
    let drive = onedrive(
        GraphAt {
            api: "http://127.0.0.1:9/v1.0".into(),
        },
        home.path(),
    );

    assert!(matches!(
        drive.put("store/a:b", &written(&home, b"x"), Expect::Any),
        Err(Hitch::Broke(_))
    ));
    assert!(matches!(drive.about("con"), Err(Hitch::Broke(_))));
}

#[test]
fn a_file_here_is_printed_as_onedrive_prints_its_own() {
    let home = tempfile::tempdir().unwrap();
    let drive = onedrive(GraphAt::microsoft(), home.path());

    let hash = drive.hash_of(&written(&home, b"J")).unwrap();

    assert_eq!(hash, "quickxor:SgAAAAAAAAAAAAAAAQAAAAAAAAA=");
}

#[test]
fn the_pieces_of_an_upload_are_what_onedrive_takes_and_names_fold_case() {
    let home = tempfile::tempdir().unwrap();
    let limits = onedrive(GraphAt::microsoft(), home.path()).limits();

    assert_eq!(limits.provider, "OneDrive");
    assert!(limits.folds_case);
    assert_eq!(limits.chunk % (320 * 1024), 0);
    assert!(limits.chunk < 60 * 1024 * 1024);
}

#[test]
fn a_large_write_over_a_revision_goes_whole_so_the_revision_is_checked_as_it_lands() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| vec![said("200 OK", &file("F", "big.tisty", "R", "e2"))]);
    let drive = onedrive(at, home.path()).with_chunk(4);

    drive
        .put(
            "big.tisty",
            &written(&home, b"0123456789"),
            Expect::Revision("e1".into()),
        )
        .unwrap();

    let asked = served.join().unwrap();
    assert_eq!(asked.len(), 1);
    assert!(asked[0].line.contains(":/content?"), "{}", asked[0].line);
    assert_eq!(header(&asked[0], "if-match").as_deref(), Some("e1"));
    assert_eq!(asked[0].body, b"0123456789");
}

#[test]
fn a_folder_is_never_deleted_in_place_of_a_file() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|_| vec![said("200 OK", &folder("S", "store", "R"))]);
    let drive = onedrive(at, home.path());

    assert!(matches!(
        drive.delete("store", None),
        Err(Hitch::Missing(_))
    ));
    assert_eq!(served.join().unwrap().len(), 1);
}

#[test]
fn a_file_already_all_here_is_not_asked_for_past_its_end() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = graph(|base| {
        vec![said(
            "200 OK",
            &format!(
                r#"{{"id":"F","name":"a.tisty","eTag":"e1","size":6,"file":{{}},"@microsoft.graph.downloadUrl":"{base}/dl/F"}}"#
            ),
        )]
    });
    let drive = onedrive(at, home.path());
    let mut got = Vec::new();

    let seen = drive.fetch("a.tisty", 6, &mut got).unwrap();

    assert!(got.is_empty());
    assert_eq!(seen.bytes, 6);
    assert_eq!(served.join().unwrap().len(), 1);
}

#[test]
fn every_verb_asks_whose_drive_it_is_before_touching_it() {
    let home = tempfile::tempdir().unwrap();
    let elsewhere = r#"{"id":"other","userPrincipalName":"otra@outlook.com"}"#;
    let (at, served) = graph(|_| {
        vec![
            said("200 OK", elsewhere),
            said("200 OK", elsewhere),
            said("200 OK", elsewhere),
        ]
    });
    let drive = onedrive(at, home.path()).for_account("mine");
    let refused = Hitch::Elsewhere {
        found: "otra@outlook.com".into(),
    };

    assert_eq!(drive.about("a.tisty"), Err(refused.clone()));
    assert_eq!(
        drive.fetch("a.tisty", 0, &mut Vec::new()).err(),
        Some(refused.clone())
    );
    assert_eq!(drive.delete("a.tisty", None), Err(refused));
    let asked = served.join().unwrap();
    assert!(
        asked
            .iter()
            .all(|one| one.line.starts_with("GET /v1.0/me?"))
    );
}
