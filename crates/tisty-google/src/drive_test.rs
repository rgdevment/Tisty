use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use tisty_carrier::{Renew, Renewed, Secrets, Tokens};

use super::{Account, Drive, DriveAt, wire};
use tisty_carrier::{Changes, Expect, Hitch, Remote, Seen};

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
                if one.status == "DROP" {
                    return asked;
                }
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

fn header(asked: &Asked, name: &str) -> String {
    asked
        .head
        .lines()
        .find(|one| one.to_ascii_lowercase().starts_with(name))
        .unwrap_or_default()
        .to_string()
}

fn written(home: &tempfile::TempDir, body: &[u8]) -> std::path::PathBuf {
    let local = home.path().join("local.bin");
    std::fs::write(&local, body).unwrap();
    local
}

const A: &str =
    r#"{"id":"A","name":"a.md","parents":["R"],"size":"1","sha256Checksum":"aa","version":"1"}"#;

#[test]
fn every_file_tisty_made_is_named_by_its_path_under_its_own_folder() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said(
            "200 OK",
            &listing(
                r#"{"id":"F","name":"docs","parents":["R"],"mimeType":"application/vnd.google-apps.folder"},
                   {"id":"D","name":"a.md","parents":["F"],"size":"5","sha256Checksum":"ab","version":"3"},
                   {"id":"T","name":"tisty.toml","parents":["R"],"size":"2","sha256Checksum":"cd","version":"1"},
                   {"id":"X","name":"elsewhere.md","parents":["Q"],"size":"1","sha256Checksum":"ef","version":"1"}"#,
            ),
        ),
    ]);
    let drive = drive(at, home.path());

    let all = drive.list("").unwrap();

    assert_eq!(
        all,
        vec![
            Seen {
                name: "docs/a.md".into(),
                bytes: 5,
                hash: "sha256:ab".into(),
                revision: "D.3".into()
            },
            Seen {
                name: "tisty.toml".into(),
                bytes: 2,
                hash: "sha256:cd".into(),
                revision: "T.1".into()
            },
        ]
    );
    let asked = asked.join().unwrap();
    assert!(asked[0].line.contains("q=name"), "{}", asked[0].line);
    assert!(header(&asked[0], "authorization").ends_with("access-1"));
}

#[test]
fn of_two_files_with_one_name_the_oldest_is_the_one() {
    let home = tempfile::tempdir().unwrap();
    let (at, _) = google(vec![
        said("200 OK", ROOT_FOUND),
        said(
            "200 OK",
            &listing(
                r#"{"id":"OLD","name":"a.md","parents":["R"],"version":"1"},
                   {"id":"NEW","name":"a.md","parents":["R"],"version":"1"}"#,
            ),
        ),
    ]);

    let shown = drive(at, home.path()).list("").unwrap();

    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].revision, "OLD.1");
}

#[test]
fn what_two_machines_wrote_in_tisty_folders_of_their_own_is_all_found() {
    let home = tempfile::tempdir().unwrap();
    let (at, _) = google(vec![
        said("200 OK", r#"{"files":[{"id":"R1"},{"id":"R2"}]}"#),
        said(
            "200 OK",
            &listing(
                r#"{"id":"A","name":"a.md","parents":["R1"],"version":"1"},
                   {"id":"B","name":"b.md","parents":["R2"],"version":"1"}"#,
            ),
        ),
    ]);

    let shown = drive(at, home.path()).list("").unwrap();

    assert_eq!(
        shown
            .iter()
            .map(|one| one.name.as_str())
            .collect::<Vec<_>>(),
        ["a.md", "b.md"]
    );
}

#[test]
fn a_drive_without_the_tisty_folder_gets_one_and_every_listing_asks_for_it_again() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", &listing("")),
        said("200 OK", r#"{"id":"R","name":"Tisty"}"#),
        said("200 OK", &listing("")),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
    ]);
    let drive = drive(at, home.path());

    assert!(drive.list("").unwrap().is_empty());
    assert!(drive.list("").unwrap().is_empty());

    let asked = asked.join().unwrap();
    assert!(
        asked[1].line.starts_with("POST /drive/v3/files"),
        "{}",
        asked[1].line
    );
    let made = String::from_utf8_lossy(&asked[1].body);
    assert!(made.contains(r#""name":"Tisty""#), "{made}");
    assert!(
        asked[3].line.contains("q=name"),
        "a Tisty folder put in the bin would be written into"
    );
}

#[test]
fn writing_over_a_file_expected_absent_is_refused_before_anything_is_sent() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(A)),
        said("200 OK", A),
    ]);

    let refused = drive(at, home.path()).put("a.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert_eq!(
        asked.join().unwrap().len(),
        3,
        "something was sent after all"
    );
}

