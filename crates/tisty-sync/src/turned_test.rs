use super::*;

fn a_place() -> (tempfile::TempDir, std::path::PathBuf) {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    (room, data)
}

#[test]
fn nothing_is_named_until_a_round_turns_something_away() {
    let (_room, data) = a_place();

    assert!(of(&data).is_empty());
}

#[test]
fn what_a_round_turned_away_is_there_to_be_read() {
    let (_room, data) = a_place();
    let away = [
        ("dev_a".to_string(), Away::Disowned),
        ("dev_b".to_string(), Away::Unreadable),
    ]
    .into();

    keep(&data, &away);

    let read = of(&data);
    assert_eq!(read.get("dev_a"), Some(&Away::Disowned));
    assert_eq!(read.get("dev_b"), Some(&Away::Unreadable));
}

/// A machine that heals has to stop being named, or the window would keep showing a warning for
/// a history that comes home fine.
#[test]
fn a_round_that_turned_nothing_away_leaves_nobody_named() {
    let (_room, data) = a_place();
    keep(&data, &[("dev_a".to_string(), Away::Disowned)].into());

    keep(&data, &Default::default());

    assert!(of(&data).is_empty());
}

#[test]
fn a_name_no_machine_could_have_is_never_written() {
    let (_room, data) = a_place();

    keep(
        &data,
        &[("dev_a\tdev_b".to_string(), Away::Disowned)].into(),
    );

    assert!(of(&data).is_empty());
}
