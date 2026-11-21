use super::*;

#[test]
fn a_session_a_panic_left_poisoned_still_says_which_way_it_syncs() {
    let kept = tempfile::tempdir().unwrap();
    let paths = tisty_core::Paths::new(kept.path().join("data"), kept.path().join("config"));
    let mut session = Session::at(paths).unwrap();
    let chosen = tisty_core::config::Sync::Folder(kept.path().join("shared"));
    session.config.sync = Some(chosen.clone());
    let session = Mutex::new(session);

    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _held = session.lock();
        panic!("a command fell over while it held the session");
    }));

    assert!(session.is_poisoned());
    assert_eq!(chosen_in(&session), Some(chosen));
}
