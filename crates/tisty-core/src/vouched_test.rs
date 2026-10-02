use super::*;

fn a_place() -> (tempfile::TempDir, std::path::PathBuf, crate::Paths) {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let paths = crate::Paths::new(data.clone(), room.path().join("config"));
    (room, data, paths)
}

fn a_key(paths: &crate::Paths, who: &DeviceId) -> String {
    crate::signing::shown(&crate::signing::mine(paths, who).unwrap())
}

fn key_of(data: &std::path::Path, who: &DeviceId) -> Option<String> {
    confirmed(data, who).map(|one| one.key)
}

#[test]
fn nothing_is_confirmed_until_somebody_says_so() {
    let (_room, data, paths) = a_place();
    let who = DeviceId("dev_a".into());
    let said = a_key(&paths, &who);

    assert_eq!(confirmed(&data, &who), None);
    assert!(confirm(&data, &who, &said));
    assert_eq!(key_of(&data, &who).as_deref(), Some(said.as_str()));
}

/// The door this exists to close: once a person has answered for a machine's key, a later write
/// cannot quietly put another one in its place.
#[test]
fn a_key_confirmed_is_not_replaced_by_a_later_one() {
    let (_room, data, paths) = a_place();
    let who = DeviceId("dev_a".into());
    let first = a_key(&paths, &who);
    let other = a_key(&paths, &DeviceId("dev_b".into()));
    assert!(confirm(&data, &who, &first));

    assert!(
        !confirm(&data, &who, &other),
        "a second key took the place of the one somebody answered for"
    );
    assert_eq!(key_of(&data, &who).as_deref(), Some(first.as_str()));
}

#[test]
fn confirming_the_same_key_twice_is_no_change_and_no_complaint() {
    let (_room, data, paths) = a_place();
    let who = DeviceId("dev_a".into());
    let said = a_key(&paths, &who);

    assert!(confirm(&data, &who, &said));
    assert!(confirm(&data, &who, &said));
    assert_eq!(all_confirmed(&data).len(), 1);
}

#[test]
fn what_is_not_a_key_is_never_confirmed() {
    let (_room, data, _paths) = a_place();
    let who = DeviceId("dev_a".into());

    assert!(!confirm(&data, &who, "not a key at all"));
    assert_eq!(confirmed(&data, &who), None);
}

#[test]
fn every_machine_keeps_its_own() {
    let (_room, data, paths) = a_place();
    let one = DeviceId("dev_a".into());
    let two = DeviceId("dev_b".into());
    assert!(confirm(&data, &one, &a_key(&paths, &one)));
    assert!(confirm(&data, &two, &a_key(&paths, &two)));

    let all = all_confirmed(&data);
    assert_eq!(all.len(), 2);
    assert_ne!(all[&one].key, all[&two].key);
}

#[test]
fn when_somebody_answered_for_a_key_is_kept_with_it() {
    let (_room, data, paths) = a_place();
    let who = DeviceId("dev_a".into());
    let before = crate::lately::now();

    assert!(confirm(&data, &who, &a_key(&paths, &who)));

    let when = confirmed(&data, &who).unwrap().when;
    assert!(when >= before, "{when} is before the confirming happened");
}

/// A name with a tab or a newline in it would write lines of its own, and the names of the other
/// machines arrive from a folder anyone may write to.
#[test]
fn a_name_that_could_write_a_line_of_its_own_is_turned_away() {
    let (_room, data, paths) = a_place();
    let said = a_key(&paths, &DeviceId("dev_a".into()));

    for named in ["dev_a\tdev_b", "dev_a\n01M3", "", "../dev_a"] {
        let who = DeviceId(named.to_string());
        assert!(!confirm(&data, &who, &said), "{named} was confirmed");
    }
    assert_eq!(all_confirmed(&data).len(), 0);
}

#[test]
fn a_line_that_does_not_read_whole_is_no_confirmation() {
    let (_room, data, paths) = a_place();
    let who = DeviceId("dev_a".into());
    let said = a_key(&paths, &who);
    let kept = data.join(KEPT);

    for line in [
        format!("dev_a\t{said}"),
        format!("dev_a\t{said}\tnot a moment"),
        "dev_a\tnot a key\t12".to_string(),
        format!("\t{said}\t12"),
    ] {
        std::fs::write(&kept, line.as_bytes()).unwrap();
        assert_eq!(confirmed(&data, &who), None, "{line} stood as a key");
    }
}

#[test]
fn answering_for_a_whole_list_leaves_out_what_already_stood() {
    let (_room, data, paths) = a_place();
    let one = DeviceId("dev_a".into());
    let two = DeviceId("dev_b".into());
    let keys: std::collections::BTreeMap<DeviceId, String> = [
        (one.clone(), a_key(&paths, &one)),
        (two.clone(), a_key(&paths, &two)),
    ]
    .into();
    assert!(confirm(&data, &one, &keys[&one]));

    assert_eq!(confirm_each(&data, &keys), 2, "it did not answer for both");
    assert_eq!(all_confirmed(&data).len(), 2);
    assert_eq!(
        confirm_each(&data, &keys),
        2,
        "a second pass changed something"
    );
}

#[test]
fn answering_for_a_list_turns_away_what_is_not_a_key() {
    let (_room, data, _paths) = a_place();
    let keys: std::collections::BTreeMap<DeviceId, String> =
        [(DeviceId("dev_a".into()), "not a key".to_string())].into();

    assert_eq!(confirm_each(&data, &keys), 0);
    assert!(all_confirmed(&data).is_empty());
}
