use super::*;

fn room() -> (tempfile::TempDir, crate::Paths) {
    let room = tempfile::tempdir().unwrap();
    let paths = crate::Paths::new(room.path().join("data"), room.path().join("config"));
    (room, paths)
}

#[test]
fn a_machine_makes_its_key_once_and_finds_the_same_one_after() {
    let (_room, paths) = room();
    let who = DeviceId("dev_a".into());

    let first = mine(&paths, &who).expect("a key was made");
    let again = mine(&paths, &who).expect("and found again");

    assert_eq!(first.to_bytes(), again.to_bytes(), "it made a second key");
    assert_eq!(shown(&first), shown(&again));
}

#[test]
fn two_machines_do_not_share_a_key() {
    let (_room, paths) = room();

    let one = mine(&paths, &DeviceId("dev_a".into())).unwrap();
    let two = mine(&paths, &DeviceId("dev_b".into())).unwrap();

    assert_ne!(one.to_bytes(), two.to_bytes());
    assert_ne!(shown(&one), shown(&two));
}

#[test]
fn the_key_never_lands_anywhere_that_travels() {
    let (_room, paths) = room();
    let who = DeviceId("dev_a".into());

    mine(&paths, &who).unwrap();

    let at = kept_at(&paths, &who).unwrap();
    assert!(at.is_file(), "nothing was written down");
    assert!(
        at.starts_with(paths.private()),
        "the key is kept where the store is carried from: {}",
        at.display()
    );
    assert!(
        !at.starts_with(paths.data()),
        "the key sits under what sync copies"
    );
}

#[test]
fn what_travels_is_the_public_half_and_it_reads_back() {
    let (_room, paths) = room();
    let key = mine(&paths, &DeviceId("dev_a".into())).unwrap();

    let said = shown(&key);

    assert_eq!(said.len(), 64, "{said}");
    assert!(said.chars().all(|one| one.is_ascii_hexdigit()));
    assert_eq!(
        read(&said).expect("it reads back"),
        key.verifying_key(),
        "what travelled is not the half that checks"
    );
    assert!(
        !said.contains(&hexed(&key.to_bytes())),
        "the half that signs went with it"
    );
}

#[test]
fn a_public_key_that_is_not_one_is_turned_away() {
    assert!(read("").is_none());
    assert!(read("not hex at all, and not sixty four either").is_none());
    assert!(read(&"z".repeat(64)).is_none());
    assert!(read(&"ab".repeat(31)).is_none(), "too short");
}

#[test]
fn a_name_a_directory_cannot_hold_gets_no_key() {
    let (_room, paths) = room();

    assert!(kept_at(&paths, &DeviceId("../elsewhere".into())).is_none());
    assert!(kept_at(&paths, &DeviceId(String::new())).is_none());
}

#[test]
fn what_was_kept_as_a_key_and_is_not_one_is_set_aside_before_a_new_one_lands() {
    let (_room, paths) = room();
    let who = DeviceId("dev_a".into());
    let at = kept_at(&paths, &who).unwrap();
    std::fs::create_dir_all(paths.private()).unwrap();
    std::fs::write(&at, b"three bytes short of a key").unwrap();

    let made = mine(&paths, &who).expect("a machine with a torn key file stayed mute for good");

    assert_eq!(
        mine(&paths, &who).unwrap().to_bytes(),
        made.to_bytes(),
        "it made a second key on the next call"
    );
    assert!(
        kept_beside(&paths, &at, b"three bytes short of a key"),
        "what somebody kept there was written over instead of set aside"
    );
}

fn kept_beside(paths: &crate::Paths, at: &std::path::Path, held: &[u8]) -> bool {
    std::fs::read_dir(paths.private())
        .unwrap()
        .filter_map(|one| one.ok().map(|one| one.path()))
        .filter(|one| one != at)
        .any(|one| std::fs::read(one).is_ok_and(|there| there == held))
}

