use super::*;

fn now() -> jiff::Zoned {
    "2026-08-11T17:04:03-04[America/Santiago]".parse().unwrap()
}

#[test]
fn a_log_another_writer_holds_is_not_rolled_out_from_under_them() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("tisty.log");
    std::fs::write(&at, vec![b'x'; ROLLS_AT as usize + 1]).unwrap();

    let gate = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(at.with_extension("log.lock"))
        .unwrap();
    gate.try_lock().unwrap();

    roll(&at);
    assert!(at.exists());
    assert!(!rolled(&at).exists());

    gate.unlock().unwrap();
    roll(&at);
    assert!(rolled(&at).exists());
}

#[test]
fn a_line_carries_the_moment_the_gravity_the_channel_and_the_words() {
    let line = lined(
        now(),
        Gravity::Warn,
        channel::SYNC,
        "folder unreachable",
        &[],
    );

    assert!(line.starts_with("2026-08-11 17:04:03-04"), "{line}");
    assert!(line.contains("WARN"), "{line}");
    assert!(line.contains("sync"), "{line}");
    assert!(line.contains("folder unreachable"), "{line}");
    assert!(line.ends_with('\n'));
}

#[test]
fn facts_are_named_beside_the_words() {
    let line = lined(
        now(),
        Gravity::Error,
        channel::STORE,
        "segment unreadable",
        &[("line", Fact::Count(41)), ("code", Fact::Code("badJson"))],
    );

    assert!(line.contains("line=41"), "{line}");
    assert!(line.contains("code=badJson"), "{line}");
}

#[test]
fn the_account_name_never_reaches_the_file() {
    assert_eq!(
        without(
            r"store C:\Users\rgdevment\tisty, sync G:\rgdevment\copies",
            "rgdevment"
        ),
        r"store C:\Users\···\tisty, sync G:\···\copies"
    );
}

#[test]
fn a_reason_that_spans_lines_is_flattened_into_one() {
    let line = lined(
        now(),
        Gravity::Error,
        channel::CONFIG,
        "config unreadable",
        &[("why", Fact::Why("expected a table\n  at line 3".into()))],
    );

    assert_eq!(line.matches('\n').count(), 1, "{line}");
    assert!(line.contains("expected a table at line 3"), "{line}");
}

#[test]
fn nothing_under_attachments_is_named() {
    let at: PathBuf = ["store", "attachments", "2026-08", "severance-juan.pdf"]
        .iter()
        .collect();

    let said = kept_short(&at);

    assert!(!said.contains("severance"), "{said}");
    assert!(said.contains("attachments"), "{said}");
    assert!(said.ends_with('…'), "{said}");
}

#[test]
fn nothing_under_attachments_is_named_however_it_is_handed_over() {
    let said = Fact::Id("attachments/ab/severance-juan-perez-91f2.pdf".into()).shown();

    assert!(!said.contains("severance"), "{said}");
    assert!(said.contains("attachments"), "{said}");
}

#[test]
fn an_identifier_that_names_nobody_is_still_readable() {
    let said = Fact::Id("01JBQ0000000000000000000AA".into()).shown();

    assert_eq!(said, "01JBQ0000000000000000000AA");
}

#[test]
fn a_path_that_names_nobody_is_left_whole() {
    let at: PathBuf = ["store", "dev_a", "000001.jsonl"].iter().collect();

    assert!(kept_short(&at).ends_with("000001.jsonl"), "{at:?}");
}

#[test]
fn a_one_letter_account_is_left_alone() {
    assert_eq!(without(r"C:\data\attachments", "a"), r"C:\data\attachments");
}

#[test]
fn notes_go_nowhere_until_somewhere_is_named() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    warn(channel::STORE, "nobody is listening", &[]);
}

#[test]
fn gravity_sorts_the_way_it_reads() {
    assert!(Gravity::Trace < Gravity::Note);
    assert!(Gravity::Note < Gravity::Warn);
    assert!(Gravity::Warn < Gravity::Error);
    assert!(Gravity::Error < Gravity::Fatal);
}

