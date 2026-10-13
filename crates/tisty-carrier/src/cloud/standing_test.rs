use super::*;

#[test]
fn a_cloud_never_reached_reads_as_nothing_spent_and_the_shipped_limits() {
    let home = tempfile::tempdir().unwrap();

    let told = standing(home.path());

    assert!(!told.listed && !told.confirmed && !told.paused_here);
    assert_eq!((told.owed, told.forgotten, told.owed_back), (0, 0, 0));
    assert_eq!((told.units_spent, told.bytes_spent), (0, 0));
    assert_eq!(
        (told.units_a_day, told.bytes_a_day),
        (UNITS_A_DAY, BYTES_A_DAY)
    );
    assert_eq!((told.halted_until, told.serial), (None, None));
}

#[test]
fn what_waits_here_is_counted_and_the_switch_here_is_seen() {
    let home = tempfile::tempdir().unwrap();
    crate::forgotten_later(home.path(), "dev_a-0001");
    crate::forgotten_later(home.path(), "dev_a-0002");
    std::fs::write(
        home.path().join(OWED_BACK),
        r#"[{"owed":"unbinned","id":"A"}]"#,
    )
    .unwrap();
    policy::pause_here(home.path(), true).unwrap();

    let told = standing(home.path());

    assert_eq!(told.forgotten, 2);
    assert_eq!(told.owed_back, 1);
    assert!(told.paused_here);
}

#[test]
fn the_signed_word_kept_here_sets_the_limits_and_a_live_pause() {
    let home = tempfile::tempdir().unwrap();
    let until = policy::now() + 3600;
    std::fs::write(
        home.path().join("cloud-policy.json"),
        format!(r#"{{"serial":7,"units_a_day":1000,"halted_until":{until}}}"#),
    )
    .unwrap();

    let told = standing(home.path());

    assert_eq!(told.serial, Some(7));
    assert_eq!(told.units_a_day, 1000);
    assert_eq!(told.bytes_a_day, BYTES_A_DAY);
    assert_eq!(told.halted_until, Some(until));
}

#[test]
fn how_the_last_round_ended_is_kept_and_read_back() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(standing(home.path()).last, None);
    let round = LastRound {
        at: 1_760_000_000,
        code: "deferred-budget".into(),
        name: None,
        left: 3,
        names: vec!["attachments/ab/foto.png".into()],
    };

    round_noted(home.path(), &round);

    assert_eq!(standing(home.path()).last, Some(round));
}

#[test]
fn a_cloud_is_joined_once_a_round_here_finished_not_when_it_only_pulled() {
    let home = tempfile::tempdir().unwrap();
    let room = tempfile::tempdir().unwrap();
    let here = crate::Here {
        data: room.path().join("data"),
        aside: room.path().join("aside"),
        device: "dev_a".into(),
    };
    let mut pulled = Index::default();
    pulled.shelf.insert(
        "attachments/ab/plano.pdf".into(),
        crate::Seen {
            name: "attachments/ab/plano.pdf".into(),
            bytes: 1,
            hash: "h".into(),
            revision: "r".into(),
        },
    );
    pulled.save(home.path()).unwrap();
    let round = |code: &str| LastRound {
        at: 1,
        code: code.into(),
        name: None,
        left: 0,
        names: Vec::new(),
    };

    round_noted(home.path(), &round("wouldReset"));
    assert!(!cloud_joined(home.path(), &here));

    round_noted(home.path(), &round("ok"));
    round_noted(home.path(), &round("cloudLost"));
    round_forgotten(home.path());
    assert!(cloud_joined(home.path(), &here));
}
