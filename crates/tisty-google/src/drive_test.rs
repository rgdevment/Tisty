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

const ROOT: &str = r#"{"id":"R","name":"Tisty","mimeType":"application/vnd.google-apps.folder","parents":["myroot"]}"#;

fn whole(files: &str) -> String {
    match files.trim().is_empty() {
        true => listing(ROOT),
        false => listing(&format!("{ROOT},{files}")),
    }
}

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
    let (at, asked) = google(vec![said(
        "200 OK",
        &whole(
            r#"{"id":"F","name":"docs","parents":["R"],"mimeType":"application/vnd.google-apps.folder"},
                   {"id":"D","name":"a.md","parents":["F"],"size":"5","sha256Checksum":"ab","version":"3"},
                   {"id":"T","name":"tisty.toml","parents":["R"],"size":"2","sha256Checksum":"cd","version":"1"},
                   {"id":"X","name":"elsewhere.md","parents":["Q"],"size":"1","sha256Checksum":"ef","version":"1"}"#,
        ),
    )]);
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
    assert_eq!(
        asked.len(),
        1,
        "the Tisty folder was searched apart from the listing"
    );
    assert!(header(&asked[0], "authorization").ends_with("access-1"));
}

#[test]
fn of_two_files_with_one_name_the_oldest_is_the_one() {
    let home = tempfile::tempdir().unwrap();
    let (at, _) = google(vec![said(
        "200 OK",
        &whole(
            r#"{"id":"OLD","name":"a.md","parents":["R"],"version":"1"},
                   {"id":"NEW","name":"a.md","parents":["R"],"version":"1"}"#,
        ),
    )]);

    let shown = drive(at, home.path()).list("").unwrap();

    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].revision, "OLD.1");
}

#[test]
fn what_two_machines_wrote_in_tisty_folders_of_their_own_is_all_found() {
    let home = tempfile::tempdir().unwrap();
    let (at, _) = google(vec![said(
        "200 OK",
        &listing(
            r#"{"id":"R1","name":"Tisty","mimeType":"application/vnd.google-apps.folder","parents":["myroot"]},
                   {"id":"R2","name":"Tisty","mimeType":"application/vnd.google-apps.folder","parents":["P"]},
                   {"id":"A","name":"a.md","parents":["R1"],"version":"1"},
                   {"id":"B","name":"b.md","parents":["R2"],"version":"1"}"#,
        ),
    )]);

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
fn reading_a_drive_without_the_tisty_folder_makes_nothing_and_writing_makes_it_once() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", &listing("")),
        said("200 OK", &listing("")),
        said("200 OK", r#"{"id":"R","name":"Tisty","version":"1"}"#),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"version":"1"}"#,
        ),
    ]);
    let drive = drive(at, home.path());

    assert!(drive.list("").unwrap().is_empty());
    drive.put("a.md", &local, Expect::Absent).unwrap();

    let asked = asked.join().unwrap();
    assert!(
        asked[1].line.contains("trashed+%3D+true")
            || asked[1].line.contains("trashed%20%3D%20true"),
        "{}",
        asked[1].line
    );
    assert!(
        asked[2].line.starts_with("POST /drive/v3/files"),
        "{}",
        asked[2].line
    );
    assert!(String::from_utf8_lossy(&asked[2].body).contains(r#""name":"Tisty""#));
    assert!(String::from_utf8_lossy(&asked[3].body).contains(r#""parents":["R"]"#));
}

#[test]
fn a_tisty_folder_put_in_the_bin_stops_the_round_instead_of_reading_an_empty_drive() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", &listing("")),
        said("200 OK", r#"{"files":[{"id":"R"}]}"#),
    ]);

    let stopped = drive(at, home.path()).list("");

    assert!(matches!(stopped, Err(Hitch::Broke(_))), "{stopped:?}");
    assert_eq!(
        asked.join().unwrap().len(),
        2,
        "a new Tisty folder was made"
    );
}

#[test]
fn a_tisty_folder_moved_inside_another_of_its_folders_is_not_taken_for_a_root() {
    let home = tempfile::tempdir().unwrap();
    let (at, _) = google(vec![said(
        "200 OK",
        &whole(
            r#"{"id":"F","name":"docs","parents":["R"],"mimeType":"application/vnd.google-apps.folder"},
               {"id":"T","name":"Tisty","parents":["F"],"mimeType":"application/vnd.google-apps.folder"},
               {"id":"D","name":"a.md","parents":["T"],"version":"1"}"#,
        ),
    )]);

    let shown = drive(at, home.path()).list("").unwrap();

    assert_eq!(
        shown
            .iter()
            .map(|one| one.name.as_str())
            .collect::<Vec<_>>(),
        ["docs/Tisty/a.md"]
    );
}

#[test]
fn writing_over_a_file_expected_absent_is_refused_before_anything_is_sent() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![said("200 OK", &whole(A)), said("200 OK", A)]);

    let refused = drive(at, home.path()).put("a.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert_eq!(
        asked.join().unwrap().len(),
        2,
        "something was sent after all"
    );
}

