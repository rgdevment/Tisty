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
        &now(),
        'w',
        Gravity::Warn,
        channel::SYNC,
        "folder unreachable",
        &[],
    );

    assert!(line.starts_with("2026-08-11 17:04:03.000-04"), "{line}");
    assert!(line.contains("WARN"), "{line}");
    assert!(line.contains("sync"), "{line}");
    assert!(line.contains("folder unreachable"), "{line}");
    assert!(line.ends_with('\n'));
}

#[test]
fn facts_are_named_beside_the_words() {
    let line = lined(
        &now(),
        'w',
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
        &now(),
        'w',
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
    let said = Fact::Id("attachments/severance-juan-perez-91f2.pdf".into()).shown();

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
    assert_eq!(seen.len(), 4, "{seen:?}");
    assert!(seen[1].contains("first"), "{seen:?}");
    assert!(seen[2].contains("second"), "{seen:?}");
    assert!(seen[3].contains("kept"), "{seen:?}");
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
        2,
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
    assert!(said.contains("witness_test.rs:"), "{said}");
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
    let line = lined(&now(), 'w', Gravity::Warn, channel::STORE, "a\nb", &[]);

    assert_eq!(line.matches('\n').count(), 1, "{line}");
}

#[test]
fn the_moment_is_written_with_its_offset_in_full() {
    let line = lined(&now(), 'w', Gravity::Warn, channel::STORE, "x", &[]);

    assert!(line.starts_with("2026-08-11 17:04:03.000-04:00"), "{line}");
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

#[test]
fn a_hostile_round_leaves_neither_its_token_nor_its_signed_address_in_the_log() {
    let token = ["ya29", ".a0AfH6SMBx7kq2Lw9ZpQ4rT8vYc1"].concat();
    let why = format!(
        "GET https://www.googleapis.com/drive/v3/files/x?alt=media&sig=Zx9 with Bearer {token}: 401"
    );

    let line = lined(
        &now(),
        'w',
        Gravity::Warn,
        channel::CLOUD,
        "the provider turned the token down",
        &[("why", Fact::Why(why))],
    );

    assert!(!line.contains(&token), "{line}");
    assert!(!line.contains("sig=Zx9"), "{line}");
    assert!(line.contains("401"), "{line}");
}

#[test]
fn a_name_whose_lower_case_is_longer_is_hidden_without_cutting_a_letter() {
    assert_eq!(without(r"C:\Users\İlker", "İlker"), r"C:\Users\···");
    assert_eq!(
        without(r"İİİ C:\Users\ilker\x", "ilker"),
        r"İİİ C:\Users\···\x"
    );
}

#[test]
fn the_same_warning_over_and_over_is_written_once() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    keeps(file(&paths), false);

    for _ in 0..5 {
        warn(
            channel::SYNC,
            "the same thing failed again",
            &[("n", Fact::Count(1))],
        );
    }
    warn(
        channel::SYNC,
        "the same thing failed again",
        &[("n", Fact::Count(2))],
    );

    let seen = recent(&paths, 20);
    assert_eq!(seen.len(), 3, "{seen:?}");
}

#[test]
fn a_standing_note_found_every_round_is_written_again_only_when_it_changes() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    keeps(file(&paths), false);

    for _ in 0..5 {
        standing(
            channel::SYNC,
            "files nothing names were left up there",
            &[("left", Fact::Count(14))],
        );
    }
    standing(
        channel::SYNC,
        "files nothing names were left up there",
        &[("left", Fact::Count(15))],
    );
    note(channel::SYNC, "an ordinary note", &[]);
    note(channel::SYNC, "an ordinary note", &[]);

    let seen = recent(&paths, 20);
    assert_eq!(seen.len(), 5, "{seen:?}");
    stops();
}

#[test]
fn a_line_says_which_process_wrote_it() {
    let line = lined(&now(), mark::DOOR, Gravity::Note, channel::DOOR, "x", &[]);

    let pid = std::process::id().to_string();
    assert!(line.contains(&format!("  d {pid}")), "{line}");
}

#[test]
fn a_new_file_begins_by_saying_what_wrote_it() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    opens(&paths, mark::WINDOW);

    warn(channel::SYNC, "first", &[]);
    warn(channel::SYNC, "second", &[]);

    let seen = recent(&paths, 10);
    assert_eq!(seen.len(), 3, "{seen:?}");
    assert!(seen[0].contains("this log file began"), "{seen:?}");
    assert!(seen[0].contains(std::env::consts::OS), "{seen:?}");
    assert!(seen[0].contains("  w "), "{seen:?}");
    stops();
}

#[test]
fn three_files_are_kept_and_the_oldest_falls_away() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("tisty.log");
    std::fs::write(rolled(&at), "older").unwrap();
    std::fs::write(&at, vec![b'x'; ROLLS_AT as usize]).unwrap();

    roll(&at);

    assert_eq!(std::fs::read_to_string(oldest(&at)).unwrap(), "older");
    assert!(rolled(&at).exists());
    assert!(!at.exists());
}

#[test]
fn a_marker_file_asks_for_the_finest_trail() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    std::fs::create_dir_all(paths.private()).unwrap();
    std::fs::write(all_marker(&paths), "").unwrap();

    opens(&paths, mark::WINDOW);

    assert!(keeping_all());
    stops();
}

#[test]
fn a_path_is_written_without_doubled_separators() {
    let said = Fact::Path(PathBuf::from(r"D:\store\dev_a")).shown();

    assert_eq!(said, r#""D:\store\dev_a""#);
}

#[test]
fn an_attachment_named_inside_a_reason_is_cut() {
    let said = Fact::Why(
        r"attachments/severance-juan.pdf: missing, and C:\x\docs\salary.md was changed; retry in 30s"
            .into(),
    )
    .shown();

    assert!(!said.contains("severance"), "{said}");
    assert!(!said.contains("salary"), "{said}");
    assert!(said.contains("attachments/…: missing"), "{said}");
    assert!(said.contains("was changed; retry in 30s"), "{said}");
}

#[test]
fn a_path_with_a_quote_or_a_control_character_cannot_break_the_line() {
    let said = Fact::Path(PathBuf::from("store\"x\u{1b}[31m")).shown();

    assert_eq!(said, r#""store\"x\u{1b}[31m""#);
}

#[test]
fn the_newest_lines_are_read_from_the_end_and_older_files_only_when_needed() {
    let _alone = super::ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let tmp = tempfile::tempdir().unwrap();
    let paths = crate::paths::Paths::new(tmp.path().join("data"), tmp.path().join("config"));
    let at = file(&paths);
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(oldest(&at), "a\nb\n").unwrap();
    std::fs::write(rolled(&at), "c\nd\n").unwrap();
    let live: String = (0..3000).map(|n| format!("line {n}\n")).collect();
    std::fs::write(&at, live).unwrap();

    assert_eq!(recent(&paths, 2), vec!["line 2998", "line 2999"]);
    let all = recent(&paths, 3003);
    assert_eq!(all.len(), 3003);
    assert_eq!(all[0], "b");
    assert_eq!(all[2], "d");
    assert_eq!(all[3], "line 0");
}

#[test]
fn a_folder_that_only_ends_like_one_is_left_alone() {
    let said = Fact::Why("mydocs/plan.md".into()).shown();

    assert!(said.contains("plan.md"), "{said}");
}

#[test]
fn an_account_address_inside_a_reason_is_hidden() {
    let said = Fact::Why("Elsewhere { found: \"maria@example.com\" }".into()).shown();

    assert!(!said.contains("maria"), "{said}");
}
