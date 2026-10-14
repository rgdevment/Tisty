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
        r#"[{"owed":"back","name":"a.md","id":"A"}]"#,
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
        ..LastRound::default()
    };

    let kept = round_noted(home.path(), &round);

    assert_eq!(standing(home.path()).last, Some(kept));
}

#[test]
fn a_round_that_fails_the_same_way_again_carries_on_the_streak_and_another_way_starts_anew() {
    let home = tempfile::tempdir().unwrap();
    let round = |at: u64, code: &str, cause: Option<&str>| LastRound {
        at,
        code: code.into(),
        cause: cause.map(Into::into),
        ..LastRound::default()
    };

    let first = round_noted(
        home.path(),
        &round(100, "deferred-offline", Some("filtered")),
    );
    let second = round_noted(
        home.path(),
        &round(160, "deferred-offline", Some("filtered")),
    );
    let other = round_noted(
        home.path(),
        &round(220, "deferred-offline", Some("blocked")),
    );
    let healed = round_noted(home.path(), &round(280, "ok", None));
    let anew = round_noted(home.path(), &round(340, "deferred-limit", None));

    assert_eq!((first.rounds, first.since), (1, 100));
    assert_eq!((second.rounds, second.since), (2, 100));
    assert_eq!((other.rounds, other.since), (3, 100));
    assert_eq!(other.cause.as_deref(), Some("blocked"));
    assert_eq!((healed.rounds, healed.since), (1, 280));
    assert_eq!((anew.rounds, anew.since), (1, 340));
    assert_eq!(standing(home.path()).last.map(|one| one.rounds), Some(1));
}

#[test]
fn a_streak_counts_from_when_its_first_round_began_not_when_it_ended() {
    let home = tempfile::tempdir().unwrap();
    let round = |began: u64, at: u64| LastRound {
        at,
        since: began,
        code: "deferred-offline".into(),
        cause: Some("slow".into()),
        ..LastRound::default()
    };

    let long = round_noted(home.path(), &round(100, 260));
    let next = round_noted(home.path(), &round(300, 320));

    assert_eq!((long.rounds, long.since), (1, 100));
    assert_eq!((next.rounds, next.since), (2, 100));
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
        ..LastRound::default()
    };

    round_noted(home.path(), &round("wouldReset"));
    assert!(!cloud_joined(home.path(), &here));

    round_noted(home.path(), &round("ok"));
    round_noted(home.path(), &round("cloudLost"));
    round_forgotten(home.path());
    assert!(cloud_joined(home.path(), &here));
}
