use super::*;

fn room() -> (tempfile::TempDir, std::path::PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("cache").join(USED);
    (room, at)
}

#[test]
fn an_attachment_nobody_reached_for_has_no_day() {
    let (_room, at) = room();

    assert_eq!(last(&at, "attachments/ab/charla-00000000.mp4"), None);
    assert!(every(&at).is_empty());
}

#[test]
fn the_day_it_was_reached_for_is_remembered_past_this_launch() {
    let (_room, at) = room();
    let one = "attachments/ab/charla-00000000.mp4";

    used_on(&at, one, 1_700_000_000);

    assert!(at.is_file(), "nothing was written down");
    assert_eq!(last(&at, one), Some(1_700_000_000));
    assert_eq!(every(&at).len(), 1);
}

#[test]
fn reaching_for_it_again_moves_the_day_and_adds_no_row() {
    let (_room, at) = room();
    let one = "attachments/ab/charla-00000000.mp4";

    used_on(&at, one, 1_700_000_000);
    used_on(&at, one, 1_800_000_000);

    assert_eq!(last(&at, one), Some(1_800_000_000));
    assert_eq!(every(&at).len(), 1, "one attachment, one row");
}

#[test]
fn reaching_for_it_again_the_same_day_writes_nothing() {
    let (_room, at) = room();
    let one = "attachments/ab/charla-00000000.mp4";

    used_on(&at, one, 1_700_000_000);
    let first = std::fs::metadata(&at).unwrap().modified().unwrap();

    used_on(&at, one, 1_700_000_000 + NO_SOONER_THAN - 1);

    assert_eq!(last(&at, one), Some(1_700_000_000), "the day did not move");
    assert_eq!(
        std::fs::metadata(&at).unwrap().modified().unwrap(),
        first,
        "the file was written again for nothing"
    );

    used_on(&at, one, 1_700_000_000 + NO_SOONER_THAN);

    assert_eq!(
        last(&at, one),
        Some(1_700_000_000 + NO_SOONER_THAN),
        "a day later it does move"
    );
}

#[test]
fn each_attachment_keeps_its_own_day() {
    let (_room, at) = room();

    used_on(&at, "attachments/ab/charla-00000000.mp4", 1_700_000_000);
    used_on(&at, "attachments/cd/nota-11111111.txt", 1_800_000_000);

    assert_eq!(
        last(&at, "attachments/ab/charla-00000000.mp4"),
        Some(1_700_000_000)
    );
    assert_eq!(
        last(&at, "attachments/cd/nota-11111111.txt"),
        Some(1_800_000_000)
    );
}

#[test]
fn what_is_written_down_stops_growing_and_the_longest_untouched_goes_first() {
    let (_room, at) = room();
    let full: Used = (0..KEPT_AT_MOST)
        .map(|n| {
            (
                format!("attachments/ab/one-{n:08}.txt"),
                2_000_000_000 + n as u64,
            )
        })
        .collect();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, serde_json::to_vec(&full).unwrap()).unwrap();

    let held = every(&at);
    assert_eq!(held.len(), KEPT_AT_MOST);
    assert!(held.contains_key("attachments/ab/one-00000000.txt"));

    used_on(&at, "attachments/cd/late-99999999.txt", 3_000_000_000);

    let held = every(&at);
    assert_eq!(held.len(), KEPT_AT_MOST, "it did not grow");
    assert!(
        !held.contains_key("attachments/ab/one-00000000.txt"),
        "the one nobody had reached for in longest is the one that went"
    );
    assert_eq!(
        last(&at, "attachments/cd/late-99999999.txt"),
        Some(3_000_000_000)
    );
}

#[test]
fn a_file_nobody_can_parse_reads_as_nothing_written_down_yet() {
    let (_room, at) = room();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(&at, b"not json at all").unwrap();

    assert!(every(&at).is_empty());

    used_on(&at, "attachments/ab/charla-00000000.mp4", 1_700_000_000);

    assert_eq!(
        last(&at, "attachments/ab/charla-00000000.mp4"),
        Some(1_700_000_000)
    );
}

#[test]
fn the_day_it_says_is_the_day_the_clock_says() {
    let (_room, at) = room();
    let before = now();

    used(&at, "attachments/ab/charla-00000000.mp4");

    let said = last(&at, "attachments/ab/charla-00000000.mp4").expect("a day");
    assert!(said >= before, "it cannot have been reached for before now");
    assert!(said <= now(), "nor after");
}
