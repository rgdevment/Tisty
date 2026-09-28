use super::*;

#[test]
fn an_empty_store_weighs_nothing_instead_of_failing() {
    let tmp = tempfile::tempdir().unwrap();

    assert_eq!(weighed(&tmp.path().join("absent")), 0);
    assert_eq!(devices(&tmp.path().join("absent")), 0);
    assert_eq!(attachments(tmp.path()).files, 0);
}

#[test]
fn what_is_under_the_root_is_weighed_all_the_way_down() {
    let tmp = tempfile::tempdir().unwrap();
    let deep = tmp.path().join("store").join("dev_a");
    std::fs::create_dir_all(&deep).unwrap();
    std::fs::write(deep.join("0001.jsonl"), b"0123456789").unwrap();
    std::fs::write(tmp.path().join("loose.txt"), b"12345").unwrap();

    assert_eq!(weighed(tmp.path()), 15);
    assert_eq!(devices(&tmp.path().join("store")), 1);
}

#[test]
fn attachments_are_counted_across_shelves() {
    let tmp = tempfile::tempdir().unwrap();
    for shelf in ["2026-08", "2026-07"] {
        let at = tmp.path().join("attachments").join(shelf);
        std::fs::create_dir_all(&at).unwrap();
        std::fs::write(at.join("one.pdf"), b"1234").unwrap();
    }

    let held = attachments(tmp.path());
    assert_eq!(held.files, 2);
    assert_eq!(held.bytes, 8);
}

fn wrote(who: &str, ago: i64) -> tisty_core::Event {
    let when = jiff::Timestamp::now() - jiff::SignedDuration::from_secs(ago);
    tisty_core::Event {
        version: 1,
        timestamp: when,
        device: tisty_core::DeviceId(who.into()),
        batch: None,
        undo: false,
        redo: false,
        seq: 0,
        optional: false,
        zone: None,
        via: None,
        op: tisty_core::event::Op::TaskAdd {
            id: ulid::Ulid::generate(),
            d: tisty_core::event::TaskAdd::new("algo", "a0"),
        },
    }
}

#[test]
fn every_machine_that_ever_wrote_is_named() {
    let told = [wrote("mac0", 0), wrote("win1", 60)];

    let all = machines(&told, "mac0", &Default::default(), &Default::default());

    assert_eq!(all.len(), 2);
    assert_eq!(all.iter().filter(|one| one.mine).count(), 1);
    assert!(
        all.iter().all(|one| one.when > 0),
        "without a date there is no way to see who is behind"
    );
}

#[test]
fn a_machine_that_was_removed_stops_being_listed() {
    let told = [wrote("mac0", 0), wrote("win1", 60)];

    let all = machines(
        &told,
        "mac0",
        &[tisty_core::DeviceId("win1".into())].into(),
        &Default::default(),
    );

    assert_eq!(all.len(), 1, "removing did not remove it from the list");
    assert_eq!(all[0].id, "mac0");
}

#[test]
fn the_one_that_wrote_last_is_shown_first() {
    let told = [wrote("old0", 60 * 60 * 24 * 12), wrote("new1", 0)];

    let all = machines(&told, "new1", &Default::default(), &Default::default());

    assert_eq!(all[0].id, "new1");
    assert!(
        all[0].when - all[1].when > 60 * 60 * 24 * 11,
        "a machine twelve days behind has to look twelve days behind"
    );
}

#[test]
fn a_machine_is_dated_by_its_last_write_and_not_its_first() {
    let told = [wrote("mac0", 60 * 60 * 24 * 30), wrote("mac0", 0)];

    let when = machines(&told, "mac0", &Default::default(), &Default::default())[0].when;
    let now = jiff::Timestamp::now().as_second();

    assert!(
        now - when < 60 * 60,
        "an old segment must not make a busy machine look abandoned"
    );
}

#[test]
fn a_machine_is_dated_by_what_it_wrote_not_by_when_the_copy_landed_here() {
    let told = [wrote("mac0", 0), wrote("win1", 60 * 60 * 24 * 12)];

    let all = machines(&told, "mac0", &Default::default(), &Default::default());
    let quiet = all.iter().find(|one| one.id == "win1").unwrap();
    let ago = jiff::Timestamp::now().as_second() - quiet.when;

    assert!(
        ago > 60 * 60 * 24 * 11,
        "una maquina callada doce dias parecia recien escrita: {ago}s"
    );
}

#[test]
fn the_operating_system_says_something_it_could_be_asked_about() {
    let said = os();
    assert!(!said.is_empty());
    assert!(said.chars().any(|c| c.is_alphabetic()), "{said}");
}

#[test]
fn an_assistant_is_not_a_machine_anyone_can_be_asked_to_open() {
    let agent = tisty_core::DeviceId("dev_agent".into());
    let told = [wrote("mac0", 0), wrote("dev_agent", 60)];
    let all = machines(&told, "mac0", &Default::default(), &[agent.clone()].into());
    assert_eq!(all.len(), 1, "only the machine is listed");
    assert_eq!(all[0].id, "mac0");
}
