use super::*;

const LANDED: &str = r#"{"id":"N","name":"n.bin","parents":["B"],"size":"5","version":"1"}"#;
const SHARD: &str = r#"{"id":"H","name":"attachments","parents":["R"],"mimeType":"application/vnd.google-apps.folder"},
    {"id":"B","name":"ab","parents":["H"],"mimeType":"application/vnd.google-apps.folder"}"#;

fn searched(asked: &[Asked]) -> usize {
    asked
        .iter()
        .filter(|one| {
            one.line.contains("q=name")
                && !["mimeType+%3D", "mimeType%20%3D"]
                    .iter()
                    .any(|folder| one.line.contains(folder))
        })
        .count()
}

fn aged(drive: &Drive) {
    let mut held = drive.known.lock().unwrap();
    let known = held.as_mut().unwrap();
    known.current_at = std::time::Instant::now()
        .checked_sub(Duration::from_secs(120))
        .unwrap();
}

#[test]
fn a_new_attachment_in_a_tree_the_changes_feed_just_caught_up_is_written_without_a_search() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(SHARD)),
        said("200 OK", LANDED),
        said("200 OK", LANDED),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    drive
        .put("attachments/ab/n.bin", &local, Expect::Absent)
        .unwrap();
    drive
        .put("attachments/ab/m.bin", &local, Expect::Absent)
        .unwrap();

    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 4, "{asked:?}");
    assert_eq!(
        searched(&asked),
        0,
        "each new attachment still paid a name search"
    );
    assert!(asked[2].line.contains("uploadType=multipart"));
}

#[test]
fn a_shared_name_absent_from_a_current_tree_is_still_searched_before_it_is_written() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(SHARD)),
        said("200 OK", &listing("")),
        said(
            "200 OK",
            r#"{"id":"M","name":"moved.json","parents":["R"],"version":"1"}"#,
        ),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    drive.put("moved.json", &local, Expect::Absent).unwrap();

    assert_eq!(searched(&asked.join().unwrap()), 1);
}

#[test]
fn a_tree_older_than_a_minute_is_caught_up_once_from_the_feed_instead_of_a_search_per_file() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let theirs = r#"{"id":"T","name":"t.bin","parents":["B"],"version":"1"}"#;
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(SHARD)),
        said(
            "200 OK",
            &format!(
                r#"{{"changes":[{{"fileId":"T","file":{theirs}}}],"newStartPageToken":"11"}}"#
            ),
        ),
        said("200 OK", LANDED),
    ]);
    let drive = drive(at, home.path());
    drive.changes(None).unwrap();
    aged(&drive);

    let refused = drive.put("attachments/ab/t.bin", &local, Expect::Absent);
    let landed = drive.put("attachments/ab/n.bin", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("attachments/ab/t.bin".into())));
    assert!(landed.is_ok(), "{landed:?}");
    let asked = asked.join().unwrap();
    assert!(asked[2].line.contains("pageToken=10"), "{}", asked[2].line);
    assert_eq!(searched(&asked), 0);
    assert_eq!(asked.len(), 4, "{asked:?}");
}

#[test]
fn a_quiet_catch_up_between_rounds_never_pushes_out_the_cursor_the_index_holds() {
    let home = tempfile::tempdir().unwrap();
    let mut script = vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(SHARD)),
    ];
    for at in 0..70 {
        script.push(said(
            "200 OK",
            &format!(r#"{{"changes":[],"newStartPageToken":"{}"}}"#, 100 + at),
        ));
    }
    script.push(said(
        "200 OK",
        r#"{"changes":[],"newStartPageToken":"500"}"#,
    ));
    let (at, asked) = google(script);
    let drive = drive(at, home.path());
    drive.changes(None).unwrap();

    for _ in 0..70 {
        aged(&drive);
        assert!(drive.kept_current().unwrap());
    }
    let later = drive.changes(Some("10")).unwrap();

    assert!(matches!(later, Changes::Since { ref cursor, .. } if cursor == "500"));
    let listings = asked
        .join()
        .unwrap()
        .iter()
        .filter(|one| one.line.contains("q=trashed"))
        .count();
    assert_eq!(
        listings, 1,
        "a quiet day of catch-ups listed the drive again"
    );
}

#[test]
fn a_tree_listed_with_no_cursor_to_follow_still_asks_each_new_name() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", &whole("")),
        said("200 OK", &listing("")),
        said("200 OK", LANDED),
    ]);

    drive(at, home.path())
        .put("n.md", &local, Expect::Absent)
        .unwrap();

    assert_eq!(searched(&asked.join().unwrap()), 1);
}

