use tisty_core::config::CloudAt;

use super::*;
use crate::Session;

fn machine(room: &Path, named: &str) -> Session {
    let paths = tisty_core::Paths::new(
        room.join(named).join("data"),
        room.join(named).join("config"),
    );
    Session::at(paths).unwrap()
}

fn vouches_for(by: &Session, of: &Session) {
    let key = tisty_core::signing::mine(&of.paths, &of.config.device_id).unwrap();
    tisty_core::vouched::confirm(
        by.paths.data(),
        &of.config.device_id,
        &tisty_core::signing::shown(&key),
    );
}

fn word_in(folder: &Path) -> Option<String> {
    std::fs::read_to_string(folder.join(tisty_core::moved::NAMED)).ok()
}

#[test]
fn moving_away_keeps_tasks_and_documents_first_and_stops_the_machine_left_behind() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    let elsewhere = room.path().join("elsewhere");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::create_dir_all(&elsewhere).unwrap();
    let (mut one, two) = (machine(room.path(), "one"), machine(room.path(), "two"));
    vouches_for(&two, &one);
    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();

    crate::answers::storing::choose(&mut one, Some(elsewhere.display().to_string())).unwrap();

    let kept = std::fs::read_dir(one.paths.private().join("before-a-change"))
        .unwrap()
        .count();
    assert_eq!(
        kept, 2,
        "a change went ahead without keeping tasks and documents first"
    );
    let stopped = heeded(
        word_in(&shared).as_deref(),
        two.paths.data(),
        &two.config.device_id,
    )
    .expect("the machine left behind went on syncing alone");
    assert_eq!(stopped.code, "movedAway");
    assert_eq!(stopped.name.as_deref(), Some("elsewhere"));
    assert!(
        heeded(
            word_in(&shared).as_deref(),
            one.paths.data(),
            &one.config.device_id
        )
        .is_none(),
        "a machine stopped on its own word"
    );
}

#[test]
fn a_place_chosen_again_is_the_person_s_and_its_old_word_goes() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::write(shared.join(tisty_core::moved::NAMED), "un aviso viejo").unwrap();
    let mut one = machine(room.path(), "one");

    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();

    assert_eq!(word_in(&shared), None);
}

#[test]
fn a_word_the_old_place_could_not_take_is_owed_and_delivered_once_it_can() {
    let room = tempfile::tempdir().unwrap();
    let gone = room.path().join("unplugged");
    let one = machine(room.path(), "one");
    let left = Sync::folder(&gone);
    let old = tisty_carrier::chosen(Some(&left)).carrier;

    leave_word(&one.paths, &one.config, &left, old, &Sync::alone());

    assert!(one.paths.cache().join(OWED).is_file());
    assert!(owing(one.paths.cache(), &one.reach).is_some_and(|owed| !owed.delivered()));
    std::fs::create_dir_all(&gone).unwrap();
    assert!(owing(one.paths.cache(), &one.reach).is_some_and(Owing::delivered));
    assert!(word_in(&gone).is_some_and(|said| said.contains("alone")));
    assert!(!one.paths.cache().join(OWED).exists());
}

#[test]
fn the_same_account_under_a_new_address_is_the_same_place() {
    let at = |email: &str| {
        Sync::cloud(CloudAt {
            provider: "google".into(),
            account: "0123".into(),
            email: email.into(),
        })
    };

    assert!(same_place(
        &at("ana@example.com"),
        &at("ana.nueva@example.com")
    ));
    assert!(!same_place(
        &Sync::folder("G:/una"),
        &Sync::folder("G:/otra")
    ));
    assert!(!same_place(&Sync::alone(), &at("ana@example.com")));
}
