use super::*;

fn listed(
    told: &[tisty_core::event::Event],
    mine: &str,
    gone: &std::collections::BTreeSet<tisty_core::DeviceId>,
    assistants: &std::collections::BTreeSet<tisty_core::DeviceId>,
) -> Vec<Machine> {
    let nowhere = tempfile::tempdir().unwrap();
    machines(
        told,
        mine,
        gone,
        assistants,
        &Default::default(),
        &tisty_core::Paths::new(nowhere.path().join("data"), nowhere.path().join("config")),
        None,
    )
}

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

    let all = listed(&told, "mac0", &Default::default(), &Default::default());

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

    let all = listed(
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

    let all = listed(&told, "new1", &Default::default(), &Default::default());

    assert_eq!(all[0].id, "new1");
    assert!(
        all[0].when - all[1].when > 60 * 60 * 24 * 11,
        "a machine twelve days behind has to look twelve days behind"
    );
}

#[test]
fn a_machine_is_dated_by_its_last_write_and_not_its_first() {
    let told = [wrote("mac0", 60 * 60 * 24 * 30), wrote("mac0", 0)];

    let when = listed(&told, "mac0", &Default::default(), &Default::default())[0].when;
    let now = jiff::Timestamp::now().as_second();

    assert!(
        now - when < 60 * 60,
        "an old segment must not make a busy machine look abandoned"
    );
}

#[test]
fn a_machine_is_dated_by_what_it_wrote_not_by_when_the_copy_landed_here() {
    let told = [wrote("mac0", 0), wrote("win1", 60 * 60 * 24 * 12)];

    let all = listed(&told, "mac0", &Default::default(), &Default::default());
    let quiet = all.iter().find(|one| one.id == "win1").unwrap();
    let ago = jiff::Timestamp::now().as_second() - quiet.when;

    assert!(
        ago > 60 * 60 * 24 * 11,
        "a machine twelve days quiet looked freshly written: {ago}s"
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
    let all = listed(&told, "mac0", &Default::default(), &[agent.clone()].into());
    assert_eq!(all.len(), 1, "only the machine is listed");
    assert_eq!(all[0].id, "mac0");
}

#[test]
fn what_the_folder_holds_and_this_machine_let_go_of_is_counted_once() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    let shared = tmp.path().join("shared");

    let here = data.join("attachments").join("ab");
    std::fs::create_dir_all(&here).unwrap();
    std::fs::write(here.join("foto-a1b2c3d4.png"), vec![0u8; 100]).unwrap();

    let theirs = shared.join("attachments");
    std::fs::create_dir_all(theirs.join("ab")).unwrap();
    std::fs::create_dir_all(theirs.join("cd")).unwrap();
    std::fs::write(theirs.join("ab").join("foto-a1b2c3d4.png"), vec![0u8; 100]).unwrap();
    std::fs::write(theirs.join("cd").join("video-9f8e7d6c.mp4"), vec![0u8; 900]).unwrap();

    assert_eq!(also_weighed(&data, None), 0, "nothing is shared");
    assert_eq!(
        also_weighed(&data, Some(&shared)),
        900,
        "the one that is in both places is already weighed at home"
    );
}

#[test]
fn a_machine_shows_both_the_key_it_publishes_and_the_one_somebody_answered_for() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let paths = tisty_core::Paths::new(data.clone(), room.path().join("config"));
    let who = tisty_core::DeviceId("win1".into());
    let said = tisty_core::signing::shown(&tisty_core::signing::mine(&paths, &who).unwrap());
    let told = [wrote("mac0", 0), wrote("win1", 60)];
    let keys = [(who.clone(), said.clone())].into();

    let before = machines(
        &told,
        "mac0",
        &Default::default(),
        &Default::default(),
        &keys,
        &paths,
        None,
    );
    let one = before.iter().find(|one| one.id == "win1").unwrap();
    assert_eq!(one.signs.as_deref(), Some(said.as_str()));
    assert_eq!(one.confirmed, None, "nobody answered for it yet");
    assert_eq!(one.confirmed_when, 0);

    assert!(tisty_core::vouched::confirm(&data, &who, &said));
    let after = machines(
        &told,
        "mac0",
        &Default::default(),
        &Default::default(),
        &keys,
        &paths,
        None,
    );
    let one = after.iter().find(|one| one.id == "win1").unwrap();
    assert_eq!(one.confirmed.as_deref(), Some(said.as_str()));
    assert!(
        one.confirmed_when > 0,
        "the day it was answered for is lost"
    );
}

