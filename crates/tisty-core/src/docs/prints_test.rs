use super::{Prints, Seen};

#[test]
fn a_file_with_a_time_before_1970_still_weighs_what_it_holds() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("acta.md");
    std::fs::write(&at, "# Acta\n\ncuerpo\n").unwrap();
    let long_ago = std::time::UNIX_EPOCH - std::time::Duration::from_secs(3600);
    let set = std::fs::File::options()
        .write(true)
        .open(&at)
        .unwrap()
        .set_modified(long_ago);
    if set.is_err() {
        return;
    }

    let seen = Prints::default().seen(&at).unwrap();

    let Seen::Held { weighs, print } = seen else {
        panic!("a plain file was read as a link");
    };
    assert!(weighs > 0, "a body with words was weighed as empty");
    assert!(print.is_some());
}

#[test]
fn a_link_is_told_apart_before_anything_of_it_is_read() {
    let room = tempfile::tempdir().unwrap();
    let real = room.path().join("real.md");
    std::fs::write(&real, "# Real\n").unwrap();
    let link = room.path().join("link.md");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&real, &link).unwrap();
    #[cfg(windows)]
    if std::os::windows::fs::symlink_file(&real, &link).is_err() {
        return;
    }

    assert_eq!(Prints::default().seen(&link).unwrap(), Seen::Linked);
}

#[test]
fn a_print_kept_for_a_body_is_forgotten_when_asked() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("acta.md");
    std::fs::write(&at, "# Uno\n").unwrap();
    let was = std::fs::metadata(&at).unwrap().modified().unwrap();
    let mut prints = Prints::default();
    let first = prints.seen(&at).unwrap();

    std::fs::write(&at, "# Dos\n").unwrap();
    let set = std::fs::File::options()
        .write(true)
        .open(&at)
        .unwrap()
        .set_modified(was);
    if set.is_err() {
        return;
    }
    assert_eq!(
        prints.seen(&at).unwrap(),
        first,
        "a body of the same size and time was read again"
    );

    prints.forget(&at);

    assert_ne!(prints.seen(&at).unwrap(), first);
}
