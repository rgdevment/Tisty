use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use tisty_carrier::{Renew, Renewed, Secrets, Tokens};

use super::*;

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

#[derive(Debug)]
struct Asked {
    line: String,
    head: String,
    body: Vec<u8>,
}

fn heard(stream: &mut TcpStream) -> Asked {
    let mut got = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let read = stream.read(&mut chunk).unwrap_or(0);
        got.extend_from_slice(&chunk[..read]);
        if let Some(end) = got.windows(4).position(|one| one == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&got[..end]).to_string();
            let wants = head
                .lines()
                .find_map(|one| {
                    one.to_ascii_lowercase()
                        .strip_prefix("content-length:")
                        .map(|n| n.trim().parse::<usize>().unwrap())
                })
                .unwrap_or(0);
            if got.len() >= end + 4 + wants {
                let line = head.lines().next().unwrap_or_default().to_string();
                return Asked {
                    line,
                    head,
                    body: got[end + 4..end + 4 + wants].to_vec(),
                };
            }
        }
        if read == 0 {
            panic!("the request ended early");
        }
    }
}

fn google(script: Vec<Said>) -> (DriveAt, JoinHandle<Vec<Asked>>) {
    google_at(move |_| script)
}

fn google_at(scripted: impl FnOnce(&str) -> Vec<Said>) -> (DriveAt, JoinHandle<Vec<Asked>>) {
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
        DriveAt {
            api: format!("{base}/drive/v3"),
            upload: format!("{base}/upload/drive/v3"),
        },
        served,
    )
}

struct Kept(Mutex<HashMap<String, String>>);

impl Secrets for Kept {
    fn read(&self, key: &str) -> Result<Option<String>, Hitch> {
        Ok(self.0.lock().unwrap().get(key).cloned())
    }
    fn write(&self, key: &str, value: &str) -> Result<(), Hitch> {
        self.0.lock().unwrap().insert(key.into(), value.into());
        Ok(())
    }
    fn forget(&self, key: &str) -> Result<(), Hitch> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}

struct Fresh(Mutex<u32>);

impl Renew for Fresh {
    fn renew(&self, _: &str) -> Result<Renewed, Hitch> {
        let mut given = self.0.lock().unwrap();
        *given += 1;
        Ok(Renewed {
            access: format!("access-{given}"),
            lasts: Duration::from_secs(3600),
            refresh: None,
        })
    }
}

fn drive(at: DriveAt, home: &std::path::Path) -> Drive {
    let kept = Kept(Mutex::new(HashMap::from([(
        "google".to_string(),
        "refresh".to_string(),
    )])));
    let tokens = Tokens::new(
        Arc::new(kept),
        Arc::new(Fresh(Mutex::new(0))),
        "google",
        home,
    );
    Drive::new(Arc::new(tokens), at).unwrap()
}

const ROOT_FOUND: &str = r#"{"files":[{"id":"R"}]}"#;

fn listing(files: &str) -> String {
    format!(r#"{{"files":[{files}]}}"#)
}

#[test]
fn every_file_tisty_made_is_named_by_its_path_under_its_own_folder() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said(
            "200 OK",
            &listing(
                r#"{"id":"F","name":"docs","parents":["R"],"mimeType":"application/vnd.google-apps.folder"},
                   {"id":"A","name":"a.md","parents":["F"],"mimeType":"text/markdown","size":"5","md5Checksum":"ab","version":"3"},
                   {"id":"T","name":"tisty.toml","parents":["R"],"mimeType":"text/plain","size":"2","md5Checksum":"cd","version":"1"},
                   {"id":"X","name":"elsewhere.md","parents":["Q"],"mimeType":"text/markdown","size":"1","md5Checksum":"ef","version":"1"}"#,
            ),
        ),
        said("200 OK", &listing("")),
    ]);
    let drive = drive(at, home.path());

    let all = drive.list("").unwrap();
    let docs = drive.list("docs").unwrap();

    assert_eq!(
        all,
        vec![
            Seen {
                name: "docs/a.md".into(),
                bytes: 5,
                hash: "md5:ab".into(),
                revision: "A.3".into()
            },
            Seen {
                name: "tisty.toml".into(),
                bytes: 2,
                hash: "md5:cd".into(),
                revision: "T.1".into()
            },
        ]
    );
    assert!(docs.is_empty());
    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 3, "the root was looked up twice");
    assert!(asked[0].line.contains("q=name"), "{}", asked[0].line);
    assert!(
        asked[0].head.contains("authorization: Bearer access-1")
            || asked[0].head.contains("Authorization: Bearer access-1")
    );
}

