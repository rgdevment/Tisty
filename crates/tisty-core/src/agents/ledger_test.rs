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

#[test]
fn tistys_own_skills_have_their_switch_and_never_count_as_a_document_taken() {
    let data = tempfile::tempdir().unwrap();
    let mut ledger = Ledger::fresh("machine-a");
    assert!(!ledger.tisty_skills());
    ledger.take_tisty_skills(true);
    let at = Path::new("/home/rina/.claude/skills/tisty/SKILL.md");
    ledger.applied_to(at, ["tisty:skill/tisty/SKILL.md".to_string()]);
    assert!(!ledger.taken_all(["tisty:skill/consolidate/SKILL.md"]));
    assert!(ledger.taken_all(["dev_here-0001"]));
    assert!(!ledger.took("tisty:skill/tisty/SKILL.md"));
    assert!(ledger.took("dev_here-0001"));
    assert_eq!(
        ledger.applied_at(at).collect::<Vec<_>>(),
        ["tisty:skill/tisty/SKILL.md"]
    );
    ledger.keep(data.path()).unwrap();
    assert!(Ledger::open(data.path(), "machine-a").tisty_skills());
    assert!(!Ledger::open(data.path(), "machine-b").tisty_skills());
}

#[test]
fn a_ledger_from_before_tistys_own_skills_reads_with_them_off() {
    let ledger: Ledger =
        serde_json::from_str(r#"{"machine":"machine-a","consented":true}"#).unwrap();
    assert!(ledger.consented());
    assert!(!ledger.tisty_skills());
}

#[test]
fn a_file_the_person_let_stay_is_kept_here_alone_until_tisty_writes_it_or_forgets_it() {
    let data = tempfile::tempdir().unwrap();
    let at = Path::new("/home/rina/.claude/skills/voice/SKILL.md");
    let mut ledger = Ledger::fresh("machine-a");
    ledger.consent(true);
    ledger.settle(at, b"Speak plainly.\n");
    assert!(!ledger.stays(at));
    ledger.stay(at);
    ledger.keep(data.path()).unwrap();
    assert!(Ledger::open(data.path(), "machine-a").stays(at));
    assert!(
        !Ledger::open(data.path(), "machine-b").stays(at),
        "the mark is this computer's, never another's"
    );

    ledger.settle(at, b"Speak plainly again.\n");
    assert!(!ledger.stays(at), "written again, it is Tisty's once more");
    ledger.stay(at);
    ledger.forget(at);
    assert!(!ledger.stays(at));
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn a_file_left_under_one_spelling_is_left_under_every_spelling_the_file_system_folds() {
    let mut ledger = Ledger::fresh("machine-a");
    ledger.stay(Path::new("/home/rina/.claude/skills/Voice/SKILL.md"));
    assert!(ledger.stays(Path::new("/home/rina/.claude/skills/voice/SKILL.md")));
    ledger.settle(Path::new("/home/rina/.claude/skills/voice/SKILL.md"), b"x");
    assert!(!ledger.stays(Path::new("/home/rina/.claude/skills/Voice/SKILL.md")));
}

#[test]
fn two_spellings_are_one_file_only_where_the_file_system_folds_them() {
    let (upper, lower) = (
        Path::new("/home/rina/.claude/skills/Voice/SKILL.md"),
        Path::new("/home/rina/.claude/skills/voice/SKILL.md"),
    );
    assert!(crate::agents::same_file(lower, lower));
    assert_eq!(
        crate::agents::same_file(upper, lower),
        cfg!(any(windows, target_os = "macos"))
    );
}

#[test]
fn a_choice_is_kept_per_place_and_replaced_by_the_next() {
    let mut ledger = Ledger::fresh("machine-a");
    assert_eq!(ledger.chosen("/h/CLAUDE.md"), None);
    ledger.choose("/h/CLAUDE.md", "doc-a");
    assert_eq!(ledger.chosen("/h/CLAUDE.md"), Some("doc-a"));
    ledger.choose("/h/CLAUDE.md", "doc-b");
    assert_eq!(ledger.chosen("/h/CLAUDE.md"), Some("doc-b"));
    assert_eq!(ledger.chosen("/h/other.md"), None);
}

#[test]
fn a_choice_touches_neither_what_was_taken_nor_what_was_left() {
    let mut ledger = Ledger::fresh("machine-a");
    ledger.take("doc-a", true);
    ledger.take("doc-b", false);
    ledger.choose("/h/CLAUDE.md", "doc-a");
    assert!(ledger.took("doc-a") && ledger.left("doc-b"));
    assert!(!ledger.waits("doc-a"));
}

#[test]
fn words_let_in_here_are_kept_by_document_and_say_whether_they_changed() {
    let mut ledger = Ledger::fresh("machine-a");
    assert_eq!(ledger.let_in_words("doc-a"), None);
    assert!(ledger.let_in("doc-a", "p1"), "the first time is news");
    assert!(
        !ledger.let_in("doc-a", "p1"),
        "the same words again are not"
    );
    assert!(ledger.let_in("doc-a", "p2"));
    assert_eq!(ledger.let_in_words("doc-a"), Some("p2"));
    assert_eq!(ledger.let_in_words("doc-b"), None);
}

#[test]
fn a_file_of_several_documents_is_mixed_and_one_of_one_is_not() {
    let mut ledger = Ledger::fresh("machine-a");
    let at = Path::new("/home/rina/.claude/CLAUDE.md");
    assert!(!ledger.mixed(at));
    ledger.applied_to(at, ["doc-a".to_string()]);
    assert!(!ledger.mixed(at));
    ledger.applied_to(at, ["doc-a".to_string(), "doc-b".to_string()]);
    assert!(ledger.mixed(at));
}

#[test]
fn homing_is_kept_and_read_back_and_is_done_when_the_set_is_empty() {
    let data = tempfile::tempdir().unwrap();
    let mut ledger = Ledger::fresh("machine-a");
    assert!(!ledger.homed());
    assert_eq!(ledger.homing(), None);
    ledger.home_all(["doc-a".to_string(), "doc-b".to_string()].into());
    assert!(!ledger.homed());
    ledger.choose("/h/CLAUDE.md", "doc-a");
    ledger.let_in("doc-a", "p1");
    ledger.keep(data.path()).unwrap();

    let mut again = Ledger::open(data.path(), "machine-a");
    assert_eq!(again, ledger);
    again.went_home("doc-a");
    assert!(!again.homed());
    assert_eq!(again.homing().map(BTreeSet::len), Some(1));
    again.went_home("doc-b");
    again.went_home("doc-b");
    assert!(again.homed());
}

#[test]
fn an_empty_set_to_home_is_homed_at_once() {
    let mut ledger = Ledger::fresh("machine-a");
    ledger.home_all(BTreeSet::new());
    assert!(ledger.homed());
}

#[test]
fn nothing_goes_home_before_the_set_is_known() {
    let mut ledger = Ledger::fresh("machine-a");
    ledger.went_home("doc-a");
    assert_eq!(ledger.homing(), None);
    assert!(!ledger.homed());
}

#[test]
fn a_ledger_from_before_computers_reads_without_the_new_fields() {
    let data = tempfile::tempdir().unwrap();
    let folder = data.path().join("agents");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(
        folder.join("ledger.json"),
        r#"{"machine":"machine-a","consented":true,"applied":{"/h/CLAUDE.md":["doc-a","doc-b"]},"left":["doc-c"]}"#,
    )
    .unwrap();
    let ledger = Ledger::open(data.path(), "machine-a");
    assert!(ledger.consented());
    assert!(!ledger.homed());
    assert!(ledger.left("doc-c"));
    assert!(ledger.mixed(Path::new("/h/CLAUDE.md")));
    assert_eq!(ledger.chosen("/h/CLAUDE.md"), None);
    assert_eq!(ledger.let_in_words("doc-a"), None);
}
