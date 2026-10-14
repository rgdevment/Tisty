use super::*;

fn cut_short(sent: &'static str) -> (DriveAt, JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let served = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        heard(&mut stream);
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 4096\r\nConnection: close\r\n\r\n{sent}"
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
    let (at, served) = cut_short(r#"{"files":[{"id":"R","name":"Tis"#);

    let cut = drive(at, home.path()).list("");

    served.join().unwrap();
    assert!(matches!(cut, Err(Hitch::Unreachable(..))), "{cut:?}");
}

#[test]
fn a_file_the_lagging_listing_does_not_show_yet_is_still_read_from_the_changes_that_named_it() {
    let home = tempfile::tempdir().unwrap();
    let z = r#"{"id":"Z","name":"z.md","parents":["R"],"size":"3","sha256Checksum":"zz","version":"1"}"#;
    let (at, _) = google(vec![
        said(
            "200 OK",
            &format!(r#"{{"changes":[{{"fileId":"Z","file":{z}}}],"newStartPageToken":"6"}}"#),
        ),
        said("200 OK", &whole(A)),
    ]);

    let heard = drive(at, home.path()).changes(Some("5")).unwrap();

    let Changes::Whole { seen, cursor } = heard else {
        panic!("{heard:?}");
    };
    assert_eq!(cursor, "6");
    assert!(
        seen.iter().any(|one| one.name == "z.md"),
        "a file the listing lagged behind on was skipped: {seen:?}"
    );
    assert!(seen.iter().any(|one| one.name == "a.md"), "{seen:?}");
}

#[test]
fn a_debt_owed_while_another_was_being_repaid_stays_owed() {
    use super::super::owed::{Owed, unpaid};
    let owed = |id: &str| Owed::Back {
        name: "a.md".into(),
        id: id.into(),
        checked_head: None,
        checked_md5: None,
        landed_head: None,
        landed_md5: None,
    };
    let paid = owed("A");
    let meanwhile = owed("B");

    let left = unpaid(vec![paid.clone(), meanwhile.clone()], &[paid]);

    assert_eq!(left, vec![meanwhile]);
}

#[test]
fn a_repayment_already_running_elsewhere_is_not_run_twice() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(Vec::new());
    let drive = drive(at, home.path()).owing_at(home.path());
    let owed = home.path().join(tisty_carrier::OWED_BACK);
    std::fs::write(&owed, br#"[{"owed":"back","name":"a.md","id":"A"}]"#).unwrap();
    let running = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(owed.with_extension("repaying.lock"))
        .unwrap();
    running.try_lock().unwrap();

    drive.repaid();

    assert!(asked.join().unwrap().is_empty());
    assert_eq!(
        std::fs::read(&owed).unwrap(),
        br#"[{"owed":"back","name":"a.md","id":"A"}]"#
    );
}

#[test]
fn a_quota_of_the_project_is_a_wait_and_only_a_quota_of_storage_is_a_full_drive() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let refused = |reason: &str, message: &str| {
        format!(r#"{{"error":{{"message":"{message}","errors":[{{"reason":"{reason}"}}]}}}}"#)
    };
    let (at, _) = google(vec![
        said("200 OK", &whole("")),
        said("200 OK", &listing("")),
        said(
            "403 Forbidden",
            &refused("quotaExceeded", "Quota exceeded for quota metric 'Queries'"),
        ),
        said("200 OK", &listing("")),
        said(
            "403 Forbidden",
            &refused(
                "quotaExceeded",
                "The user's Drive storage quota has been exceeded.",
            ),
        ),
    ]);
    let drive = drive(at, home.path());

    assert_eq!(
        drive.put("a.md", &local, Expect::Any),
        Err(Hitch::Limited {
            wait: Duration::from_secs(3600)
        }),
        "a quota of the project was told as a full Drive"
    );
    assert_eq!(drive.put("a.md", &local, Expect::Any), Err(Hitch::Full));
}

#[test]
fn a_long_upload_cut_short_goes_on_next_round_from_where_google_kept_it() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"0123456789");
    let landed = r#"{"id":"B","name":"big.bin","parents":["R"],"size":"10","sha256Checksum":"x","version":"1"}"#;
    let kept_up_to = |last: u64| Said {
        status: "308 Resume Incomplete",
        headers: format!("Range: bytes=0-{last}\r\n"),
        body: String::new(),
    };
    let (at, asked) = google_at(|base| {
        vec![
            said("200 OK", &whole("")),
            said("200 OK", &listing("")),
            Said {
                status: "200 OK",
                headers: format!("Location: {base}/upload/session\r\n"),
                body: String::new(),
            },
            kept_up_to(3),
            said(
                "403 Forbidden",
                r#"{"error":{"message":"m","errors":[{"reason":"userRateLimitExceeded"}]}}"#,
            ),
            said("200 OK", &listing("")),
            kept_up_to(3),
            kept_up_to(7),
            said("200 OK", landed),
        ]
    });
    let drive = drive(at, home.path()).owing_at(home.path()).with_chunk(4);

    let cut = drive.put("big.bin", &local, Expect::Absent);
    let again = drive.put("big.bin", &local, Expect::Absent);

    assert!(matches!(cut, Err(Hitch::Limited { .. })), "{cut:?}");
    assert_eq!(again.unwrap().revision, "B.1");
    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 9);
    assert!(
        asked[5..]
            .iter()
            .all(|one| !one.line.contains("uploadType=resumable")),
        "a second session was opened instead of going on with the first"
    );
    assert!(
        asked[6].line.contains("/upload/session"),
        "{}",
        asked[6].line
    );
    assert!(
        header(&asked[6], "content-range").ends_with("bytes */10"),
        "{}",
        asked[6].head
    );
    assert!(
        header(&asked[7], "content-range").ends_with("bytes 4-7/10"),
        "it started over: {}",
        asked[7].head
    );
    assert_eq!(asked[8].body, b"89");
    assert!(
        !home.path().join("uploading.json").exists(),
        "a finished upload is still written down"
    );
}

