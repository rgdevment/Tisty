use super::*;

#[test]
fn a_wait_the_provider_asked_for_is_kept_capped_and_let_go_of() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(left(home.path()), None);

    rest(home.path(), Duration::from_secs(24 * 60 * 60));
    let asked = left(home.path()).unwrap();
    assert!(
        asked <= RESTED_AT_MOST && asked > RESTED_AT_MOST - Duration::from_secs(60),
        "a day asked for was kept whole: {asked:?}"
    );

    rested(home.path());
    assert_eq!(left(home.path()), None);
}

#[test]
fn a_wait_that_already_passed_or_reads_wrong_holds_nothing_back() {
    let home = tempfile::tempdir().unwrap();
    std::fs::write(home.path().join(FILE), br#"{"until_ms":5}"#).unwrap();
    assert_eq!(left(home.path()), None);

    std::fs::write(home.path().join(FILE), b"no es json").unwrap();
    assert_eq!(left(home.path()), None);

    std::fs::write(
        home.path().join(FILE),
        format!(r#"{{"until_ms":{}}}"#, u64::MAX),
    )
    .unwrap();
    assert_eq!(
        left(home.path()),
        None,
        "a clock set back silenced the cloud for good"
    );
}