#[test]
fn of_two_files_with_one_name_the_oldest_is_the_one() {
    let home = tempfile::tempdir().unwrap();
    let (at, _) = google(vec![
        said("200 OK", ROOT_FOUND),
        said(
            "200 OK",
            &listing(
                r#"{"id":"OLD","name":"a.md","parents":["R"],"size":"1","md5Checksum":"aa","version":"1"},
                   {"id":"NEW","name":"a.md","parents":["R"],"size":"1","md5Checksum":"bb","version":"1"}"#,
            ),
        ),
    ]);

    let shown = drive(at, home.path()).list("").unwrap();

    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].revision, "OLD.1");
}

#[test]
fn a_drive_without_the_tisty_folder_gets_one() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", &listing("")),
        said("200 OK", r#"{"id":"R","name":"Tisty"}"#),
        said("200 OK", &listing("")),
    ]);

    assert!(drive(at, home.path()).list("").unwrap().is_empty());

    let asked = asked.join().unwrap();
    assert!(
        asked[1].line.starts_with("POST /drive/v3/files"),
        "{}",
        asked[1].line
    );
    let made = String::from_utf8_lossy(&asked[1].body);
    assert!(
        made.contains(r#""name":"Tisty""#) && made.contains("vnd.google-apps.folder"),
        "{made}"
    );
}

#[test]
fn writing_over_a_file_that_was_expected_absent_is_refused_before_anything_is_sent() {
    let home = tempfile::tempdir().unwrap();
    let local = home.path().join("a.md");
    std::fs::write(&local, b"hello").unwrap();
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said(
            "200 OK",
            &listing(
                r#"{"id":"A","name":"a.md","parents":["R"],"size":"1","md5Checksum":"aa","version":"1"}"#,
            ),
        ),
    ]);
    let drive = drive(at, home.path());

    let refused = drive.put("a.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert_eq!(
        asked.join().unwrap().len(),
        2,
        "something was sent after all"
    );
}

#[test]
fn a_revision_that_moved_on_is_refused_and_a_missing_one_is_missing() {
    let home = tempfile::tempdir().unwrap();
    let local = home.path().join("a.md");
    std::fs::write(&local, b"hello").unwrap();
    let file =
        r#"{"id":"A","name":"a.md","parents":["R"],"size":"1","md5Checksum":"aa","version":"7"}"#;
    let (at, _) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(file)),
        said("200 OK", &listing("")),
    ]);
    let drive = drive(at, home.path());

    let moved = drive.put("a.md", &local, Expect::Revision("A.6".into()));
    let gone = drive.put("a.md", &local, Expect::Revision("A.7".into()));

    assert_eq!(moved, Err(Hitch::Changed("a.md".into())));
    assert_eq!(gone, Err(Hitch::Missing("a.md".into())));
}