#[test]
fn a_file_another_machine_made_since_the_listing_is_found_before_writing() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("200 OK", &listing(A)),
    ]);

    let refused = drive(at, home.path()).put("a.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert!(asked.join().unwrap()[2].line.contains("q=name"));
}

#[test]
fn a_file_another_machine_replaced_is_found_by_its_name_and_never_doubled() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let replaced = r#"{"id":"A3","name":"a.md","parents":["R"],"version":"1"}"#;
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(A)),
        said("404 Not Found", ""),
        said("200 OK", &listing(replaced)),
    ]);

    let refused = drive(at, home.path()).put("a.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert_eq!(asked.join().unwrap().len(), 4, "a second a.md was uploaded");
}

#[test]
fn a_file_renamed_in_drive_is_never_written_over_under_its_old_name() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let renamed = r#"{"id":"A","name":"b.md","parents":["R"],"version":"2"}"#;
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(A)),
        said("200 OK", renamed),
        said("200 OK", &listing("")),
        said(
            "200 OK",
            r#"{"id":"N","name":"a.md","parents":["R"],"version":"1"}"#,
        ),
    ]);

    let landed = drive(at, home.path())
        .put("a.md", &local, Expect::Any)
        .unwrap();

    assert_eq!(landed.revision, "N.1");
    let asked = asked.join().unwrap();
    assert!(
        asked[4].line.starts_with("POST /upload/drive/v3/files"),
        "{}",
        asked[4].line
    );
}

#[test]
fn a_folder_another_machine_made_is_found_and_what_is_in_it_is_never_doubled() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("200 OK", r#"{"files":[{"id":"H"}]}"#),
        said("200 OK", r#"{"files":[{"id":"B"}]}"#),
        said(
            "200 OK",
            &listing(r#"{"id":"X","name":"x","parents":["B"],"version":"1"}"#),
        ),
    ]);

    let refused = drive(at, home.path()).put("HELD/ab/x", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("HELD/ab/x".into())));
    let asked = asked.join().unwrap();
    assert!(
        asked[3].line.contains("H%27+in+parents") || asked[3].line.contains("H%27%20in%20parents"),
        "{}",
        asked[3].line
    );
}

#[test]
fn a_revision_that_moved_on_is_refused_and_a_missing_one_is_missing() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let file = r#"{"id":"A","name":"a.md","parents":["R"],"version":"7"}"#;
    let (at, _) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(file)),
        said("200 OK", file),
        said("404 Not Found", ""),
        said("200 OK", &listing("")),
    ]);
    let drive = drive(at, home.path());

    assert_eq!(
        drive.put("a.md", &local, Expect::Revision("A.6".into())),
        Err(Hitch::Changed("a.md".into()))
    );
    assert_eq!(
        drive.put("a.md", &local, Expect::Revision("A.7".into())),
        Err(Hitch::Missing("a.md".into()))
    );
}