#[test]
fn a_file_another_machine_made_since_the_listing_is_found_before_writing() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", &whole("")),
        said("200 OK", &listing(A)),
    ]);

    let refused = drive(at, home.path()).put("a.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert!(asked.join().unwrap()[1].line.contains("q=name"));
}

#[test]
fn a_file_another_machine_replaced_is_found_by_its_name_and_never_doubled() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let replaced = r#"{"id":"A3","name":"a.md","parents":["R"],"version":"1"}"#;
    let (at, asked) = google(vec![
        said("200 OK", &whole(A)),
        said("404 Not Found", ""),
        said("200 OK", &listing(replaced)),
    ]);

    let refused = drive(at, home.path()).put("a.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert_eq!(asked.join().unwrap().len(), 3, "a second a.md was uploaded");
}

#[test]
fn a_file_renamed_in_drive_is_never_written_over_under_its_old_name() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let renamed = r#"{"id":"A","name":"b.md","parents":["R"],"version":"2"}"#;
    let (at, asked) = google(vec![
        said("200 OK", &whole(A)),
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
        asked[3].line.starts_with("POST /upload/drive/v3/files"),
        "{}",
        asked[3].line
    );
}

#[test]
fn a_folder_another_machine_made_is_found_and_what_is_in_it_is_never_doubled() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", &whole("")),
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
        asked[2].line.contains("H%27+in+parents") || asked[2].line.contains("H%27%20in%20parents"),
        "{}",
        asked[2].line
    );
}