#[test]
fn every_ask_leaves_one_line_at_the_finest_trail_without_where_it_went_or_what_it_named() {
    let home = tempfile::tempdir().unwrap();
    let log = home.path().join("trail").join("tisty.log");
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", &whole("")),
        said("200 OK", &listing("")),
        said("200 OK", LANDED),
    ]);
    tisty_core::witness::keeps(log.clone(), true);

    drive(at, home.path())
        .put("secreto-de-nadie.md", &local, Expect::Absent)
        .unwrap();

    tisty_core::witness::keeps(home.path().join("elsewhere").join("tisty.log"), false);
    asked.join().unwrap();
    let trail = std::fs::read_to_string(&log).unwrap();
    let ours: Vec<&str> = trail
        .lines()
        .filter(|one| one.contains("Google Drive was asked"))
        .collect();
    assert!(ours.len() >= 3, "{trail}");
    assert!(
        ours.iter().any(|one| one.contains("TRACE")
            && one.contains("cloud")
            && one.contains("verb=GET")
            && one.contains("kind=list")),
        "{trail}"
    );
    assert!(
        ours.iter()
            .any(|one| one.contains("kind=search") && one.contains("status=200")),
        "{trail}"
    );
    assert!(
        ours.iter().any(|one| one.contains("verb=POST")
            && one.contains("kind=upload")
            && one.contains("received=")
            && one.contains("ms=")),
        "{trail}"
    );
    for hidden in ["secreto", "127.0.0.1", "access-", "drive/v3"] {
        assert!(!trail.contains(hidden), "{hidden} reached the log: {trail}");
    }
}

#[test]
fn a_folder_a_current_tree_does_not_hold_is_searched_once_so_two_machines_meet_in_one() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole("")),
        said("200 OK", &listing("")),
        said("200 OK", r#"{"id":"H","name":"attachments"}"#),
        said("200 OK", &listing(r#"{"id":"B"}"#)),
        said(
            "200 OK",
            r#"{"id":"X","name":"x.bin","parents":["B"],"size":"5","version":"1"}"#,
        ),
        said(
            "200 OK",
            r#"{"id":"Y","name":"y.bin","parents":["B"],"size":"5","version":"1"}"#,
        ),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    let landed = drive.put("attachments/ab/x.bin", &local, Expect::Absent);
    let again = drive.put("attachments/ab/y.bin", &local, Expect::Absent);

    assert_eq!(landed.map(|one| one.revision), Ok("X.1".into()));
    assert_eq!(again.map(|one| one.revision), Ok("Y.1".into()));
    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 7, "{asked:?}");
    assert!(asked[2].line.contains("mimeType"), "{}", asked[2].line);
    assert!(asked[4].line.contains("mimeType"), "{}", asked[4].line);
    assert!(
        asked[4].line.contains("H%27+in+parents") || asked[4].line.contains("H%27%20in%20parents"),
        "{}",
        asked[4].line
    );
    assert_eq!(searched(&asked), 0);
    assert!(String::from_utf8_lossy(&asked[5].body).contains(r#""parents":["B"]"#));
}

#[test]
fn a_listing_taken_between_rounds_still_knows_the_cursor_the_next_round_asks_from() {
    let home = tempfile::tempdir().unwrap();
    let renamed = r#"{"id":"H","name":"attachments2","parents":["R"],"mimeType":"application/vnd.google-apps.folder"},
    {"id":"B","name":"ab","parents":["H"],"mimeType":"application/vnd.google-apps.folder"}"#;
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(&format!("{SHARD},{LANDED}"))),
        said(
            "200 OK",
            r#"{"changes":[{"fileId":"H","file":{"id":"H","name":"attachments2","parents":["R"],"mimeType":"application/vnd.google-apps.folder"}}],"newStartPageToken":"11"}"#,
        ),
        said("200 OK", &whole(&format!("{renamed},{LANDED}"))),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
    ]);
    let drive = drive(at, home.path());
    drive.changes(None).unwrap();
    aged(&drive);
    assert_eq!(drive.about("attachments/ab/m.bin"), Ok(None));

    let round = drive.changes(Some("10")).unwrap();

    let Changes::Since {
        changed,
        gone,
        cursor,
    } = round
    else {
        panic!("{round:?}");
    };
    assert_eq!(cursor, "11");
    assert_eq!(gone, vec!["attachments/ab/n.bin".to_string()]);
    assert_eq!(
        changed
            .iter()
            .map(|one| one.name.as_str())
            .collect::<Vec<_>>(),
        ["attachments2/ab/n.bin"]
    );
    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 5, "{asked:?}");
    assert_eq!(
        asked
            .iter()
            .filter(|one| one.line.contains("q=trashed"))
            .count(),
        2,
        "the round listed the drive again right after a listing between rounds"
    );
}
