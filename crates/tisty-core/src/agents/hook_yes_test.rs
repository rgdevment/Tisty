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
