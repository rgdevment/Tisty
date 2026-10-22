use std::io::Write;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread::JoinHandle;

use tisty_carrier::testing::{Asked, Kept, heard};
use tisty_carrier::{Expect, Hitch, Remote, Tokens};

use super::WebDav;
use crate::{DavAt, Password};

struct Said {
    status: &'static str,
    headers: String,
    body: String,
}

fn said(status: &'static str, body: &str) -> Said {
    said_with(status, "", body)
}

fn said_with(status: &'static str, headers: &str, body: &str) -> Said {
    Said {
        status,
        headers: match headers {
            "" => String::new(),
            some => format!("{some}\r\n"),
        },
        body: body.to_string(),
    }
}

fn server(script: Vec<Said>) -> (DavAt, JoinHandle<Vec<Asked>>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let base = format!(
        "http://127.0.0.1:{}/dav/files/mario",
        listener.local_addr().unwrap().port()
    );
    let served = std::thread::spawn(move || {
        script
            .into_iter()
            .map(|one| {
                let (mut stream, _) = listener.accept().unwrap();
                let asked = heard(&mut stream);
                write!(
                    stream,
                    "HTTP/1.1 {}\r\nContent-Type: application/xml\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
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
    (DavAt::under(&base, "mario").unwrap(), served)
}

fn dav(at: DavAt, home: &std::path::Path) -> WebDav {
    let tokens = Tokens::new(
        Arc::new(Kept::holding("refresh", "r0ck4me!")),
        Arc::new(Password),
        "refresh",
        home,
    );
    WebDav::new(Arc::new(tokens), at).unwrap()
}

fn written(home: &tempfile::TempDir, body: &[u8]) -> std::path::PathBuf {
    let local = home.path().join("local.bin");
    std::fs::write(&local, body).unwrap();
    local
}

fn multistatus(responses: &[String]) -> String {
    format!(
        r#"<?xml version="1.0"?><d:multistatus xmlns:d="DAV:">{}</d:multistatus>"#,
        responses.concat()
    )
}

fn folder(href: &str) -> String {
    format!(
        r#"<d:response><d:href>{href}</d:href><d:propstat><d:prop><d:resourcetype><d:collection/></d:resourcetype></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>"#
    )
}

fn file(href: &str, bytes: u64, etag: &str) -> String {
    format!(
        r#"<d:response><d:href>{href}</d:href><d:propstat><d:prop><d:resourcetype/><d:getcontentlength>{bytes}</d:getcontentlength><d:getetag>{etag}</d:getetag></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>"#
    )
}

const ROOT: &str = "/dav/files/mario/Tisty/";

#[test]
fn every_file_under_tisty_s_folder_is_listed_by_its_path_folder_by_folder() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said(
            "207 Multi-Status",
            &multistatus(&[
                folder(ROOT),
                file(&format!("{ROOT}tisty.toml"), 12, "\"e1\""),
                folder(&format!("{ROOT}store/")),
            ]),
        ),
        said(
            "207 Multi-Status",
            &multistatus(&[
                folder(&format!("{ROOT}store/")),
                file(&format!("{ROOT}store/my%20notes.md"), 4, "\"e2\""),
            ]),
        ),
    ]);
    let dav = dav(at, home.path());

    let mut listed = dav.list("").unwrap();
    listed.sort_by(|one, other| one.name.cmp(&other.name));

    let names: Vec<&str> = listed.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["store/my notes.md", "tisty.toml"]);
    assert_eq!(listed[1].revision, "\"e1\"");
    assert_eq!(listed[1].bytes, 12);
    assert!(listed[1].hash.is_empty());
    let asked = served.join().unwrap();
    assert!(
        asked[0].line.starts_with(&format!("PROPFIND {ROOT} ")),
        "{}",
        asked[0].line
    );
    assert_eq!(asked[0].header("depth").as_deref(), Some("1"));
    assert!(
        asked[1]
            .line
            .starts_with(&format!("PROPFIND {ROOT}store/ ")),
        "{}",
        asked[1].line
    );
    assert!(
        asked[0]
            .header("authorization")
            .is_some_and(|one| one.starts_with("Basic "))
    );
}

#[test]
fn a_folder_never_made_lists_as_empty() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![said("404 Not Found", "")]);
    let dav = dav(at, home.path());

    assert!(dav.list("").unwrap().is_empty());
    served.join().unwrap();
}