#[test]
fn a_new_file_gets_its_folders_and_lands_with_the_fingerprint_tisty_computes() {
    let home = tempfile::tempdir().unwrap();
    let local = home.path().join("a.md");
    std::fs::write(&local, b"hello").unwrap();
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("200 OK", r#"{"id":"F","name":"docs"}"#),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["F"],"size":"5","md5Checksum":"5d41402abc4b2a76b9719d911017c592","version":"1"}"#,
        ),
    ]);
    let drive = drive(at, home.path());

    let landed = drive.put("docs/a.md", &local, Expect::Absent).unwrap();

    assert_eq!(landed.name, "docs/a.md");
    assert_eq!(landed.revision, "A.1");
    assert_eq!(landed.hash, drive.hash_of(&local).unwrap());
    let asked = asked.join().unwrap();
    assert!(String::from_utf8_lossy(&asked[2].body).contains(r#""name":"docs""#));
    assert!(
        asked[3].line.starts_with("POST /upload/drive/v3/files"),
        "{}",
        asked[3].line
    );
    assert!(asked[3].line.contains("uploadType=multipart"));
    let sent = String::from_utf8_lossy(&asked[3].body);
    assert!(
        sent.contains(r#""parents":["F"]"#) && sent.contains("hello"),
        "{sent}"
    );
}

#[test]
fn a_big_file_goes_up_in_parts_through_a_session() {
    let home = tempfile::tempdir().unwrap();
    let local = home.path().join("big.bin");
    std::fs::write(&local, b"0123456789").unwrap();
    let (at, asked) = google_at(|base| {
        vec![
            said("200 OK", ROOT_FOUND),
            said("200 OK", &listing("")),
            Said {
                status: "200 OK",
                headers: format!("Location: {base}/upload/session\r\n"),
                body: String::new(),
            },
            said("308 Resume Incomplete", ""),
            said("308 Resume Incomplete", ""),
            said(
                "200 OK",
                r#"{"id":"B","name":"big.bin","parents":["R"],"size":"10","md5Checksum":"x","version":"1"}"#,
            ),
        ]
    });
    let drive = drive(at, home.path()).with_chunk(4);

    let landed = drive.put("big.bin", &local, Expect::Absent).unwrap();

    assert_eq!(landed.revision, "B.1");
    let asked = asked.join().unwrap();
    assert!(
        asked[2].line.contains("uploadType=resumable"),
        "{}",
        asked[2].line
    );
    let ranges: Vec<String> = asked[3..]
        .iter()
        .map(|one| {
            one.head
                .lines()
                .find(|line| line.to_ascii_lowercase().starts_with("content-range"))
                .unwrap()
                .to_string()
        })
        .collect();
    assert!(ranges[0].ends_with("bytes 0-3/10"), "{ranges:?}");
    assert!(ranges[1].ends_with("bytes 4-7/10"), "{ranges:?}");
    assert!(ranges[2].ends_with("bytes 8-9/10"), "{ranges:?}");
    let parts: Vec<&[u8]> = asked[3..].iter().map(|one| one.body.as_slice()).collect();
    assert_eq!(parts, [&b"0123"[..], &b"4567"[..], &b"89"[..]]);
}

#[test]
fn what_google_refuses_becomes_what_tisty_knows() {
    let home = tempfile::tempdir().unwrap();
    let local = home.path().join("a.md");
    std::fs::write(&local, b"hello").unwrap();
    let full = r#"{"error":{"message":"The user's Drive storage quota has been exceeded.","errors":[{"reason":"storageQuotaExceeded"}]}}"#;
    let (at, _) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("403 Forbidden", full),
        said("200 OK", &listing("")),
        Said {
            status: "429 Too Many Requests",
            headers: "Retry-After: 9\r\n".into(),
            body: String::new(),
        },
    ]);
    let drive = drive(at, home.path());

    assert_eq!(drive.put("a.md", &local, Expect::Any), Err(Hitch::Full));
    assert_eq!(
        drive.put("a.md", &local, Expect::Any),
        Err(Hitch::Limited {
            wait: Duration::from_secs(9)
        })
    );
}

#[test]
fn a_token_turned_down_is_renewed_once_and_a_second_refusal_means_reconnecting() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("401 Unauthorized", ""),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("401 Unauthorized", ""),
        said("401 Unauthorized", ""),
    ]);
    let drive = drive(at, home.path());

    assert!(drive.list("").unwrap().is_empty());
    assert_eq!(drive.list(""), Err(Hitch::Lost));

    let asked = asked.join().unwrap();
    let bearer = |at: usize| {
        asked[at]
            .head
            .lines()
            .find(|one| one.to_ascii_lowercase().starts_with("authorization"))
            .unwrap()
            .to_string()
    };
    assert!(bearer(0).ends_with("access-1") && bearer(1).ends_with("access-2"));
}