#[test]
fn a_new_file_gets_its_folders_and_lands_with_a_boundary_of_its_own() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello\r\n--tisty-part\r\nstill hello");
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("200 OK", &listing("")),
        said("200 OK", &listing("")),
        said("200 OK", r#"{"id":"F","name":"docs"}"#),
        said(
            "200 OK",
            r#"{"id":"D","name":"a.md","parents":["F"],"size":"5","sha256Checksum":"x","version":"1"}"#,
        ),
    ]);

    let landed = drive(at, home.path())
        .put("docs/a.md", &local, Expect::Absent)
        .unwrap();

    assert_eq!(landed.revision, "D.1");
    let asked = asked.join().unwrap();
    assert!(String::from_utf8_lossy(&asked[4].body).contains(r#""name":"docs""#));
    assert!(
        asked[5].line.contains("uploadType=multipart"),
        "{}",
        asked[5].line
    );
    let kind = header(&asked[5], "content-type");
    let boundary = kind.split("boundary=").nth(1).unwrap().trim();
    assert_ne!(boundary, "tisty-part");
    assert!(boundary.len() > 20, "{boundary}");
    let sent = String::from_utf8_lossy(&asked[5].body);
    assert!(
        sent.contains(r#""parents":["F"]"#) && sent.contains("still hello"),
        "{sent}"
    );
}

#[test]
fn a_big_file_goes_up_in_parts_from_where_google_says_it_kept() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"0123456789");
    let (at, asked) = google_at(|base| {
        vec![
            said("200 OK", ROOT_FOUND),
            said("200 OK", &listing("")),
            said("200 OK", &listing("")),
            Said {
                status: "200 OK",
                headers: format!("Location: {base}/upload/session\r\n"),
                body: String::new(),
            },
            Said {
                status: "308 Resume Incomplete",
                headers: "Range: bytes=0-1\r\n".into(),
                body: String::new(),
            },
            said("DROP", ""),
            Said {
                status: "308 Resume Incomplete",
                headers: "Range: bytes=0-5\r\n".into(),
                body: String::new(),
            },
            said(
                "200 OK",
                r#"{"id":"B","name":"local.bin","parents":["R"],"size":"10","sha256Checksum":"x","version":"1"}"#,
            ),
        ]
    });
    let drive = drive(at, home.path()).with_chunk(4);

    let landed = drive.put("big.bin", &local, Expect::Absent).unwrap();

    assert_eq!(landed.revision, "B.1");
    let asked = asked.join().unwrap();
    assert!(
        asked[3].line.contains("uploadType=resumable"),
        "{}",
        asked[3].line
    );
    let ranges: Vec<String> = asked[4..]
        .iter()
        .map(|one| header(one, "content-range"))
        .collect();
    assert!(ranges[0].ends_with("bytes 0-3/10"), "{ranges:?}");
    assert!(
        ranges[1].ends_with("bytes 2-5/10"),
        "it went on from what it sent, not from what Google kept: {ranges:?}"
    );
    assert!(ranges[2].ends_with("bytes */10"), "{ranges:?}");
    assert!(ranges[3].ends_with("bytes 6-9/10"), "{ranges:?}");
    assert_eq!(asked[7].body, b"6789");
}

#[test]
fn what_google_refuses_becomes_what_tisty_knows() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let reason =
        |one: &str| format!(r#"{{"error":{{"message":"m","errors":[{{"reason":"{one}"}}]}}}}"#);
    let (at, _) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("200 OK", &listing("")),
        said("403 Forbidden", &reason("storageQuotaExceeded")),
        said("200 OK", &listing("")),
        Said {
            status: "429 Too Many Requests",
            headers: "Retry-After: 9\r\n".into(),
            body: String::new(),
        },
        said("200 OK", &listing("")),
        said("403 Forbidden", &reason("dailyLimitExceeded")),
        said("200 OK", &listing("")),
        said("403 Forbidden", &reason("appNotAuthorizedToFile")),
    ]);
    let drive = drive(at, home.path());

    assert_eq!(drive.put("a.md", &local, Expect::Any), Err(Hitch::Full));
    assert_eq!(
        drive.put("a.md", &local, Expect::Any),
        Err(Hitch::Limited {
            wait: Duration::from_secs(9)
        })
    );
    assert_eq!(
        drive.put("a.md", &local, Expect::Any),
        Err(Hitch::Limited {
            wait: Duration::from_secs(3600)
        })
    );
    assert!(matches!(
        drive.put("a.md", &local, Expect::Any),
        Err(Hitch::Broke(_))
    ));
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
    assert!(header(&asked[0], "authorization").ends_with("access-1"));
    assert!(header(&asked[1], "authorization").ends_with("access-2"));
}

