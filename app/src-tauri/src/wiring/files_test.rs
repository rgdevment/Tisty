use super::*;

#[test]
fn a_file_that_is_not_there_is_empty_and_one_that_cannot_be_read_is_not() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("settings.json");

    assert_eq!(read(&at), Read::Missing);
    assert_eq!(text(&at).unwrap(), "");
    std::fs::write(&at, b"Jos\xE9").unwrap();
    assert_eq!(read(&at), Read::Unread);
    assert!(matches!(text(&at), Err(Stuck::Unread(_))));
    std::fs::write(&at, "José").unwrap();
    assert_eq!(read(&at), Read::Text("José".into()));
    assert_eq!(read(tmp.path()), Read::Unread, "a folder is not a file");
}

#[test]
fn a_file_seen_empty_while_the_agent_writes_it_is_read_again() {
    for (seen, said) in [
        (["", "one"], "one"),
        (["", ""], ""),
        (["one", "two"], "one"),
    ] {
        let mut looks = seen.into_iter();
        let read = settled(|| Read::Text(looks.next().unwrap().into()));
        assert_eq!(read, Read::Text(said.into()));
    }
}

#[test]
fn one_operation_keeps_one_copy_of_what_the_file_held_before_it() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("settings.json");
    std::fs::write(&at, "one").unwrap();
    let last = tmp.path().join("settings.json.before-tisty-last");

    let mut writes = Writes::default();
    writes.change(&at, "one", "two", false).unwrap();
    writes.change(&at, "two", "three", false).unwrap();
    assert_eq!(std::fs::read_to_string(&at).unwrap(), "three");
    assert_eq!(std::fs::read_to_string(&last).unwrap(), "one");

    Writes::default().remove(&at, "three").unwrap();
    assert!(!at.exists());
    assert_eq!(std::fs::read_to_string(&last).unwrap(), "three");
    Writes::default().remove(&at, "").unwrap();
}

fn linked(real: &Path, link: &Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(real, link).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_file(real, link).unwrap();
}

#[test]
#[cfg_attr(
    windows,
    ignore = "a link needs a privilege Windows gives only to some accounts"
)]
fn a_link_is_written_through_and_stays_a_link() {
    let tmp = tempfile::tempdir().unwrap();
    let real = tmp.path().join("dotfiles.json");
    let link = tmp.path().join("settings.json");
    std::fs::write(&real, "was").unwrap();
    linked(&real, &link);

    Writes::default()
        .change(&link, "was", "now", false)
        .unwrap();

    assert!(std::fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(std::fs::read_to_string(&real).unwrap(), "now");
}

#[test]
#[cfg_attr(
    windows,
    ignore = "a link needs a privilege Windows gives only to some accounts"
)]
fn a_link_that_leads_nowhere_is_never_taken_for_an_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let link = tmp.path().join("settings.json");
    linked(&tmp.path().join("nowhere.json"), &link);

    assert_eq!(read(&link), Read::Unread);
}

#[cfg(unix)]
#[test]
fn a_private_file_and_its_copies_stay_private_and_an_open_one_open() {
    use std::os::unix::fs::PermissionsExt;
    for mode in [0o600, 0o644] {
        let tmp = tempfile::tempdir().unwrap();
        let at = tmp.path().join("settings.json");
        std::fs::write(&at, "was").unwrap();
        std::fs::set_permissions(&at, Permissions::from_mode(mode)).unwrap();

        Writes::default().change(&at, "was", "now", false).unwrap();

        for end in ["", ".before-tisty", ".before-tisty-last"] {
            let one = tmp.path().join(format!("settings.json{end}"));
            let now = std::fs::metadata(&one).unwrap().permissions().mode();
            assert_eq!(now & 0o777, mode, "{end}");
        }
    }
}