#[test]
fn a_machine_that_published_nothing_shows_no_key_rather_than_an_empty_one() {
    let told = [wrote("mac0", 0)];

    let all = listed(&told, "mac0", &Default::default(), &Default::default());

    assert_eq!(all[0].signs, None);
    assert_eq!(all[0].confirmed, None);
}

/// The log keeps the first key a machine published and never another, so for this machine the
/// claim can be stale while the key on disk is the one it really signs with. Reading the claim
/// here is how a row says "confirmed" in green while every other machine turns it away.
#[test]
fn this_machine_shows_the_key_it_really_signs_with_not_the_one_the_log_froze() {
    let room = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(room.path().join("data"), room.path().join("config"));
    std::fs::create_dir_all(paths.data()).unwrap();
    let who = tisty_core::DeviceId("mac0".into());
    let ours = tisty_core::signing::shown(&tisty_core::signing::mine(&paths, &who).unwrap());
    let stale = tisty_core::signing::shown(
        &tisty_core::signing::mine(&paths, &tisty_core::DeviceId("win1".into())).unwrap(),
    );
    let told = [wrote("mac0", 0)];

    let all = machines(
        &told,
        "mac0",
        &Default::default(),
        &Default::default(),
        &[(who.clone(), stale.clone())].into(),
        &paths,
        None,
    );

    assert_eq!(
        all[0].signs.as_deref(),
        Some(ours.as_str()),
        "the row showed the key the log froze instead of the one on disk"
    );
    assert_ne!(all[0].signs.as_deref(), Some(stale.as_str()));
}

#[test]
fn a_machine_waiting_in_the_folder_is_listed_with_the_key_it_says_it_signs_with() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let paths = tisty_core::Paths::new(data.clone(), room.path().join("config"));
    let folder = tempfile::tempdir().unwrap();
    let who = tisty_core::DeviceId("dev_x".into());
    let said = tisty_core::signing::shown(&tisty_core::signing::mine(&paths, &who).unwrap());
    let mut store =
        tisty_core::Store::open(folder.path().join(tisty_sync::STORE), who.clone()).unwrap();
    store
        .append(tisty_core::Op::DeviceKey {
            d: who.clone(),
            p: said.clone(),
        })
        .unwrap();
    drop(store);
    tisty_sync::turned::keep(
        &data,
        &[("dev_x".to_string(), tisty_sync::turned::Away::Unconfirmed)].into(),
    );
    let told = [wrote("mac0", 0)];

    let all = machines(
        &told,
        "mac0",
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &paths,
        Some(folder.path()),
    );

    let one = all
        .iter()
        .find(|one| one.id == "dev_x")
        .expect("a machine waiting to be answered for is nowhere to be confirmed");
    assert_eq!(one.signs.as_deref(), Some(said.as_str()));
    assert_eq!(one.turned_away.as_deref(), Some("unconfirmed"));
    assert_eq!(one.when, 0, "it has never written here");
}

#[test]
fn a_machine_the_person_removed_is_not_offered_back_because_it_kept_writing() {
    let room = tempfile::tempdir().unwrap();
    let data = room.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let paths = tisty_core::Paths::new(data.clone(), room.path().join("config"));
    let gone: std::collections::BTreeSet<tisty_core::DeviceId> =
        [tisty_core::DeviceId("dev_x".into())].into();
    tisty_sync::turned::keep(
        &data,
        &[("dev_x".to_string(), tisty_sync::turned::Away::Unconfirmed)].into(),
    );
    let told = [wrote("mac0", 0)];

    let all = machines(
        &told,
        "mac0",
        &gone,
        &Default::default(),
        &Default::default(),
        &paths,
        None,
    );

    assert!(
        all.iter().all(|one| one.id != "dev_x"),
        "a machine the person dropped was offered back as one waiting to be let in"
    );
}