struct Unwritable;

impl Write for Unwritable {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("the disk is full"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_download_starts_where_asked_a_full_disk_is_no_offline_and_a_vanished_file_is_missing() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(A)),
        said("200 OK", A),
        said("206 Partial Content", "lo"),
        said("200 OK", A),
        said("200 OK", "hello"),
        said("404 Not Found", ""),
        said("200 OK", &listing("")),
    ]);
    let drive = drive(at, home.path());
    let mut into = Vec::new();

    let got = drive.fetch("a.md", 3, &mut into).unwrap();
    let full = drive.fetch("a.md", 0, &mut Unwritable);
    let gone = drive.fetch("a.md", 0, &mut Vec::new());

    assert_eq!(into, b"lo");
    assert_eq!(got.revision, "A.1");
    assert!(matches!(full, Err(Hitch::Broke(_))), "{full:?}");
    assert_eq!(gone, Err(Hitch::Missing("a.md".into())));
    let asked = asked.join().unwrap();
    assert!(
        asked[3].line.contains("/files/A?alt=media"),
        "{}",
        asked[3].line
    );
    assert!(
        header(&asked[3], "range").ends_with("bytes=3-"),
        "{}",
        asked[3].head
    );
}

#[test]
fn deleting_takes_every_copy_of_a_name_and_respects_the_revision() {
    let home = tempfile::tempdir().unwrap();
    let one = r#"{"id":"A1","name":"a.md","parents":["R"],"version":"1"}"#;
    let two = format!(r#"{one},{{"id":"A2","name":"a.md","parents":["R"],"version":"1"}}"#);
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(&two)),
        said("200 OK", one),
        said("200 OK", one),
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
    assert!(asked[4].line.starts_with("DELETE /drive/v3/files/A1"));
    assert!(asked[5].line.starts_with("DELETE /drive/v3/files/A2"));
}

#[test]
fn what_this_machine_wrote_is_no_news_and_what_another_wrote_brings_the_listing() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let a2 =
        r#"{"id":"A","name":"a.md","parents":["R"],"size":"5","sha256Checksum":"x","version":"2"}"#;
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(A)),
        said("200 OK", A),
        said("200 OK", a2),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"A","file":{"id":"A","version":"2"}}],"newStartPageToken":"11"}"#,
        ),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"Z","file":{"id":"Z","version":"1"}}],"nextPageToken":"12"}"#,
        ),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"13"}"#),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(a2)),
    ]);
    let drive = drive(at, home.path());

    let first = drive.changes(None).unwrap();
    drive
        .put("a.md", &local, Expect::Revision("A.1".into()))
        .unwrap();
    let own = drive.changes(Some("10")).unwrap();
    let theirs = drive.changes(Some("11")).unwrap();

    assert!(matches!(first, Changes::Whole { ref cursor, .. } if cursor == "10"));
    assert_eq!(
        own,
        Changes::Since {
            changed: Vec::new(),
            gone: Vec::new(),
            cursor: "11".into()
        },
        "this machine's own write was taken for someone else's"
    );
    assert!(matches!(theirs, Changes::Whole { ref cursor, .. } if cursor == "13"));
    let asked = asked.join().unwrap();
    assert_eq!(
        asked
            .iter()
            .filter(|one| one.line.contains("q=trashed"))
            .count(),
        2
    );
}