#[test]
fn the_rolled_file_sits_beside_the_live_one() {
    assert_eq!(
        rolled(&PathBuf::from("/somewhere/tisty.log")),
        PathBuf::from("/somewhere/tisty.log.1")
    );
}

#[test]
fn what_is_written_can_be_read_back_newest_last() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    keeps(file(&paths), false);

    warn(channel::SYNC, "first", &[]);
    error(channel::SYNC, "second", &[]);
    note(
        channel::SYNC,
        "kept, so a shared log says what happened",
        &[],
    );
    trace(channel::SYNC, "quiet unless asked for", &[]);

    let seen = recent(&paths, 10);
    assert_eq!(seen.len(), 3, "{seen:?}");
    assert!(seen[0].contains("first"), "{seen:?}");
    assert!(seen[1].contains("second"), "{seen:?}");
    assert!(seen[2].contains("kept"), "{seen:?}");
    assert!(weighs(&paths) > 0);

    forget(&paths).unwrap();
    assert!(recent(&paths, 10).is_empty());
    assert_eq!(weighs(&paths), 0);
    stops();
}

#[test]
fn the_finest_trail_is_kept_only_when_it_is_asked_for() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));

    keeps(file(&paths), true);
    trace(channel::SYNC, "asked for", &[]);
    assert_eq!(
        recent(&paths, 10).len(),
        1,
        "verbose keeps the finest trail"
    );

    forget(&paths).unwrap();
    keeps(file(&paths), false);
    trace(channel::SYNC, "not asked for", &[]);
    assert!(
        recent(&paths, 10).is_empty(),
        "and a log somebody shares is not drowned in it"
    );
    stops();
}

#[test]
fn a_panic_leaves_a_line_behind() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    keeps(file(&paths), false);

    let quiet = std::panic::take_hook();
    catches(channel::WINDOW);
    let _ = std::panic::catch_unwind(|| panic!("the sky fell"));
    let _ = std::panic::take_hook();
    std::panic::set_hook(quiet);

    let seen = recent(&paths, 10);
    // Another test writing a note of its own must not decide whether this one passes.
    let said = seen
        .iter()
        .rfind(|one| one.contains("FATAL"))
        .expect("a line");
    assert!(said.contains("FATAL"), "{said}");
    assert!(said.contains("panicked"), "{said}");
    assert!(said.contains("witness"), "{said}");
    assert!(!said.contains("the sky fell"), "{said}");
    stops();
}

#[test]
fn the_account_name_goes_whatever_case_it_is_written_in() {
    let said = without(r"D:\Dropbox\MARIO\tisty and C:\Users\mario\x", "Mario");

    assert!(!said.to_lowercase().contains("mario"), "{said}");
}

#[test]
fn a_name_too_short_to_replace_safely_is_left_whole() {
    assert_eq!(
        without(r"C:\data\attachments", "ab"),
        r"C:\data\attachments"
    );
}

#[test]
fn a_message_with_a_line_break_still_makes_one_note() {
    let line = lined(now(), Gravity::Warn, channel::STORE, "a\nb", &[]);

    assert_eq!(line.matches('\n').count(), 1, "{line}");
}

#[test]
fn the_moment_is_written_with_its_offset_in_full() {
    let line = lined(now(), Gravity::Warn, channel::STORE, "x", &[]);

    assert!(line.starts_with("2026-08-11 17:04:03-04:00"), "{line}");
}

#[test]
fn a_torn_character_does_not_blank_the_whole_file() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    keeps(file(&paths), false);
    warn(channel::STORE, "before the tear", &[]);
    let mut raw = std::fs::read(file(&paths)).unwrap();
    raw.extend_from_slice(&[0xff, 0xfe, b'\n']);
    std::fs::write(file(&paths), raw).unwrap();

    let seen = recent(&paths, 10);

    assert!(
        seen.iter().any(|one| one.contains("before the tear")),
        "{seen:?}"
    );
    stops();
}