#[test]
fn a_revision_that_moved_on_is_refused_and_a_missing_one_is_missing() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let file = r#"{"id":"A","name":"a.md","parents":["R"],"version":"7"}"#;
    let (at, _) = google(vec![
        said("200 OK", &whole(file)),
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
        said("200 OK", &whole("")),
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
    assert!(String::from_utf8_lossy(&asked[3].body).contains(r#""name":"docs""#));
    assert!(
        asked[4].line.contains("uploadType=multipart"),
        "{}",
        asked[4].line
    );
    let kind = header(&asked[4], "content-type");
    let boundary = kind.split("boundary=").nth(1).unwrap().trim();
    assert_ne!(boundary, "tisty-part");
    assert!(boundary.len() > 20, "{boundary}");
    let sent = String::from_utf8_lossy(&asked[4].body);
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
            said("200 OK", &whole("")),
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
        asked[2].line.contains("uploadType=resumable"),
        "{}",
        asked[2].line
    );
    let ranges: Vec<String> = asked[3..]
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
    assert_eq!(asked[6].body, b"6789");
}

#[test]
fn what_google_refuses_becomes_what_tisty_knows() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let reason =
        |one: &str| format!(r#"{{"error":{{"message":"m","errors":[{{"reason":"{one}"}}]}}}}"#);
    let (at, _) = google(vec![
        said("200 OK", &whole("")),
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
        said("200 OK", &whole("")),
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
        said("200 OK", &whole(A)),
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
        asked[2].line.contains("/files/A?alt=media"),
        "{}",
        asked[2].line
    );
    assert!(
        header(&asked[2], "range").ends_with("bytes=3-"),
        "{}",
        asked[2].head
    );
}

#[test]
fn deleting_with_a_revision_bins_the_file_and_takes_every_other_copy() {
    let home = tempfile::tempdir().unwrap();
    let one = r#"{"id":"A1","name":"a.md","parents":["R"],"version":"1"}"#;
    let two = format!(r#"{one},{{"id":"A2","name":"a.md","parents":["R"],"version":"1"}}"#);
    let (at, asked) = google(vec![
        said("200 OK", &whole(&two)),
        said("200 OK", one),
        said("200 OK", one),
        said(
            "200 OK",
            r#"{"id":"A1","name":"a.md","parents":["R"],"version":"2","trashed":true}"#,
        ),
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
    assert!(asked[3].line.starts_with("PATCH /drive/v3/files/A1"));
    assert_eq!(asked[3].body, br#"{"trashed":true}"#);
    assert!(asked[4].line.starts_with("DELETE /drive/v3/files/A2"));
}

#[test]
fn what_this_machine_wrote_is_no_news_and_what_another_wrote_comes_alone() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let a2 =
        r#"{"id":"A","name":"a.md","parents":["R"],"size":"5","sha256Checksum":"x","version":"2"}"#;
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(A)),
        said("200 OK", A),
        said("200 OK", a2),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"A","file":{"id":"A","version":"2"}}],"newStartPageToken":"11"}"#,
        ),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"Z","file":{"id":"Z","name":"z.md","parents":["R"],"size":"3","sha256Checksum":"zz","version":"1"}}],"nextPageToken":"12"}"#,
        ),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"13"}"#),
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
    assert_eq!(
        theirs,
        Changes::Since {
            changed: vec![Seen {
                name: "z.md".into(),
                bytes: 3,
                hash: "sha256:zz".into(),
                revision: "Z.1".into(),
            }],
            gone: Vec::new(),
            cursor: "13".into()
        },
        "another machine's write did not come as what changed"
    );
    let asked = asked.join().unwrap();
    assert_eq!(
        asked
            .iter()
            .filter(|one| one.line.contains("q=trashed"))
            .count(),
        1,
        "a change elsewhere listed the whole drive again"
    );
}

#[test]
fn a_cursor_another_process_moved_brings_the_listing_back_in_step() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole("")),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"30"}"#),
        said("200 OK", &whole(A)),
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
        said("200 OK", &whole(&format!("{A},{b}"))),
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
        asked[2].line.starts_with("PATCH /upload/drive/v3/files/A"),
        "{}",
        asked[2].line
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
        said("200 OK", &whole(theirs)),
        said("200 OK", theirs),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"F","file":{"id":"F","version":"4"}}],"newStartPageToken":"21"}"#,
        ),
        said("200 OK", &whole(theirs)),
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
        said("200 OK", &whole("")),
        said("200 OK", &whole(theirs)),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"N","file":{"id":"N","version":"2"}}],"newStartPageToken":"11"}"#,
        ),
        said("200 OK", &whole(theirs)),
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
            said("200 OK", &whole("")),
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

#[test]
fn a_big_transfer_is_given_hours_and_a_small_one_still_a_couple_of_minutes() {
    assert!(wire::given(8 * 1024 * 1024 * 1024) > Duration::from_secs(2 * 60 * 60));
    assert!(wire::given(0) >= Duration::from_secs(2 * 60));
}

fn kept(range: &str) -> Said {
    Said {
        status: "308 Resume Incomplete",
        headers: format!("Range: bytes=0-{range}\r\n"),
        body: String::new(),
    }
}