#[test]
fn a_new_file_is_written_only_if_absent_after_the_folders_above_it() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("201 Created", ""),
        said("405 Method Not Allowed", ""),
        said_with("201 Created", "ETag: \"e9\"", ""),
    ]);
    let dav = dav(at, home.path());
    let local = written(&home, b"body");

    let seen = dav.put("store/a.md", &local, Expect::Absent).unwrap();

    assert_eq!(seen.revision, "\"e9\"");
    assert_eq!(seen.bytes, 4);
    let asked = served.join().unwrap();
    assert!(
        asked[0].line.starts_with(&format!("MKCOL {ROOT} ")),
        "{}",
        asked[0].line
    );
    assert!(
        asked[1].line.starts_with(&format!("MKCOL {ROOT}store/ ")),
        "{}",
        asked[1].line
    );
    assert!(
        asked[2].line.starts_with(&format!("PUT {ROOT}store/a.md ")),
        "{}",
        asked[2].line
    );
    assert_eq!(asked[2].header("if-none-match").as_deref(), Some("*"));
    assert_eq!(asked[2].body, b"body");
}

#[test]
fn a_write_over_a_revision_another_copy_replaced_answers_changed() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("201 Created", ""),
        said("412 Precondition Failed", ""),
        said(
            "207 Multi-Status",
            &multistatus(&[file(&format!("{ROOT}a.md"), 4, "\"e2\"")]),
        ),
    ]);
    let dav = dav(at, home.path());
    let local = written(&home, b"body");

    let refused = dav.put("a.md", &local, Expect::Revision("\"e1\"".into()));

    assert!(matches!(refused, Err(Hitch::Changed(_))), "{refused:?}");
    let asked = served.join().unwrap();
    assert_eq!(asked[1].header("if-match").as_deref(), Some("\"e1\""));
}

#[test]
fn a_write_over_a_revision_of_a_file_now_gone_answers_missing() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("201 Created", ""),
        said("412 Precondition Failed", ""),
        said("404 Not Found", ""),
    ]);
    let dav = dav(at, home.path());
    let local = written(&home, b"body");

    let refused = dav.put("a.md", &local, Expect::Revision("\"e1\"".into()));

    assert!(matches!(refused, Err(Hitch::Missing(_))), "{refused:?}");
    served.join().unwrap();
}

#[test]
fn a_server_with_no_strong_etag_is_never_written_over_blindly() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(Vec::new());
    let dav = dav(at, home.path());
    let local = written(&home, b"body");

    let put = dav.put("a.md", &local, Expect::Revision("W/\"e1\"".into()));
    let deleted = dav.delete("a.md", Some("modified:x:4"));

    assert!(matches!(put, Err(Hitch::Broke(_))), "{put:?}");
    assert!(matches!(deleted, Err(Hitch::Broke(_))), "{deleted:?}");
    assert!(served.join().unwrap().is_empty());
}

#[test]
fn a_written_file_without_an_etag_in_the_answer_is_asked_for_its_revision() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("201 Created", ""),
        said("204 No Content", ""),
        said(
            "207 Multi-Status",
            &multistatus(&[file(&format!("{ROOT}a.md"), 4, "\"e3\"")]),
        ),
    ]);
    let dav = dav(at, home.path());
    let local = written(&home, b"body");

    let seen = dav.put("a.md", &local, Expect::Any).unwrap();

    assert_eq!(seen.revision, "\"e3\"");
    let asked = served.join().unwrap();
    assert!(asked[1].header("if-match").is_none());
    assert_eq!(asked[2].header("depth").as_deref(), Some("0"));
}