#[test]
fn a_download_starts_where_it_was_asked_and_a_vanished_file_is_missing_by_name() {
    let home = tempfile::tempdir().unwrap();
    let file =
        r#"{"id":"A","name":"a.md","parents":["R"],"size":"5","md5Checksum":"aa","version":"2"}"#;
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(file)),
        said("206 Partial Content", "lo"),
        said("200 OK", &listing(file)),
        said("404 Not Found", r#"{"error":{"message":"File not found"}}"#),
    ]);
    let drive = drive(at, home.path());
    let mut into = Vec::new();

    let got = drive.fetch("a.md", 3, &mut into).unwrap();
    let gone = drive.fetch("a.md", 0, &mut Vec::new());

    assert_eq!(into, b"lo");
    assert_eq!(got.revision, "A.2");
    assert_eq!(gone, Err(Hitch::Missing("a.md".into())));
    let asked = asked.join().unwrap();
    assert!(
        asked[2].line.contains("/files/A?alt=media"),
        "{}",
        asked[2].line
    );
    assert!(
        asked[2]
            .head
            .to_ascii_lowercase()
            .contains("range: bytes=3-"),
        "{}",
        asked[2].head
    );
}

#[test]
fn deleting_takes_every_copy_of_a_name_and_respects_the_revision() {
    let home = tempfile::tempdir().unwrap();
    let two = r#"{"id":"A1","name":"a.md","parents":["R"],"version":"1"},{"id":"A2","name":"a.md","parents":["R"],"version":"1"}"#;
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(two)),
        said("200 OK", &listing(two)),
        said("204 No Content", ""),
        said("404 Not Found", ""),
        said("200 OK", &listing("")),
    ]);
    let drive = drive(at, home.path());

    assert_eq!(
        drive.delete("a.md", Some("A1.0")),
        Err(Hitch::Changed("a.md".into()))
    );
    assert_eq!(drive.delete("a.md", Some("A1.1")), Ok(()));
    assert_eq!(
        drive.delete("a.md", None),
        Err(Hitch::Missing("a.md".into()))
    );

    let asked = asked.join().unwrap();
    assert!(asked[3].line.starts_with("DELETE /drive/v3/files/A1"));
    assert!(asked[4].line.starts_with("DELETE /drive/v3/files/A2"));
}

#[test]
fn a_quiet_drive_costs_one_ask_and_any_change_brings_the_whole_listing() {
    let home = tempfile::tempdir().unwrap();
    let file =
        r#"{"id":"A","name":"a.md","parents":["R"],"size":"1","md5Checksum":"aa","version":"1"}"#;
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(file)),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"10"}"#),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"A"}],"nextPageToken":"11"}"#,
        ),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"12"}"#),
        said("200 OK", &listing(file)),
    ]);
    let drive = drive(at, home.path());

    let first = drive.changes(None).unwrap();
    let quiet = drive.changes(Some("10")).unwrap();
    let stirred = drive.changes(Some("10")).unwrap();

    assert!(
        matches!(first, Changes::Whole { ref seen, ref cursor } if seen.len() == 1 && cursor == "10")
    );
    assert_eq!(
        quiet,
        Changes::Since {
            changed: Vec::new(),
            gone: Vec::new(),
            cursor: "10".into()
        }
    );
    assert!(matches!(stirred, Changes::Whole { ref cursor, .. } if cursor == "12"));
    let asked = asked.join().unwrap();
    assert!(asked[4].line.contains("pageToken=10"));
    assert!(asked[5].line.contains("pageToken=11"));
}

#[test]
fn the_account_is_checked_and_another_one_is_named_and_refused() {
    let home = tempfile::tempdir().unwrap();
    let me = r#"{"user":{"emailAddress":"mario@example.com","permissionId":"P1"}}"#;
    let (at, _) = google(vec![said("200 OK", me), said("200 OK", me)]);
    let drive = drive(at, home.path());

    assert_eq!(
        drive.verify("P1"),
        Ok(Account {
            id: "P1".into(),
            email: "mario@example.com".into()
        })
    );
    assert_eq!(
        drive.verify("P2"),
        Err(Hitch::Elsewhere {
            found: "mario@example.com".into()
        })
    );
}

#[test]
fn the_fingerprint_of_a_file_here_is_the_md5_google_publishes() {
    let home = tempfile::tempdir().unwrap();
    let local = home.path().join("hello.txt");
    std::fs::write(&local, b"hello").unwrap();
    let (at, _) = google(vec![]);

    assert_eq!(
        drive(at, home.path()).hash_of(&local).unwrap(),
        "md5:5d41402abc4b2a76b9719d911017c592"
    );
}
