use super::*;
use crate::agents::{Event, Tools, Trigger, hook_doc};

fn trigger() -> Trigger {
    Trigger {
        event: Event::BeforeTool,
        tools: Tools::Shell,
        timeout: 5,
        agents: vec!["claude-code".into()],
    }
}

#[test]
fn a_hook_yes_holds_the_words_and_the_script_they_become() {
    let body = hook_doc("guard.sh", None, &trigger(), "exit 2\n").unwrap();
    let (yes, bytes) = yes_of(&body).unwrap();
    assert_eq!(bytes, b"exit 2\n");
    assert_eq!(yes.name, "guard.sh");
    assert_eq!(yes.event, "before-tool");
    assert_eq!(yes.print, crate::docs::print_of_text(&body));

    let tmp = tempfile::tempdir().unwrap();
    keep_script(tmp.path(), &yes, &bytes).unwrap();
    assert_eq!(script_kept(tmp.path(), &yes).as_deref(), Some(&bytes[..]));
    std::fs::write(script_at(tmp.path(), "guard.sh"), b"exit 0\n").unwrap();
    assert_eq!(script_kept(tmp.path(), &yes), None);
}

#[test]
fn a_document_that_is_not_a_hook_gives_no_yes() {
    let rule = "---\nagent: claude/rule/CLAUDE.md\n---\nBe kind.\n";
    assert!(yes_of(rule).is_err());
    assert!(yes_of("# Plain\n").is_err());
}

#[test]
fn editing_the_body_changes_the_print_and_the_script_it_stands_for() {
    let first = hook_doc("guard.sh", None, &trigger(), "exit 2\n").unwrap();
    let edited = first.replace("exit 2", "exit 0");
    let (yes_a, _) = yes_of(&first).unwrap();
    let (yes_b, bytes_b) = yes_of(&edited).unwrap();
    assert_ne!(yes_a.print, yes_b.print);
    assert_ne!(yes_a.script, yes_b.script);
    assert_eq!(bytes_b, b"exit 0\n");
}

#[test]
fn a_kept_script_is_read_only_while_its_bytes_still_match_and_dropping_removes_it() {
    let body = hook_doc("guard.sh", None, &trigger(), "exit 2\n").unwrap();
    let (yes, bytes) = yes_of(&body).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    assert_eq!(script_kept(tmp.path(), &yes), None);
    keep_script(tmp.path(), &yes, &bytes).unwrap();
    assert!(script_at(tmp.path(), "guard.sh").starts_with(hooks_folder(tmp.path())));
    assert!(script_kept(tmp.path(), &yes).is_some());
    drop_script(tmp.path(), "guard.sh");
    assert_eq!(script_kept(tmp.path(), &yes), None);
    drop_script(tmp.path(), "guard.sh");
}

#[test]
fn a_kept_script_over_the_size_limit_is_not_read() {
    let body = hook_doc("big.sh", None, &trigger(), "x\n").unwrap();
    let (yes, _) = yes_of(&body).unwrap();
    let tmp = tempfile::tempdir().unwrap();
    keep_script(tmp.path(), &yes, b"x\n").unwrap();
    let big = vec![b'a'; 1024 * 1024 + 1];
    std::fs::write(script_at(tmp.path(), "big.sh"), big).unwrap();
    assert_eq!(script_kept(tmp.path(), &yes), None);
}
