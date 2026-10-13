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
    assert!(owing(&one.paths, &one.reach, None).is_some_and(|owed| !owed.delivered()));
    std::fs::create_dir_all(&gone).unwrap();
    assert!(owing(&one.paths, &one.reach, None).is_some_and(Owing::delivered));
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

#[test]
fn choosing_again_the_place_this_machine_was_stopped_in_takes_the_word_away() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let mut two = machine(room.path(), "two");
    crate::answers::storing::choose(&mut two, Some(shared.display().to_string())).unwrap();
    std::fs::write(shared.join(tisty_core::moved::NAMED), "otro equipo se fue").unwrap();

    crate::answers::storing::choose(&mut two, Some(shared.display().to_string())).unwrap();

    assert_eq!(
        word_in(&shared),
        None,
        "staying on purpose left the machine stopped"
    );
}

#[test]
fn a_word_owed_to_a_place_this_machine_came_back_to_is_never_delivered() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    let mut one = machine(room.path(), "one");
    let left = Sync::folder(&shared);
    leave_word(&one.paths, &one.config, &left, None, &Sync::alone());
    assert!(one.paths.cache().join(OWED).is_file());
    std::fs::create_dir_all(&shared).unwrap();

    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();

    assert!(!one.paths.cache().join(OWED).exists());
    assert!(owing(&one.paths, &one.reach, one.config.sync.as_ref()).is_none());
    assert_eq!(word_in(&shared), None);
}

#[test]
fn a_word_owed_is_dropped_when_the_way_now_chosen_is_that_place() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let one = machine(room.path(), "one");
    let left = Sync::folder(&shared);
    owe(one.paths.cache(), &left, "un aviso");

    assert!(owing(&one.paths, &one.reach, Some(&left)).is_none());
    assert!(!one.paths.cache().join(OWED).exists());
}

#[test]
fn leaving_this_machine_alone_owes_nobody_and_keeps_the_word_owed_to_a_real_place() {
    let room = tempfile::tempdir().unwrap();
    let unplugged = room.path().join("unplugged");
    let elsewhere = room.path().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let mut one = machine(room.path(), "one");
    one.keep(|c| c.sync = Some(Sync::alone())).unwrap();
    owe(
        one.paths.cache(),
        &Sync::folder(&unplugged),
        "para la carpeta desconectada",
    );

    crate::answers::storing::choose(&mut one, Some(elsewhere.display().to_string())).unwrap();

    let owed: Owed =
        serde_json::from_slice(&std::fs::read(one.paths.cache().join(OWED)).unwrap()).unwrap();
    assert_eq!(
        owed.way,
        Sync::folder(&unplugged),
        "a word to nobody pushed out a real one"
    );
}

#[test]
fn a_word_owed_to_no_place_is_dropped_instead_of_kept_forever() {
    let room = tempfile::tempdir().unwrap();
    let one = machine(room.path(), "one");
    owe(one.paths.cache(), &Sync::alone(), "a nadie");

    assert!(owing(&one.paths, &one.reach, None).is_none());
    assert!(!one.paths.cache().join(OWED).exists());
}

#[test]
fn a_change_that_is_refused_keeps_no_copy() {
    let room = tempfile::tempdir().unwrap();
    let mut one = machine(room.path(), "one");
    let later: Sync = toml::from_str("how = \"nube\"\nat = { cuenta = \"yo\" }").unwrap();
    one.keep(|c| c.sync = Some(later.clone())).unwrap();

    let refused = crate::answers::storing::choose(&mut one, Some("G:/otra".to_string()))
        .expect_err("left a way a newer build chose");

    assert_eq!(refused.code, "syncLaterToLeave");
    assert!(
        !one.paths.private().join("before-a-change").exists(),
        "a refused change pushed out a real copy"
    );
}

#[test]
fn machines_left_behind_are_told_the_cloud_the_others_moved_to() {
    let room = tempfile::tempdir().unwrap();
    let shared = room.path().join("shared");
    std::fs::create_dir_all(&shared).unwrap();
    let (mut one, two) = (machine(room.path(), "one"), machine(room.path(), "two"));
    vouches_for(&two, &one);
    crate::answers::storing::choose(&mut one, Some(shared.display().to_string())).unwrap();
    let cloud = Sync::cloud(CloudAt {
        provider: "google".into(),
        account: "0123".into(),
        email: "ana@example.com".into(),
    });

    leave_word(
        &one.paths,
        &one.config,
        &Sync::folder(&shared),
        Some(tisty_carrier::considering(&shared)),
        &cloud,
    );

    let stopped = heeded(
        word_in(&shared).as_deref(),
        two.paths.data(),
        &two.config.device_id,
    )
    .expect("the machine left behind went on syncing alone");
    assert_eq!(stopped.code, "movedToCloud");
    assert_eq!(stopped.name.as_deref(), Some("ana@example.com"));
}