#[test]
fn the_room_left_is_the_ceiling_less_what_is_used_and_no_ceiling_is_no_limit() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said(
            "200 OK",
            r#"{"storageQuota":{"limit":"16106127360","usage":"16000000000"}}"#,
        ),
        said("200 OK", r#"{"storageQuota":{"usage":"5"}}"#),
    ]);
    let drive = drive(at, home.path());

    assert_eq!(drive.room(), Ok(Some(106_127_360)));
    assert_eq!(drive.room(), Ok(None));
    assert!(
        asked.join().unwrap()[0]
            .line
            .contains("fields=storageQuota"),
        "the room was not asked of the quota"
    );
}

#[test]
fn a_grant_from_before_tisty_kept_to_its_own_space_asks_to_connect_again() {
    let home = tempfile::tempdir().unwrap();
    let refused = |reason: &str, message: &str| {
        format!(r#"{{"error":{{"message":"{message}","errors":[{{"reason":"{reason}"}}]}}}}"#)
    };
    let (at, _) = google(vec![
        said(
            "403 Forbidden",
            &refused("insufficientScopes", "Insufficient Permission"),
        ),
        said(
            "403 Forbidden",
            &refused(
                "forbidden",
                "The granted scopes do not give access to all of the requested spaces: Application Data folder.",
            ),
        ),
    ]);
    let drive = drive(at, home.path());

    assert_eq!(drive.list(""), Err(Hitch::Lost));
    assert_eq!(drive.list(""), Err(Hitch::Lost));
}

#[test]
fn a_debt_of_a_kind_no_longer_owed_is_dropped_and_never_hides_the_others() {
    let home = tempfile::tempdir().unwrap();
    let (at, _) = google(Vec::new());
    let drive = drive(at, home.path()).owing_at(home.path());
    let owed = home.path().join(tisty_carrier::OWED_BACK);
    std::fs::write(&owed, br#"[{"owed":"unbinned","id":"A"}]"#).unwrap();

    drive.repaid();

    assert!(!owed.exists(), "a debt of the old kind was kept forever");
}
