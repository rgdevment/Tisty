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
fn what_was_kept_as_a_key_and_is_not_one_is_not_read_as_one() {
    let (_room, paths) = room();
    let who = DeviceId("dev_a".into());
    let at = kept_at(&paths, &who).unwrap();
    std::fs::create_dir_all(paths.private()).unwrap();
    std::fs::write(&at, b"three bytes short of a key").unwrap();

    assert!(
        mine(&paths, &who).is_none(),
        "it made a key over what somebody kept there"
    );
}