#[test]
fn a_file_is_brought_from_where_this_copy_stopped() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![said_with(
        "206 Partial Content",
        "Content-Range: bytes 2-5/6\r\nETag: \"e1\"",
        "dyyy",
    )]);
    let dav = dav(at, home.path());
    let mut got = Vec::new();

    let seen = dav.fetch("a.md", 2, &mut got).unwrap();

    assert_eq!(got, b"dyyy");
    assert_eq!(seen.bytes, 6);
    assert_eq!(seen.revision, "\"e1\"");
    let asked = served.join().unwrap();
    assert_eq!(asked[0].header("range").as_deref(), Some("bytes=2-"));
}

#[test]
fn a_server_that_sends_the_whole_file_has_what_was_here_passed_over() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![said_with("200 OK", "ETag: \"e1\"", "abcdef")]);
    let dav = dav(at, home.path());
    let mut got = Vec::new();

    let seen = dav.fetch("a.md", 2, &mut got).unwrap();

    assert_eq!(got, b"cdef");
    assert_eq!(seen.bytes, 6);
    served.join().unwrap();
}

#[test]
fn a_download_with_a_weak_etag_takes_its_revision_from_the_listing() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said_with("200 OK", "ETag: W/\"e1\"", "abcdef"),
        said(
            "207 Multi-Status",
            &multistatus(&[file(&format!("{ROOT}a.md"), 6, "\"e1\"")]),
        ),
    ]);
    let dav = dav(at, home.path());
    let mut got = Vec::new();

    let seen = dav.fetch("a.md", 0, &mut got).unwrap();

    assert_eq!(got, b"abcdef");
    assert_eq!(seen.revision, "\"e1\"");
    let asked = served.join().unwrap();
    assert_eq!(asked[1].header("depth").as_deref(), Some("0"));
}

#[test]
fn a_part_that_does_not_begin_where_asked_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![said_with(
        "206 Partial Content",
        "Content-Range: bytes 0-5/6",
        "abcdef",
    )]);
    let dav = dav(at, home.path());

    assert!(matches!(
        dav.fetch("a.md", 2, &mut Vec::new()),
        Err(Hitch::Broke(_))
    ));
    served.join().unwrap();
}

#[test]
fn a_delete_against_a_revision_sends_it_and_a_gone_file_is_missing() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![said("204 No Content", ""), said("404 Not Found", "")]);
    let dav = dav(at, home.path());

    dav.delete("a.md", Some("\"e1\"")).unwrap();
    let again = dav.delete("a.md", None);

    assert!(matches!(again, Err(Hitch::Missing(_))), "{again:?}");
    let asked = served.join().unwrap();
    assert!(asked[0].line.starts_with(&format!("DELETE {ROOT}a.md ")));
    assert_eq!(asked[0].header("if-match").as_deref(), Some("\"e1\""));
}

#[test]
fn a_rename_moves_in_place_over_what_bears_the_name() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("201 Created", ""),
        said("201 Created", ""),
        said("201 Created", ""),
    ]);
    let dav = dav(at, home.path());

    dav.renamed("a.md", "kept/b c.md").unwrap();

    let asked = served.join().unwrap();
    assert!(
        asked[2].line.starts_with(&format!("MOVE {ROOT}a.md ")),
        "{}",
        asked[2].line
    );
    assert!(
        asked[2]
            .header("destination")
            .is_some_and(|one| one.ends_with(&format!("{ROOT}kept/b%20c.md"))),
        "{:?}",
        asked[2].header("destination")
    );
    assert_eq!(asked[2].header("overwrite").as_deref(), Some("T"));
}

