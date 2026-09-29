use super::*;

fn a_part(at: &Path, named: &str, ago: i64) -> std::path::PathBuf {
    let one = at.join(named);
    std::fs::write(&one, b"half of something").unwrap();
    let when = std::time::UNIX_EPOCH
        + std::time::Duration::from_secs((jiff::Timestamp::now().as_second() - ago) as u64);
    std::fs::File::options()
        .write(true)
        .open(&one)
        .unwrap()
        .set_modified(when)
        .unwrap();
    one
}

#[test]
fn the_mark_is_drawn_once_and_no_other_machine_can_wear_it() {
    assert_eq!(ours(), ours());
    assert_eq!(ours().len(), 26);
    assert!(named(3).contains(ours()));
    assert!(
        beside(Path::new("a.tisty"), 3)
            .display()
            .to_string()
            .contains(ours())
    );
    assert_ne!(ours(), "01ARZ3NDEKTSV4RRFFQ69G5FAV");
}

#[test]
fn a_copy_that_may_still_be_in_flight_is_left_alone_whoever_started_it() {
    let room = tempfile::tempdir().unwrap();
    let now = jiff::Timestamp::now().as_second();
    let theirs = a_part(room.path(), ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part", 60);
    let mine = a_part(room.path(), &named(7), 60);

    assert!(
        !spent_by(&theirs, ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part", now),
        "a minute old is a copy in flight, not one nobody is coming back for"
    );
    assert!(!spent_by(&mine, &named(7), now));
}

#[test]
fn what_another_run_left_behind_a_day_ago_goes_and_our_own_never_does() {
    let room = tempfile::tempdir().unwrap();
    let now = jiff::Timestamp::now().as_second();
    let theirs = a_part(
        room.path(),
        ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part",
        25 * 60 * 60,
    );
    let mine = a_part(room.path(), &named(7), 25 * 60 * 60);

    assert!(spent_by(&theirs, ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part", now));
    assert!(
        !spent_by(&mine, &named(7), now),
        "this run may be writing it this very second, whatever its age says"
    );
}

#[test]
fn an_attachment_that_happens_to_end_in_part_is_not_a_leftover() {
    let room = tempfile::tempdir().unwrap();
    let now = jiff::Timestamp::now().as_second();
    let kept = a_part(
        room.path(),
        "pelicula-a3f90001.part",
        LEFT_BEHIND_AFTER * 30,
    );
    let odd = a_part(room.path(), "lo.que.sea.part", LEFT_BEHIND_AFTER * 30);

    assert!(
        !spent_by(&kept, "pelicula-a3f90001.part", now),
        "somebody attached a half-downloaded film and the round took it"
    );
    assert!(!spent_by(&odd, "lo.que.sea.part", now));
}

#[test]
fn nothing_but_a_part_file_is_ever_swept() {
    let room = tempfile::tempdir().unwrap();
    let now = jiff::Timestamp::now().as_second();
    let kept = a_part(room.path(), "active.tisty", LEFT_BEHIND_AFTER * 30);

    assert!(!spent_by(&kept, "active.tisty", now));
}

#[test]
fn what_is_not_there_is_not_swept_either() {
    let now = jiff::Timestamp::now().as_second();

    assert!(!spent_by(
        Path::new("nowhere/.01.0.part"),
        ".01.0.part",
        now
    ));
}

#[test]
fn a_day_is_what_it_takes_to_call_a_copy_abandoned() {
    let room = tempfile::tempdir().unwrap();
    let now = jiff::Timestamp::now().as_second();
    let young = a_part(
        room.path(),
        ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part",
        23 * 60 * 60,
    );
    let old = a_part(
        room.path(),
        ".01BX5ZZKBKACTAV9WEVGEMMVRZ.0.part",
        25 * 60 * 60,
    );

    assert!(!spent_by(&young, ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part", now));
    assert!(spent_by(&old, ".01BX5ZZKBKACTAV9WEVGEMMVRZ.0.part", now));
    assert!(
        !spent_by(&old, ".01BX5ZZKBKACTAV9WEVGEMMVRZ.part", now),
        "a name with no turn in it was never one of ours"
    );
}

#[test]
fn the_one_the_sweepers_call_asks_the_clock_itself() {
    let room = tempfile::tempdir().unwrap();
    let young = a_part(room.path(), ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part", 60);
    let old = a_part(
        room.path(),
        ".01BX5ZZKBKACTAV9WEVGEMMVRZ.0.part",
        25 * 60 * 60,
    );

    assert!(!spent(&young, ".01ARZ3NDEKTSV4RRFFQ69G5FAV.0.part"));
    assert!(spent(&old, ".01BX5ZZKBKACTAV9WEVGEMMVRZ.0.part"));
}

#[test]
fn only_a_name_this_tisty_could_have_written_is_a_leftover() {
    let room = tempfile::tempdir().unwrap();
    let now = jiff::Timestamp::now().as_second();
    let old = LEFT_BEHIND_AFTER * 30;

    for named in [
        ".abc.0.part",
        "..0.part",
        ".pelicula-a3f90001.0.part",
        ".01ARZ3NDEKTSV4RRFFQ69G5FAV.uno.part",
        ".01ARZ3NDEKTSV4RRFFQ69G5FAV..part",
        ".01ARZ3NDEKTSV4RRFFQ69G5FA!.0.part",
    ] {
        let at = a_part(room.path(), named, old);
        assert!(
            !spent_by(&at, named, now),
            "{named} was never written by this program and the round took it"
        );
    }

    let mine = a_part(room.path(), ".01ARZ3NDEKTSV4RRFFQ69G5FAV.12.part", old);
    assert!(spent_by(&mine, ".01ARZ3NDEKTSV4RRFFQ69G5FAV.12.part", now));
}
