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

#[test]
fn a_file_forgotten_is_no_longer_answered_for_and_the_line_mark_reads_back() {
    let data = tempfile::tempdir().unwrap();
    let at = Path::new("/home/rina/.claude/rules/style.md");
    let gemini = Path::new("/home/rina/.gemini/GEMINI.md");
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    ledger.settle(at, b"Style.\n");
    ledger.applied_to(at, ["a-0001".to_string()]);
    ledger.lead(gemini);
    ledger.keep(data.path()).unwrap();

    let mut again = Ledger::open(data.path(), "machine-a");
    assert!(again.led(gemini));
    assert_eq!(
        again.written_files().map(|(at, _)| at).collect::<Vec<_>>(),
        [at]
    );
    again.forget(at);
    assert!(again.written(at).is_none());
    assert_eq!(again.applied_at(at).count(), 0);
    assert!(again.took("a-0001"));
    again.forget(gemini);
    assert!(!again.led(gemini));
}

#[test]
fn a_ledger_from_an_earlier_build_reads_without_the_line_mark() {
    let data = tempfile::tempdir().unwrap();
    let folder = data.path().join("agents");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join("ledger.json"),
        r#"{"machine":"machine-a","consented":true}"#,
    )
    .unwrap();
    let ledger = Ledger::open(data.path(), "machine-a");
    assert!(ledger.consented());
    assert!(!ledger.led(Path::new("/home/rina/.gemini/GEMINI.md")));
}

#[test]
fn a_document_marked_to_wait_stays_marked_until_the_person_applies_or_leaves_it() {
    let data = tempfile::tempdir().unwrap();
    let mut ledger = Ledger::fresh("machine-a");
    assert!(ledger.wait_all(["rule", "skill"]));
    assert!(!ledger.wait_all(["rule"]), "marking again changes nothing");
    ledger.keep(data.path()).unwrap();

    let mut again = Ledger::open(data.path(), "machine-a");
    assert!(again.waits("rule") && again.waits("skill"));
    again.take("rule", true);
    again.take("skill", false);
    assert!(!again.waits("rule") && !again.waits("skill"));
}

#[test]
fn only_this_computers_own_ledger_is_handed_back_to_be_written() {
    let data = tempfile::tempdir().unwrap();
    assert_eq!(
        Ledger::mine(data.path(), "machine-a"),
        Some(Ledger::fresh("machine-a"))
    );
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    ledger.keep(data.path()).unwrap();

    assert_eq!(Ledger::mine(data.path(), "machine-a"), Some(ledger));
    assert_eq!(Ledger::mine(data.path(), "machine-b"), None);
}