#[test]
fn a_password_refused_twice_is_the_access_gone() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("401 Unauthorized", ""),
        said("401 Unauthorized", ""),
    ]);
    let dav = dav(at, home.path());

    let refused = dav.list("");

    assert!(matches!(refused, Err(Hitch::Lost)), "{refused:?}");
    assert_eq!(served.join().unwrap().len(), 2);
}

#[test]
fn a_redirect_is_never_followed_with_the_password() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![said_with(
        "302 Found",
        "Location: https://elsewhere.invalid/",
        "",
    )]);
    let dav = dav(at, home.path());

    assert!(dav.list("").is_err());
    assert_eq!(served.join().unwrap().len(), 1);
}

#[test]
fn the_room_left_is_what_the_server_says_and_a_negative_one_says_nothing() {
    let home = tempfile::tempdir().unwrap();
    let quota = |left: &str| {
        said(
            "207 Multi-Status",
            &multistatus(&[format!(
                r#"<d:response><d:href>{ROOT}</d:href><d:propstat><d:prop><d:quota-available-bytes>{left}</d:quota-available-bytes></d:prop><d:status>HTTP/1.1 200 OK</d:status></d:propstat></d:response>"#
            )]),
        )
    };
    let (at, served) = server(vec![quota("5000"), quota("-3")]);
    let dav = dav(at, home.path());

    assert_eq!(dav.room().unwrap(), Some(5000));
    assert_eq!(dav.room().unwrap(), None);
    served.join().unwrap();
}

#[test]
fn no_fingerprint_is_given_since_none_is_kept_up_there() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(Vec::new());
    let dav = dav(at, home.path());
    let local = written(&home, b"body");

    assert_eq!(dav.hash_of(&local).unwrap(), "");
    assert!(dav.hash_of(&home.path().join("no-such-file")).is_err());
    served.join().unwrap();
}

#[test]
fn a_server_already_holding_tisty_is_judged_by_the_etag_of_what_it_keeps() {
    let home = tempfile::tempdir().unwrap();
    let listing = |etag: &str| {
        said(
            "207 Multi-Status",
            &multistatus(&[folder(ROOT), file(&format!("{ROOT}a.md"), 4, etag)]),
        )
    };
    let (strong_at, strong_served) = server(vec![listing("\"e1\"")]);
    let (weak_at, weak_served) = server(vec![listing("W/\"e1\"")]);

    assert!(dav(strong_at, home.path()).guards_writes().unwrap());
    assert!(!dav(weak_at, home.path()).guards_writes().unwrap());
    assert_eq!(strong_served.join().unwrap().len(), 1);
    weak_served.join().unwrap();
}

#[test]
fn an_empty_server_is_judged_by_a_note_written_and_taken_back() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("404 Not Found", ""),
        said("201 Created", ""),
        said_with("201 Created", "ETag: \"e1\"", ""),
        said("204 No Content", ""),
    ]);

    assert!(dav(at, home.path()).guards_writes().unwrap());
    let asked = served.join().unwrap();
    assert!(
        asked[2]
            .line
            .starts_with(&format!("PUT {ROOT}tisty-probe ")),
        "{}",
        asked[2].line
    );
    assert_eq!(asked[2].header("if-none-match").as_deref(), Some("*"));
    assert!(
        asked[3]
            .line
            .starts_with(&format!("DELETE {ROOT}tisty-probe ")),
        "{}",
        asked[3].line
    );
}

#[test]
fn an_empty_server_that_gives_no_etag_for_the_note_does_not_guard_writes() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = server(vec![
        said("404 Not Found", ""),
        said("201 Created", ""),
        said("201 Created", ""),
        said(
            "207 Multi-Status",
            &multistatus(&[file(&format!("{ROOT}tisty-probe"), 11, "")]),
        ),
        said("204 No Content", ""),
    ]);

    assert!(!dav(at, home.path()).guards_writes().unwrap());
    served.join().unwrap();
}