#[test]
fn a_long_upload_survives_cuts_and_server_errors_that_do_not_come_in_a_row() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"0123456789AB");
    let (at, asked) = google_at(|base| {
        vec![
            said("200 OK", &whole("")),
            said("200 OK", &listing("")),
            Said {
                status: "200 OK",
                headers: format!("Location: {base}/upload/session\r\n"),
                body: String::new(),
            },
            said("DROP", ""),
            kept("3"),
            kept("3"),
            said("503 Service Unavailable", ""),
            kept("7"),
            kept("7"),
            said("DROP", ""),
            kept("7"),
            said(
                "200 OK",
                r#"{"id":"B","name":"big.bin","parents":["R"],"size":"12","version":"1"}"#,
            ),
        ]
    });
    let drive = drive(at, home.path()).with_chunk(4);

    let landed = drive.put("big.bin", &local, Expect::Absent);

    assert_eq!(landed.map(|one| one.revision), Ok("B.1".into()));
    let asked = asked.join().unwrap();
    let ranges: Vec<String> = asked[3..]
        .iter()
        .map(|one| header(one, "content-range"))
        .collect();
    assert!(
        ranges[1].ends_with("bytes */12"),
        "a cut did not ask what Google kept: {ranges:?}"
    );
    assert!(
        ranges[4].ends_with("bytes */12"),
        "a 503 did not ask what Google kept: {ranges:?}"
    );
    assert!(
        ranges.last().unwrap().ends_with("bytes 8-11/12"),
        "{ranges:?}"
    );
}

#[test]
fn asking_google_what_it_kept_survives_a_cut_of_its_own() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"0123456789");
    let (at, asked) = google_at(|base| {
        vec![
            said("200 OK", &whole("")),
            said("200 OK", &listing("")),
            Said {
                status: "200 OK",
                headers: format!(
                    "Location: {base}/upload/session
"
                ),
                body: String::new(),
            },
            said("DROP", ""),
            said("DROP", ""),
            kept("3"),
            said(
                "200 OK",
                r#"{"id":"B","name":"big.bin","parents":["R"],"size":"10","version":"1"}"#,
            ),
        ]
    });
    let drive = drive(at, home.path()).with_chunk(8);

    let landed = drive.put("big.bin", &local, Expect::Absent);

    assert_eq!(landed.map(|one| one.revision), Ok("B.1".into()));
    let asked = asked.join().unwrap();
    assert!(
        header(&asked[6], "content-range").ends_with("bytes 4-9/10"),
        "{}",
        asked[6].head
    );
}

#[test]
fn a_store_split_across_two_roots_reads_a_vanished_copy_from_the_whole_listing() {
    let home = tempfile::tempdir().unwrap();
    let roots = r#"{"id":"R1","name":"Tisty","mimeType":"application/vnd.google-apps.folder","parents":["myroot"]},
                   {"id":"R2","name":"Tisty","mimeType":"application/vnd.google-apps.folder","parents":["myroot"]}"#;
    let x2 = r#"{"id":"X2","name":"x","parents":["R2"],"version":"1"}"#;
    let (at, asked) = google(vec![
        said(
            "200 OK",
            &listing(&format!(
                r#"{roots},{{"id":"X1","name":"x","parents":["R1"],"version":"1"}},{x2}"#
            )),
        ),
        said("404 Not Found", ""),
        said("200 OK", &listing(&format!("{roots},{x2}"))),
    ]);

    let found = drive(at, home.path()).about("x").unwrap();

    assert_eq!(found.map(|one| one.revision), Some("X2.1".into()));
    assert!(
        !asked.join().unwrap()[2].line.contains("name+%3D"),
        "only one root was searched"
    );
}

const CHECKED: &str = r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m1","headRevisionId":"h1","version":"1"}"#;

