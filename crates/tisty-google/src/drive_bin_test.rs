use super::*;
use crate::drive::owed::Owed;

const SET_ASIDE: &str = r#"{"id":"A","name":"a.md","parents":["R"],"headRevisionId":"h1","md5Checksum":"m1","version":"1"}"#;
const CHANGED_IN_BIN: &str = r#"{"id":"A","name":"1-a.md","parents":["BIN"],"headRevisionId":"h2","md5Checksum":"m2","version":"3"}"#;

fn debts(home: &tempfile::TempDir) -> Option<Vec<u8>> {
    std::fs::read(home.path().join(tisty_carrier::OWED_BACK)).ok()
}

fn removed(asked: &[Asked], id: &str) -> bool {
    asked.iter().any(|one| {
        one.line
            .starts_with(&format!("DELETE /drive/v3/files/{id} "))
    })
}

#[test]
fn a_changed_file_that_could_not_leave_the_bin_goes_back_on_a_later_round_and_is_never_swept() {
    let home = tempfile::tempdir().unwrap();
    let (at, asked) = google(vec![
        said("200 OK", &whole(SET_ASIDE)),
        said("200 OK", SET_ASIDE),
        said("200 OK", &listing("")),
        said("200 OK", BIN_MADE),
        said("200 OK", r#"{"id":"A"}"#),
        said("200 OK", CHANGED_IN_BIN),
        said("503 Service Unavailable", ""),
        said("200 OK", CHANGED_IN_BIN),
        said("200 OK", r#"{"id":"A"}"#),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
        said("200 OK", &whole(SET_ASIDE)),
    ]);
    let drive = drive(at, home.path()).owing_at(home.path());

    assert!(drive.delete("a.md", Some("A.1")).is_err());
    assert!(
        debts(&home).is_some(),
        "putting it back was never written down"
    );
    drive.changes(Some("10")).unwrap();

    let asked = asked.join().unwrap();
    assert!(
        asked[8].line.starts_with("PATCH /drive/v3/files/A")
            && asked[8].line.contains("addParents=R")
            && asked[8].line.contains("removeParents=BIN"),
        "{}",
        asked[8].line
    );
    assert!(String::from_utf8_lossy(&asked[8].body).contains(r#""name":"a.md""#));
    assert!(!removed(&asked, "A"), "another machine's write was deleted");
    assert_eq!(debts(&home), None, "a debt paid was kept");
}

#[test]
fn what_is_owed_back_out_of_the_bin_is_never_swept_however_old() {
    let home = tempfile::tempdir().unwrap();
    let old = r#"{"id":"X","name":"100-b.md","parents":["BIN"],"version":"2"}"#;
    let owed = vec![Owed::Unbinned {
        name: "b.md".into(),
        id: "X".into(),
        parent: "R".into(),
        bin: "BIN".into(),
    }];
    let written = serde_json::to_vec(&owed).unwrap();
    std::fs::write(home.path().join(tisty_carrier::OWED_BACK), &written).unwrap();
    let (at, asked) = google(vec![
        said("200 OK", &whole(&format!("{BIN_MADE},{A},{old}"))),
        said("503 Service Unavailable", ""),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
        said("200 OK", &whole(&format!("{BIN_MADE},{A},{old}"))),
    ]);
    let drive = drive(at, home.path()).owing_at(home.path());

    drive.list("").unwrap();
    drive.changes(Some("10")).unwrap();

    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 4, "{asked:?}");
    assert!(!removed(&asked, "X"), "a file owed back was swept for good");
    assert_eq!(debts(&home), Some(written), "a debt not paid was let go");
}

#[test]
fn a_debt_is_never_written_over_a_list_that_cannot_be_read() {
    let home = tempfile::tempdir().unwrap();
    let unread = b"[{\"owed\":\"back\",\"na".to_vec();
    std::fs::write(home.path().join(tisty_carrier::OWED_BACK), &unread).unwrap();
    let (at, asked) = google(Vec::new());
    let drive = drive(at, home.path()).owing_at(home.path());

    let owed = drive.owe(Owed::Unbinned {
        name: "b.md".into(),
        id: "X".into(),
        parent: "R".into(),
        bin: "BIN".into(),
    });

    assert!(matches!(owed, Err(Hitch::Broke(_))), "{owed:?}");
    assert_eq!(
        debts(&home),
        Some(unread),
        "the debts already owed were lost"
    );
    assert!(asked.join().unwrap().is_empty());
}

#[test]
fn a_sweep_cut_short_or_too_early_keeps_what_is_left_for_the_next_one() {
    let home = tempfile::tempdir().unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let first = r#"{"id":"X","name":"100-b.md","parents":["BIN"],"version":"2"}"#;
    let second = r#"{"id":"Z","name":"100-d.md","parents":["BIN"],"version":"2"}"#;
    let young = format!(r#"{{"id":"Y","name":"{now}-c.md","parents":["BIN"],"version":"2"}}"#);
    let (at, asked) = google(vec![
        said("200 OK", r#"{"startPageToken":"10"}"#),
        said(
            "200 OK",
            &whole(&format!("{BIN_MADE},{A},{first},{second},{young}")),
        ),
        said("503 Service Unavailable", ""),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
        said("200 OK", first),
        said("204 No Content", ""),
        said("200 OK", second),
        said("204 No Content", ""),
        said("200 OK", r#"{"changes":[],"newStartPageToken":"11"}"#),
    ]);
    let drive = drive(at, home.path());

    drive.changes(None).unwrap();
    drive.changes(Some("10")).unwrap();
    drive.changes(Some("11")).unwrap();

    let asked = asked.join().unwrap();
    assert_eq!(asked.len(), 9, "{asked:?}");
    assert!(
        removed(&asked, "X"),
        "a leftover after a failed read stayed forever"
    );
    assert!(
        removed(&asked, "Z"),
        "a leftover behind a failed read stayed forever"
    );
    assert!(
        asked.iter().all(|one| !one.line.contains("/files/Y")),
        "a deletion another machine may still be checking was swept"
    );
}