#[test]
fn the_tip_moves_with_every_line_and_with_what_came_before() {
    let one = tip_of(NOTHING_BEFORE, b"first\n");
    let two = tip_of(NOTHING_BEFORE, b"first\nsecond\n");
    let onward = tip_of(one, b"second\n");

    assert_ne!(one, two, "a second line left the tip where it was");
    assert_eq!(
        two, onward,
        "a segment does not carry on from the one before"
    );
    assert_ne!(
        tip_of([1u8; 32], b"first\n"),
        one,
        "what came before did not reach the tip"
    );
}

#[test]
fn a_line_changed_anywhere_moves_the_tip() {
    let said = tip_of(NOTHING_BEFORE, b"one\ntwo\nthree\n");

    assert_ne!(
        said,
        tip_of(NOTHING_BEFORE, b"one\ntwo!\nthree\n"),
        "middle"
    );
    assert_ne!(said, tip_of(NOTHING_BEFORE, b"one\ntwo\n"), "cut short");
    assert_ne!(
        said,
        tip_of(NOTHING_BEFORE, b"two\none\nthree\n"),
        "reordered"
    );
}

fn about<'a>(device: &'a str, segment: &'a str) -> About<'a> {
    About { device, segment }
}

#[test]
fn what_a_machine_signed_it_can_answer_for_and_nobody_else_can() {
    let (_room, paths) = room();
    let ours = mine(&paths, &DeviceId("dev_a".into())).unwrap();
    let other = mine(&paths, &DeviceId("dev_b".into())).unwrap();
    let tip = tip_of(NOTHING_BEFORE, b"what it wrote\n");
    let one = about("dev_a", "active.tisty");

    let said = signed(&ours, &one, &tip);

    assert_eq!(
        holds(&ours.verifying_key(), &one, &said),
        Some(tip),
        "it could not answer for its own signature"
    );
    assert!(
        holds(&other.verifying_key(), &one, &said).is_none(),
        "another machine's key answered for it"
    );
}

#[test]
fn a_signature_only_answers_for_the_segment_and_the_machine_it_was_made_for() {
    let (_room, paths) = room();
    let key = mine(&paths, &DeviceId("dev_a".into())).unwrap();
    let by = key.verifying_key();
    let tip = tip_of(NOTHING_BEFORE, b"what it wrote\n");

    let said = signed(&key, &about("dev_a", "000004.tisty"), &tip);

    assert!(
        holds(&by, &about("dev_a", "000005.tisty"), &said).is_none(),
        "a signature was moved onto another segment and still answered"
    );
    assert!(
        holds(&by, &about("dev_b", "000004.tisty"), &said).is_none(),
        "a signature was moved onto another machine and still answered"
    );
    assert!(holds(&by, &about("dev_a", "000004.tisty"), &said).is_some());
}

#[test]
fn a_signature_over_something_else_is_turned_away() {
    let (_room, paths) = room();
    let key = mine(&paths, &DeviceId("dev_a".into())).unwrap();
    let by = key.verifying_key();
    let one = about("dev_a", "active.tisty");
    let said = signed(&key, &one, &tip_of(NOTHING_BEFORE, b"what it wrote\n"));

    let swapped = said.replace(
        &hexed(&tip_of(NOTHING_BEFORE, b"what it wrote\n")),
        &hexed(&tip_of(NOTHING_BEFORE, b"what it did not\n")),
    );

    assert!(
        holds(&by, &one, &swapped).is_none(),
        "a tip was swapped under a signature and it still answered"
    );
    assert!(holds(&by, &one, "").is_none());
    assert!(holds(&by, &one, "{}").is_none());
    assert!(holds(&by, &one, r#"{"tip":"ab","sig":"cd"}"#).is_none());
}

#[test]
fn a_key_truncated_on_disk_does_not_mute_the_machine_for_good() {
    let (_room, paths) = room();
    let who = DeviceId("dev_a".into());
    let first = mine(&paths, &who).expect("a key was made");
    let at = kept_at(&paths, &who).unwrap();

    std::fs::write(&at, b"").unwrap();

    let after = mine(&paths, &who).expect("an empty key file left the machine unable to sign");
    assert_ne!(after.to_bytes(), first.to_bytes());
    assert_eq!(mine(&paths, &who).unwrap().to_bytes(), after.to_bytes());
}