#[test]
fn bytes_another_machine_wrote_in_the_gap_go_back_on_top_and_the_write_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"mine");
    let (at, asked) = google(vec![
        said("200 OK", &whole(CHECKED)),
        said("200 OK", CHECKED),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m3","headRevisionId":"h3","version":"3"}"#,
        ),
        said(
            "200 OK",
            r#"{"revisions":[{"id":"h3","md5Checksum":"m3","modifiedTime":"2026-10-12T10:00:03Z"},{"id":"h1","md5Checksum":"m1","modifiedTime":"2026-10-12T10:00:01Z"},{"id":"h2","md5Checksum":"m2","modifiedTime":"2026-10-12T10:00:02Z"}]}"#,
        ),
        said("200 OK", r#"{"id":"h2"}"#),
        said("200 OK", "theirs"),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m2","headRevisionId":"h4","version":"4"}"#,
        ),
        said("204 No Content", ""),
    ]);

    let refused = drive(at, home.path()).put("a.md", &local, Expect::Revision("A.1".into()));

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    let asked = asked.join().unwrap();
    assert!(
        asked[4]
            .line
            .starts_with("PATCH /drive/v3/files/A/revisions/h2")
    );
    assert_eq!(asked[4].body, br#"{"keepForever":true}"#);
    assert!(
        asked[5]
            .line
            .starts_with("GET /drive/v3/files/A/revisions/h2?alt=media")
    );
    assert!(asked[6].line.starts_with("PATCH /upload/drive/v3/files/A"));
    let sent = String::from_utf8_lossy(&asked[6].body);
    assert!(
        sent.contains("theirs"),
        "their bytes did not go back: {sent}"
    );
    assert!(
        asked[7]
            .line
            .starts_with("DELETE /drive/v3/files/A/revisions/h2"),
        "a pin cannot come off, so the pinned revision is deleted"
    );
}

#[test]
fn a_version_that_moved_without_other_bytes_lets_the_write_stand() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"mine");
    let (at, asked) = google(vec![
        said("200 OK", &whole(CHECKED)),
        said("200 OK", CHECKED),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m3","headRevisionId":"h3","version":"3"}"#,
        ),
        said(
            "200 OK",
            r#"{"revisions":[{"id":"h1","md5Checksum":"m1"},{"id":"h3","md5Checksum":"m3"}]}"#,
        ),
    ]);

    let landed = drive(at, home.path()).put("a.md", &local, Expect::Revision("A.1".into()));

    assert_eq!(landed.map(|one| one.revision), Ok("A.3".to_string()));
    assert_eq!(asked.join().unwrap().len(), 4);
}

#[test]
fn a_deletion_that_finds_other_bytes_in_the_bin_takes_them_back_out_and_is_refused() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", &whole(CHECKED)),
        said("200 OK", CHECKED),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m2","version":"3","trashed":true}"#,
        ),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m2","version":"4"}"#,
        ),
    ]);

    let refused = drive(at, home.path()).delete("a.md", Some("A.1"));

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    let asked = asked.join().unwrap();
    assert_eq!(asked[2].body, br#"{"trashed":true}"#);
    assert_eq!(asked[3].body, br#"{"trashed":false}"#);
}

fn changed(json_list: &str) -> Vec<super::tree::Changed> {
    serde_json::from_str(json_list).unwrap()
}

fn held_tree(files: &str) -> super::tree::Tree {
    let all: Vec<super::tree::File> = serde_json::from_str(&format!("[{ROOT},{files}]")).unwrap();
    super::tree::planted(&["R".to_string()], all)
}

const DOCS: &str =
    r#"{"id":"F","name":"docs","parents":["R"],"mimeType":"application/vnd.google-apps.folder"}"#;

#[test]
fn a_file_binned_or_removed_elsewhere_comes_as_gone_and_one_added_in_new_folders_as_changed() {
    let tree = held_tree(&format!(
        r#"{DOCS},{{"id":"D","name":"a.md","parents":["F"],"version":"1"}}"#
    ));
    let roots = ["R".to_string()];
    let moved = changed(
        r#"[{"fileId":"D","file":{"id":"D","name":"a.md","parents":["F"],"version":"2","trashed":true}},
            {"fileId":"X","file":{"id":"X","name":"x.png","parents":["G2"],"version":"1"}},
            {"fileId":"G2","file":{"id":"G2","name":"04","parents":["G1"],"mimeType":"application/vnd.google-apps.folder"}},
            {"fileId":"G1","file":{"id":"G1","name":"attachments","parents":["R"],"mimeType":"application/vnd.google-apps.folder"}}]"#,
    );

    let (next, applied) = super::tree::applied(&tree, &roots, &moved).unwrap();

    assert_eq!(applied.gone, vec!["docs/a.md".to_string()]);
    assert_eq!(applied.changed, vec!["attachments/04/x.png".to_string()]);
    assert!(!next.files.contains_key("docs/a.md"));
}

