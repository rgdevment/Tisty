use tisty_markdown::front;

use super::*;

fn fields(lines: &str) -> Vec<Field> {
    front::fields(&format!("---\n{lines}---\n"))
}

const WHOLE: &str = "hook-event: before-tool\nhook-tools: shell\nhook-timeout: 10\nhook-agents: [claude-code, codex]\n";

#[test]
fn a_whole_trigger_is_read() {
    let trigger = read(&fields(WHOLE)).unwrap();
    assert_eq!(trigger.event, Event::BeforeTool);
    assert_eq!(trigger.tools, Tools::Shell);
    assert_eq!(trigger.timeout, 10);
    assert_eq!(trigger.agents, ["claude-code", "codex"]);
    assert_eq!(read(&fields(&trigger.lines())), Some(trigger));
}

#[test]
fn a_trigger_missing_repeating_or_adding_a_field_is_none() {
    for lines in [
        WHOLE.replace("hook-timeout: 10\n", ""),
        format!("{WHOLE}hook-event: stop\n"),
        format!("{WHOLE}hook-when: later\n"),
        WHOLE.replace("10", "0"),
        WHOLE.replace("10", "601"),
        WHOLE.replace("before-tool", "start"),
        WHOLE.replace("[claude-code, codex]", "[codex, codex]"),
    ] {
        assert_eq!(read(&fields(&lines)), None, "{lines}");
    }
}

#[test]
fn a_hook_name_is_one_plain_leaf_with_an_interpreter_tisty_knows() {
    for name in ["guard.ps1", "no-rm.sh", "check_2.py", "ask.js"] {
        assert!(hook_name(name), "{name}");
    }
    for name in [
        "Guard.ps1",
        "guard",
        "guard.exe",
        "a b.sh",
        "dir/x.sh",
        ".sh",
        "-x.sh",
        "x;rm.sh",
    ] {
        assert!(!hook_name(name), "{name}");
    }
}
