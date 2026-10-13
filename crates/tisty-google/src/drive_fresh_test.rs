use super::*;

const LANDED: &str = r#"{"id":"N","name":"n.md","parents":["R"],"size":"5","version":"1"}"#;

fn searched(asked: &[Asked]) -> usize {
    asked
        .iter()
        .filter(|one| one.line.contains("q=name"))
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
fn a_new_file_in_a_tree_the_changes_feed_just_caught_up_is_written_without_a_search() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole("")),
        said("200 OK", LANDED),
        said("200 OK", LANDED),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    drive.put("n.md", &local, Expect::Absent).unwrap();
    drive.put("m.md", &local, Expect::Absent).unwrap();

    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 4, "{asked:?}");
    assert_eq!(
        searched(&asked),
        0,
        "each new file still paid a name search"
    );
    assert!(asked[2].line.contains("uploadType=multipart"));
}

#[test]
fn a_tree_older_than_a_minute_is_caught_up_once_from_the_feed_instead_of_a_search_per_file() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let theirs = r#"{"id":"T","name":"t.md","parents":["R"],"version":"1"}"#;
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole(A)),
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

    let refused = drive.put("t.md", &local, Expect::Absent);
    let landed = drive.put("n.md", &local, Expect::Absent);

    assert_eq!(refused, Err(Hitch::Changed("t.md".into())));
    assert!(landed.is_ok(), "{landed:?}");
    let asked = asked.join().unwrap();
    assert!(asked[2].line.contains("pageToken=10"), "{}", asked[2].line);
    assert_eq!(searched(&asked), 0);
    assert_eq!(asked.len(), 4, "{asked:?}");
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
    assert_eq!(ours.len(), 3, "{trail}");
    assert!(
        ours[0].contains("TRACE") && ours[0].contains("cloud"),
        "{trail}"
    );
    assert!(
        ours[0].contains("verb=GET") && ours[0].contains("kind=list"),
        "{trail}"
    );
    assert!(
        ours[1].contains("kind=search") && ours[1].contains("status=200"),
        "{trail}"
    );
    assert!(
        ours[2].contains("verb=POST") && ours[2].contains("kind=upload"),
        "{trail}"
    );
    assert!(
        ours[2].contains("received=") && ours[2].contains("ms="),
        "{trail}"
    );
    for hidden in ["secreto", "127.0.0.1", "access-", "drive/v3"] {
        assert!(!trail.contains(hidden), "{hidden} reached the log: {trail}");
    }
}

#[test]
fn folders_a_current_tree_does_not_hold_are_made_without_searching_for_them_first() {
    let home = tempfile::tempdir().unwrap();
    let local = written(&home, b"hello");
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said("200 OK", &whole("")),
        said("200 OK", r#"{"id":"H","name":"attachments"}"#),
        said("200 OK", r#"{"id":"B","name":"ab"}"#),
        said(
            "200 OK",
            r#"{"id":"X","name":"x.bin","parents":["B"],"size":"5","version":"1"}"#,
        ),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    let landed = drive.put("attachments/ab/x.bin", &local, Expect::Absent);

    assert_eq!(landed.map(|one| one.revision), Ok("X.1".into()));
    let asked = asked.join().unwrap();
    assert!(
        asked[2..].iter().all(|one| !one.line.contains("q=")),
        "{asked:?}"
    );
    assert!(String::from_utf8_lossy(&asked[3].body).contains(r#""parents":["H"]"#));
}