#[test]
fn a_folder_of_the_store_renamed_or_binned_asks_for_the_whole_listing() {
    let tree = held_tree(DOCS);
    let roots = ["R".to_string()];
    let renamed = changed(
        r#"[{"fileId":"F","file":{"id":"F","name":"papers","parents":["R"],"mimeType":"application/vnd.google-apps.folder"}}]"#,
    );
    let binned = changed(
        r#"[{"fileId":"R","file":{"id":"R","name":"Tisty","parents":["myroot"],"mimeType":"application/vnd.google-apps.folder","trashed":true}}]"#,
    );
    let touched = changed(
        r#"[{"fileId":"F","file":{"id":"F","name":"docs","parents":["R"],"mimeType":"application/vnd.google-apps.folder"}}]"#,
    );

    assert!(super::tree::applied(&tree, &roots, &renamed).is_none());
    assert!(super::tree::applied(&tree, &roots, &binned).is_none());
    assert!(super::tree::applied(&tree, &roots, &touched).is_some());
}

#[test]
fn an_administrator_s_block_and_drive_s_own_ceilings_are_told_apart() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let reason =
        |one: &str| format!(r#"{{"error":{{"message":"m","errors":[{{"reason":"{one}"}}]}}}}"#);
    let (at, _) = google(vec![
        said("200 OK", &whole("")),
        said("200 OK", &listing("")),
        said("403 Forbidden", &reason("domainPolicy")),
        said("200 OK", &listing("")),
        said(
            "403 Forbidden",
            &reason("numChildrenInNonRootLimitExceeded"),
        ),
        said("200 OK", &listing("")),
        said("403 Forbidden", &reason("insufficientFilePermissions")),
    ]);
    let drive = drive(at, home.path());

    assert!(matches!(
        drive.put("a.md", &local, Expect::Any),
        Err(Hitch::Broke(said)) if said.contains("administrators")
    ));
    assert_eq!(drive.put("a.md", &local, Expect::Any), Err(Hitch::Full));
    assert!(matches!(
        drive.put("a.md", &local, Expect::Any),
        Err(Hitch::Broke(said)) if said.contains("may not change")
    ));
}

#[test]
fn a_session_google_let_go_of_starts_the_upload_over_and_is_never_a_missing_file() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"0123456789");
    let (at, _) = google_at(|base| {
        vec![
            said("200 OK", &whole("")),
            said("200 OK", &listing("")),
            Said {
                status: "200 OK",
                headers: format!("Location: {base}/upload/session\r\n"),
                body: String::new(),
            },
            said("503 Service Unavailable", ""),
            said("404 Not Found", ""),
        ]
    });
    let drive = drive(at, home.path()).with_chunk(8);

    let landed = drive.put("big.bin", &local, Expect::Absent);

    assert!(matches!(landed, Err(Hitch::Unreachable(_))), "{landed:?}");
}

#[test]
fn a_drive_bound_to_an_account_reads_nothing_of_another_and_asks_once() {
    let home = tempfile::tempdir().unwrap();
    let them = r#"{"user":{"emailAddress":"otra@example.com","permissionId":"P2"}}"#;
    let (at, asked) = google(vec![said("200 OK", them)]);
    let elsewhere = drive(at, home.path()).for_account("P1");

    assert_eq!(
        elsewhere.list(""),
        Err(Hitch::Elsewhere {
            found: "otra@example.com".into()
        })
    );
    assert_eq!(asked.join().unwrap().len(), 1, "a listing went out first");

    let me = r#"{"user":{"emailAddress":"mario@example.com","permissionId":"P1"}}"#;
    let (at, asked) = google(vec![
        said("200 OK", me),
        said("200 OK", &whole("")),
        said("200 OK", &whole("")),
    ]);
    let mine = drive(at, home.path()).for_account("P1");

    mine.list("").unwrap();
    mine.list("").unwrap();

    assert_eq!(
        asked
            .join()
            .unwrap()
            .iter()
            .filter(|one| one.line.contains("/about"))
            .count(),
        1
    );
}

