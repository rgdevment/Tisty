use super::*;

#[test]
fn an_event_from_a_newer_tisty_says_so_instead_of_looking_broken() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("000001.tisty");
    std::fs::write(
        &at,
        format!(
            "{{\"v\":{},\"ts\":\"2026-08-13T00:00:00Z\",\"by\":\"dev_a\",\"op\":\"folder.colour\",\"id\":\"01J\",\"d\":{{}}}}\n",
            SCHEMA_VERSION + 1
        ),
    )
    .unwrap();

    let mut out = Vec::new();
    let why = read_segment(&at, &mut out).unwrap_err();

    assert!(
        matches!(why, Error::UnsupportedVersion(_)),
        "it read as corruption: {why:?}"
    );
}

#[test]
fn a_store_written_before_the_schema_moved_still_opens() {
    let room = tempfile::tempdir().unwrap();
    std::fs::create_dir(room.path().join("dev_a")).unwrap();
    let at = room.path().join("dev_a/000001.tisty");
    std::fs::write(
        &at,
        "{\"v\":2,\"ts\":\"2026-08-13T00:00:00Z\",\"by\":\"dev_a\",\"seq\":1,\"op\":\"task.done\",\"id\":\"01JBQ0000000000000000000AA\"}\n",
    )
    .unwrap();

    let mut out = Vec::new();
    read_segment(&at, &mut out).expect("an older store is not a broken store");

    assert_eq!(out.len(), 1);
    assert_eq!(out[0].version, 2, "what was written is not rewritten");
}

/// A machine whose clock runs behind writes after what it read: sorted by stamp, its
/// event lands after the one that let it through, not before.
#[test]
fn a_write_judged_against_the_log_is_stamped_after_all_of_it() {
    let room = tempfile::tempdir().unwrap();
    let ahead = jiff::Timestamp::now() + jiff::SignedDuration::from_secs(3600);
    let id = ulid::Ulid::generate();
    let mut theirs = Store::open(room.path(), DeviceId("dev_b".into())).unwrap();
    theirs
        .append_event(&Event::new(
            DeviceId("dev_b".into()),
            ahead,
            Op::TaskAdd {
                id,
                d: crate::event::TaskAdd::new("opened later", "a0"),
            },
        ))
        .unwrap();

    let mut mine = Store::open(room.path(), DeviceId("dev_a".into())).unwrap();
    let written = mine
        .append_batch_unless(vec![Op::TaskDone { id, filled: false }], |_| false)
        .unwrap()
        .unwrap();

    assert!(
        written[0].timestamp > ahead,
        "{} is not after {ahead}",
        written[0].timestamp
    );
    let all = read_all(room.path()).unwrap();
    assert!(matches!(all.last().unwrap().op, Op::TaskDone { .. }));
}

#[test]
fn an_event_in_another_devices_name_is_let_go_and_the_rest_of_the_segment_read() {
    let room = tempfile::tempdir().unwrap();
    std::fs::create_dir(room.path().join("dev_a")).unwrap();
    let at = room.path().join("dev_a/active.tisty");
    std::fs::write(
        &at,
        format!(
            "{}\n{}\n",
            r#"{"v":6,"ts":"2026-08-01T10:00:00Z","by":"dev_a","op":"task.add","id":"01M14RFT9ECC2B6E4CX4P59XPH","d":{"title":"mine","order":"V"}}"#,
            r#"{"v":6,"ts":"2026-08-01T10:00:01Z","by":"dev_laptop","op":"task.delete","id":"01M14RFT9ECC2B6E4CX4P59XPH"}"#
        ),
    )
    .unwrap();

    let mut out = Vec::new();
    read_segment(&at, &mut out).unwrap();

    assert_eq!(out.len(), 1, "the one in the directory's own name");
    assert_eq!(out[0].device.0, "dev_a");
}

#[test]
fn a_rename_that_fails_takes_its_temporary_with_it() {
    let room = tempfile::tempdir().unwrap();
    let blocked = room.path().join("busy.md");
    std::fs::create_dir(&blocked).unwrap();

    assert!(write_atomic(&blocked, b"x").is_err());

    let left: Vec<_> = std::fs::read_dir(room.path())
        .unwrap()
        .filter_map(|one| one.ok())
        .filter(|one| one.path().extension().is_some_and(|e| e == "tmp"))
        .collect();
    assert!(left.is_empty(), "a temporary was left behind: {left:?}");
}

#[cfg(windows)]
#[test]
fn a_file_held_open_for_a_moment_is_waited_out_rather_than_refused() {
    use std::os::windows::fs::OpenOptionsExt;

    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("a3f1-0001.md");
    std::fs::write(&at, b"before").unwrap();
    let held = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&at)
        .unwrap();

    std::thread::scope(|threads| {
        threads.spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(40));
            drop(held);
        });
        write_atomic(&at, b"after").expect("a file let go of is a file that can be written");
    });

    assert_eq!(std::fs::read_to_string(&at).unwrap(), "after");
}

#[test]
fn two_writers_of_one_file_do_not_share_a_temporary() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("a3f1-0001.md");
    std::fs::write(&at, b"before").unwrap();

    std::thread::scope(|threads| {
        let mut hands = Vec::new();
        for n in 0..8 {
            let at = at.clone();
            hands.push(
                threads.spawn(move || write_atomic(&at, format!("written by {n}").as_bytes())),
            );
        }
        for hand in hands {
            hand.join()
                .unwrap()
                .expect("a concurrent save must not fail");
        }
    });

    let kept = std::fs::read_to_string(&at).unwrap();
    assert!(kept.starts_with("written by"), "{kept}");
}

#[test]
fn a_missing_parent_directory_leaves_nothing_temporary_either() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("nope").join("file.md");

    assert!(write_atomic(&at, b"x").is_err());

    let left: Vec<_> = std::fs::read_dir(room.path())
        .unwrap()
        .filter_map(|one| one.ok())
        .collect();
    assert!(
        left.is_empty(),
        "something was created in a directory that was never made: {left:?}"
    );
}

#[cfg(unix)]
#[test]
fn a_read_only_directory_refuses_the_temporary_and_leaves_nothing_behind() {
    use std::os::unix::fs::PermissionsExt;
    let room = tempfile::tempdir().unwrap();
    let locked = room.path().join("locked");
    std::fs::create_dir(&locked).unwrap();
    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();

    let result = write_atomic(&locked.join("file.md"), b"x");

    std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.is_err());
    let left: Vec<_> = std::fs::read_dir(&locked)
        .unwrap()
        .filter_map(|one| one.ok())
        .collect();
    assert!(
        left.is_empty(),
        "a temporary survived in a directory with no write permission: {left:?}"
    );
}

#[test]
fn poured_does_not_partially_write_when_the_temporary_path_is_a_directory() {
    let room = tempfile::tempdir().unwrap();
    let dir_as_tmp = room.path().join("oops");
    std::fs::create_dir(&dir_as_tmp).unwrap();

    assert!(poured(&dir_as_tmp, b"x").is_err());
}

#[test]
fn nothing_temporary_is_left_behind() {
    let tmp = tempfile::tempdir().unwrap();
    let at = tmp.path().join("a3f1-0001.md");

    write_atomic(&at, b"one").unwrap();
    write_atomic(&at, b"two").unwrap();

    let left: Vec<_> = std::fs::read_dir(tmp.path())
        .unwrap()
        .filter_map(|one| one.ok())
        .map(|one| one.file_name().to_string_lossy().into_owned())
        .filter(|named| named.contains("tmp"))
        .collect();
    assert!(left.is_empty(), "{left:?}");
}
