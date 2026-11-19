use super::*;

#[test]
fn a_ledger_starts_without_consent_and_with_every_assistant_on() {
    let ledger = Ledger::fresh("machine-a");
    assert!(!ledger.consented());
    assert!(ledger.syncs("claude-code"));
}

#[test]
fn what_this_computer_decided_is_read_back_here() {
    let data = tempfile::tempdir().unwrap();
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    ledger.turn("codex", false);
    ledger.settle(Path::new("/home/rina/.claude/CLAUDE.md"), b"# Rules\n");
    ledger.keep(data.path()).unwrap();

    let again = Ledger::open(data.path(), "machine-a");
    assert_eq!(again, ledger);
    assert!(!again.syncs("codex"));
    assert!(again.syncs("claude-code"));
    assert_eq!(
        again.written(Path::new("/home/rina/.claude/CLAUDE.md")),
        Some(crate::attach::printed(b"# Rules\n").as_str())
    );
}

#[test]
fn a_ledger_copied_to_another_computer_grants_nothing_there() {
    let data = tempfile::tempdir().unwrap();
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    ledger.settle(Path::new("/home/rina/.claude/CLAUDE.md"), b"# Rules\n");
    ledger.keep(data.path()).unwrap();

    let elsewhere = Ledger::open(data.path(), "machine-b");
    assert!(!elsewhere.consented());
    assert_eq!(
        elsewhere.written(Path::new("/home/rina/.claude/CLAUDE.md")),
        None
    );
}

#[test]
fn turning_the_whole_folder_off_stops_every_assistant() {
    let mut ledger = Ledger::fresh("machine-a");
    ledger.turn_all(false);
    assert!(!ledger.syncs("claude-code"));
    ledger.turn_all(true);
    assert!(ledger.syncs("claude-code"));
}

#[test]
fn the_ledger_lives_outside_everything_sync_and_backups_carry() {
    let data = tempfile::tempdir().unwrap();
    Ledger::fresh("machine-a").keep(data.path()).unwrap();
    let carried = crate::backup::CARRIED;
    let first = data.path().join("agents").join("ledger.json");
    assert!(first.exists());
    assert!(!carried.contains(&"agents"));
}

#[test]
fn a_computer_that_cannot_name_itself_never_counts_as_consented() {
    let mut ledger = Ledger::fresh("");
    ledger.consent(true);
    assert!(!ledger.consented());
}

#[test]
fn what_this_computer_chose_and_applied_is_kept_and_read_back() {
    let data = tempfile::tempdir().unwrap();
    let at = Path::new("/home/rina/.claude/CLAUDE.md");
    let mut ledger = Ledger::fresh("machine-a");
    ledger.take("a-0001", true);
    ledger.take("a-0002", false);
    assert!(ledger.taken_all(["a-0003"]));
    assert!(
        !ledger.taken_all(["a-0003"]),
        "taking it again changes nothing"
    );
    ledger.settle(at, b"# Rules\n");
    assert!(ledger.unproven(at));
    ledger.applied_to(at, ["a-0001".to_string()]);
    ledger.keep(data.path()).unwrap();

    let again = Ledger::open(data.path(), "machine-a");
    assert!(again.took("a-0001") && again.took("a-0003"));
    assert!(again.left("a-0002") && !again.took("a-0002"));
    assert!(!again.unproven(at));
    assert_eq!(again.applied_at(at).collect::<Vec<_>>(), ["a-0001"]);
    assert_eq!(again.applied().into_iter().collect::<Vec<_>>(), ["a-0001"]);

    let mut changed = again.clone();
    changed.take("a-0002", true);
    assert!(changed.took("a-0002") && !changed.left("a-0002"));
}