#[test]
fn an_older_revision_is_never_taken_for_another_machine_s_write() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"mine");
    let (at, asked) = google(vec![
        said("200 OK", &whole(CHECKED)),
        said("200 OK", CHECKED),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m3","headRevisionId":"h3","version":"4"}"#,
        ),
        said(
            "200 OK",
            r#"{"revisions":[{"id":"h0","md5Checksum":"m0","modifiedTime":"2026-10-11T09:00:00Z"},{"id":"h1","md5Checksum":"m1","modifiedTime":"2026-10-12T10:00:01Z"},{"id":"h3","md5Checksum":"m3","modifiedTime":"2026-10-12T10:00:03Z"}]}"#,
        ),
    ]);

    let landed = drive(at, home.path()).put("a.md", &local, Expect::Revision("A.1".into()));

    assert_eq!(landed.map(|one| one.revision), Ok("A.4".to_string()));
    assert_eq!(
        asked.join().unwrap().len(),
        4,
        "yesterday's bytes went back on top"
    );
}

#[test]
fn asking_again_from_a_cursor_the_listing_already_passed_lists_nothing_again() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(A)),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    drive.changes(Some("10")).unwrap();
    let again = drive.changes(Some("10")).unwrap();

    assert!(matches!(again, Changes::Since { ref cursor, .. } if cursor == "11"));
    let listings = asked
        .join()
        .unwrap()
        .iter()
        .filter(|one| one.line.contains("q=trashed"))
        .count();
    assert_eq!(
        listings, 1,
        "a poll from the index's older cursor listed the drive again"
    );
}

#[test]
fn a_change_inside_a_second_folder_of_the_same_name_is_still_found() {
    let tree = held_tree(&format!(
        r#"{DOCS},{{"id":"F2","name":"docs","parents":["R"],"mimeType":"application/vnd.google-apps.folder"}},{{"id":"B","name":"b.md","parents":["F2"],"version":"1"}}"#
    ));
    let roots = ["R".to_string()];
    let moved = changed(
        r#"[{"fileId":"B","file":{"id":"B","name":"b.md","parents":["F2"],"version":"2"}},
            {"fileId":"C","file":{"id":"C","name":"c.md","parents":["F2"],"version":"1"}}]"#,
    );

    let (_, applied) = super::tree::applied(&tree, &roots, &moved).unwrap();

    assert_eq!(
        applied.changed,
        vec!["docs/b.md".to_string(), "docs/c.md".to_string()]
    );
    assert!(applied.gone.is_empty());
}