#[test]
fn a_cursor_another_process_moved_brings_the_listing_back_in_step() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"30"}"#),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(A)),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    let quiet = drive.changes(Some("20")).unwrap();

    assert!(matches!(quiet, Changes::Since { .. }));
    assert_eq!(
        asked
            .join()
            .unwrap()
            .iter()
            .filter(|one| one.line.contains("q=trashed"))
            .count(),
        2,
        "a listing older than the cursor it is used at was kept"
    );
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
fn the_fingerprint_of_a_file_here_is_the_sha256_google_publishes() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, _) = google(vec![]);

    assert_eq!(
        drive(at, home.path()).hash_of(&local).unwrap(),
        "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn writing_many_files_in_one_round_lists_the_drive_once_and_asks_each_file_alone() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let b = r#"{"id":"B","name":"b.md","parents":["R"],"version":"1"}"#;
    let a2 = r#"{"id":"A","name":"a.md","parents":["R"],"version":"2"}"#;
    let b2 = r#"{"id":"B","name":"b.md","parents":["R"],"version":"2"}"#;
    let (at, asked) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(&format!("{A},{b}"))),
        said("200 OK", A),
        said("200 OK", a2),
        said("200 OK", b),
        said("200 OK", b2),
        said("200 OK", a2),
    ]);
    let drive = drive(at, home.path());

    drive
        .put("a.md", &local, Expect::Revision("A.1".into()))
        .unwrap();
    drive
        .put("b.md", &local, Expect::Revision("B.1".into()))
        .unwrap();
    let again = drive.about("a.md").unwrap().unwrap();

    assert_eq!(
        again.revision, "A.2",
        "what this machine just wrote was forgotten"
    );
    let asked = asked.join().unwrap();
    assert_eq!(
        asked
            .iter()
            .filter(|one| one.line.contains("q=trashed"))
            .count(),
        1
    );
    assert!(
        asked[3].line.starts_with("PATCH /upload/drive/v3/files/A"),
        "{}",
        asked[3].line
    );
}

#[test]
fn a_name_with_a_quote_is_asked_for_as_written() {
    assert_eq!(wire::quoted("it's"), r"it\'s");
    assert_eq!(wire::quoted(r"a\b"), r"a\\b");
}

#[test]
fn a_change_another_machine_made_is_news_even_when_the_listing_already_holds_it() {
    let home = tempfile::tempdir().unwrap();
    let theirs = r#"{"id":"F","name":"f.md","parents":["R"],"version":"4"}"#;
    let (at, _) = google(vec![
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(theirs)),
        said("200 OK", theirs),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"F","file":{"id":"F","version":"4"}}],"newStartPageToken":"21"}"#,
        ),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(theirs)),
    ]);
    let drive = drive(at, home.path());

    drive.about("f.md").unwrap();
    let after_restart = drive.changes(Some("20")).unwrap();

    assert!(
        matches!(after_restart, Changes::Whole { ref seen, .. } if seen.len() == 1),
        "another machine's file was taken for this one's: {after_restart:?}"
    );
}

#[test]
fn a_listing_taken_for_one_folder_never_hides_what_changed_elsewhere() {
    let home = tempfile::tempdir().unwrap();
    let theirs = r#"{"id":"N","name":"note.md","parents":["R"],"version":"2"}"#;
    let (at, _) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing("")),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(theirs)),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"N","file":{"id":"N","version":"2"}}],"newStartPageToken":"11"}"#,
        ),
        said("200 OK", ROOT_FOUND),
        said("200 OK", &listing(theirs)),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    drive.list("attachments").unwrap();
    let next = drive.changes(Some("10")).unwrap();

    assert!(matches!(next, Changes::Whole { .. }), "{next:?}");
}

#[test]
fn an_upload_google_finished_as_the_line_dropped_is_taken_as_landed() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"0123456789");
    let (at, _) = google_at(|base| {
        vec![
            said("200 OK", ROOT_FOUND),
            said("200 OK", &listing("")),
            said("200 OK", &listing("")),
            Said {
                status: "200 OK",
                headers: format!("Location: {base}/upload/session\r\n"),
                body: String::new(),
            },
            Said {
                status: "308 Resume Incomplete",
                headers: "Range: bytes=0-3\r\n".into(),
                body: String::new(),
            },
            Said {
                status: "308 Resume Incomplete",
                headers: "Range: bytes=0-7\r\n".into(),
                body: String::new(),
            },
            said("DROP", ""),
            said(
                "201 Created",
                r#"{"id":"B","name":"big.bin","parents":["R"],"size":"10","version":"1"}"#,
            ),
        ]
    });
    let drive = drive(at, home.path()).with_chunk(4);

    let landed = drive.put("big.bin", &local, Expect::Absent);

    assert_eq!(landed.map(|one| one.revision), Ok("B.1".into()));
}
