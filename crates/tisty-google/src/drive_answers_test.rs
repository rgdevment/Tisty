use super::*;

fn cut_short(head: &'static str, sent: &'static str) -> (DriveAt, JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let served = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        heard(&mut stream);
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n{head}Content-Length: 4096\r\nConnection: close\r\n\r\n{sent}"
        )
        .unwrap();
    });
    (
        DriveAt {
            api: format!("{base}/drive/v3"),
            upload: format!("{base}/upload/drive/v3"),
        },
        served,
    )
}

#[test]
fn bytes_put_back_after_the_gap_come_as_another_machine_s_change_and_are_never_taken_for_ours() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"mine");
    let restored = r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m2","headRevisionId":"h4","version":"4"}"#;
    let (at, _) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(CHECKED)),
        said("200 OK", CHECKED),
        said(
            "200 OK",
            r#"{"id":"A","name":"a.md","parents":["R"],"md5Checksum":"m3","headRevisionId":"h3","version":"3"}"#,
        ),
        said(
            "200 OK",
            r#"{"revisions":[{"id":"h1","md5Checksum":"m1","modifiedTime":"2026-10-12T10:00:01Z"},{"id":"h2","md5Checksum":"m2","modifiedTime":"2026-10-12T10:00:02Z"},{"id":"h3","md5Checksum":"m3","modifiedTime":"2026-10-12T10:00:03Z"}]}"#,
        ),
        said("200 OK", r#"{"id":"h2"}"#),
        said("200 OK", "theirs"),
        said("200 OK", restored),
        said("204 No Content", ""),
        said(
            "200 OK",
            &format!(
                r#"{{"changes":[{{"fileId":"A","file":{restored}}}],"newStartPageToken":"11"}}"#
            ),
        ),
    ]);
    let drive = drive(at, home.path());
    drive.changes(None).unwrap();

    let refused = drive.put("a.md", &local, Expect::Revision("A.1".into()));
    let heard = drive.changes(Some("10")).unwrap();

    assert_eq!(refused, Err(Hitch::Changed("a.md".into())));
    assert!(
        matches!(&heard, Changes::Since { changed, .. } if changed.iter().any(|one| one.revision == "A.4")),
        "the bytes put back were taken for this machine's own write: {heard:?}"
    );
}

#[test]
fn an_empty_file_goes_up_and_an_answer_without_a_sha256_still_lands() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"");
    let (at, asked) = google(vec![
        said("200 OK", &whole("")),
        said("200 OK", &listing("")),
        said(
            "200 OK",
            r#"{"id":"E","name":"e.md","parents":["R"],"size":"0","version":"1"}"#,
        ),
    ]);

    let landed = drive(at, home.path())
        .put("e.md", &local, Expect::Absent)
        .unwrap();

    assert_eq!(
        landed,
        Seen {
            name: "e.md".into(),
            bytes: 0,
            hash: String::new(),
            revision: "E.1".into(),
        }
    );
    let asked = asked.join().unwrap();
    assert!(
        asked[2].line.contains("uploadType=multipart"),
        "an empty file opened an upload session: {}",
        asked[2].line
    );
}

#[test]
fn garbage_and_html_answers_are_a_stop_not_a_wedge() {
    let home = tempfile::tempdir().unwrap();
    let html = Said {
        status: "200 OK",
        headers: String::new(),
        body: "<html><body>Sign in to continue</body></html>".into(),
    };
    let (at, _) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(A)),
        html,
        said("200 OK", "{\"changes\": [ nonsense"),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
    ]);
    let drive = drive(at, home.path());
    drive.changes(None).unwrap();

    let first = drive.changes(Some("10"));
    let second = drive.changes(Some("10"));
    let third = drive.changes(Some("10"));

    assert!(matches!(first, Err(Hitch::Broke(_))), "{first:?}");
    assert!(matches!(second, Err(Hitch::Broke(_))), "{second:?}");
    assert_eq!(
        third,
        Ok(Changes::Since {
            changed: Vec::new(),
            gone: Vec::new(),
            cursor: "11".into()
        }),
        "an answer that did not read left the listing stuck"
    );
}

#[test]
fn every_way_google_says_slow_down_is_a_wait_and_a_bare_fault_is_trouble_not_offline() {
    let home = tempfile::tempdir().unwrap();
    let reason =
        |one: &str| format!(r#"{{"error":{{"message":"m","errors":[{{"reason":"{one}"}}]}}}}"#);
    let (at, _) = google(vec![
        said("403 Forbidden", &reason("rateLimitExceeded")),
        said("403 Forbidden", &reason("userRateLimitExceeded")),
        said("503 Service Unavailable", ""),
        said("500 Internal Server Error", "<html>oops</html>"),
        Said {
            status: "503 Service Unavailable",
            headers: "Retry-After: 7\r\n".into(),
            body: String::new(),
        },
        said("410 Gone", &reason("badRequest")),
    ]);
    let drive = drive(at, home.path());

    let mut heard = Vec::new();
    for _ in 0..6 {
        heard.push(drive.changes(Some("10")).err().unwrap());
    }

    assert!(matches!(heard[0], Hitch::Limited { .. }), "{:?}", heard[0]);
    assert!(matches!(heard[1], Hitch::Limited { .. }), "{:?}", heard[1]);
    assert!(matches!(heard[2], Hitch::Troubled(_)), "{:?}", heard[2]);
    assert!(matches!(heard[3], Hitch::Troubled(_)), "{:?}", heard[3]);
    assert_eq!(
        heard[4],
        Hitch::Limited {
            wait: Duration::from_secs(7)
        }
    );
    assert!(
        matches!(heard[5], Hitch::Broke(_)),
        "a cursor Google let go of must be read whole again: {:?}",
        heard[5]
    );
}

#[test]
fn a_line_cut_while_an_answer_was_coming_is_no_connection_and_never_a_broken_drive() {
    let home = tempfile::tempdir().unwrap();
    let (at, served) = cut_short("", r#"{"files":[{"id":"R","name":"Tis"#);

    let cut = drive(at, home.path()).list("");

    served.join().unwrap();
    assert!(matches!(cut, Err(Hitch::Unreachable(_))), "{cut:?}");
}