#[test]
fn a_file_gone_before_it_reached_the_bin_still_takes_its_other_copies() {
    let home = tempfile::tempdir().unwrap();
    let one = r#"{"id":"A1","name":"a.md","parents":["R"],"version":"1"}"#;
    let two = format!(r#"{one},{{"id":"A2","name":"a.md","parents":["R"],"version":"1"}}"#);
    let (at, asked) = google(vec![
        said("200 OK", &whole(&two)),
        said("200 OK", one),
        said("404 Not Found", ""),
        said("204 No Content", ""),
    ]);

    let gone = drive(at, home.path()).delete("a.md", Some("A1.1"));

    assert_eq!(gone, Ok(()));
    assert!(
        asked.join().unwrap()[3]
            .line
            .starts_with("DELETE /drive/v3/files/A2")
    );
}

#[test]
fn bytes_that_could_not_go_back_at_once_go_back_before_the_next_change_is_read() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"mine");
    let revisions = r#"{"revisions":[{"id":"h1","md5Checksum":"m1","modifiedTime":"2026-10-12T10:00:01Z"},{"id":"h2","md5Checksum":"m2","modifiedTime":"2026-10-12T10:00:02Z"},{"id":"h3","md5Checksum":"m3","modifiedTime":"2026-10-12T10:00:03Z"}]}"#;
    let (at, asked) = google(vec![
        said("200 OK", &whole(CHECKED)),
        said("200 OK", CHECKED),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m3","headRevisionId":"h3","version":"3"}"#,
        ),
        said("503 Service Unavailable", ""),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m3","headRevisionId":"h3","version":"3"}"#,
        ),
        said("200 OK", revisions),
        said("200 OK", r#"{"id":"h2"}"#),
        said("200 OK", "theirs"),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m2","headRevisionId":"h4","version":"4"}"#,
        ),
        said("204 No Content", ""),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
        said("200 OK", &whole(CHECKED)),
    ]);
    let drive = drive(at, home.path()).owing_at(home.path());

    let first = drive.put("a.md", &local, Expect::Revision("A.1".into()));
    assert!(matches!(first, Err(Hitch::Unreachable(_))), "{first:?}");
    assert!(home.path().join("drive-owed.json").is_file());

    drive.changes(Some("10")).unwrap();

    let asked = asked.join().unwrap();
    let sent = String::from_utf8_lossy(&asked[8].body);
    assert!(
        sent.contains("theirs"),
        "their bytes never went back: {sent}"
    );
    assert!(
        asked[9]
            .line
            .starts_with("DELETE /drive/v3/files/A/revisions/h2")
    );
    assert!(
        !home.path().join("drive-owed.json").exists(),
        "a debt paid was kept"
    );
}

#[test]
fn a_deletion_heard_by_a_poll_is_still_gone_for_the_round_that_asks_from_the_same_cursor() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(A)),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"A","removed":true}],"newStartPageToken":"11"}"#,
        ),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
    ]);
    let drive = drive(at, home.path());
    drive.changes(None).unwrap();

    let polled = drive.changes(Some("10")).unwrap();
    let round = drive.changes(Some("10")).unwrap();

    for heard in [polled, round] {
        assert!(
            matches!(heard, Changes::Since { ref gone, .. } if gone == &vec!["a.md".to_string()]),
            "{heard:?}"
        );
    }
    let asked = asked.join().unwrap();
    assert!(asked[3].line.contains("pageToken=11"), "{}", asked[3].line);
}

#[test]
fn a_debt_whose_file_moved_on_keeps_the_bytes_in_drive_s_history_and_writes_over_nothing() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"mine");
    let revisions = r#"{"revisions":[{"id":"h1","md5Checksum":"m1","modifiedTime":"2026-10-12T10:00:01Z"},{"id":"h2","md5Checksum":"m2","modifiedTime":"2026-10-12T10:00:02Z"},{"id":"h3","md5Checksum":"m3","modifiedTime":"2026-10-12T10:00:03Z"},{"id":"h5","md5Checksum":"m5","modifiedTime":"2026-10-12T11:00:00Z"}]}"#;
    let (at, asked) = google(vec![
        said("200 OK", &whole(CHECKED)),
        said("200 OK", CHECKED),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m3","headRevisionId":"h3","version":"3"}"#,
        ),
        said("503 Service Unavailable", ""),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m5","headRevisionId":"h5","version":"6"}"#,
        ),
        said("200 OK", revisions),
        said("200 OK", r#"{"id":"h2"}"#),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
        said("200 OK", &whole(CHECKED)),
    ]);
    let drive = drive(at, home.path()).owing_at(home.path());

    assert!(
        drive
            .put("a.md", &local, Expect::Revision("A.1".into()))
            .is_err()
    );
    drive.changes(Some("10")).unwrap();

    let asked = asked.join().unwrap();
    assert!(
        asked[6]
            .line
            .starts_with("PATCH /drive/v3/files/A/revisions/h2")
    );
    assert!(
        !asked[4..]
            .iter()
            .any(|one| one.line.starts_with("PATCH /upload")),
        "a newer write was covered by older bytes"
    );
    assert!(!home.path().join("drive-owed.json").exists());
}
