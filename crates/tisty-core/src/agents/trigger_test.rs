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

#[test]
fn every_event_and_tool_word_reads_back_and_only_tool_events_take_a_tool() {
    for event in Event::ALL {
        assert_eq!(Event::read(event.word()), Some(event));
    }
    for tools in Tools::ALL {
        assert_eq!(Tools::read(tools.word()), Some(tools));
    }
    assert_eq!(Event::read("Stop"), None);
    assert_eq!(Tools::read("web"), None);
    let on_tools: Vec<Event> = Event::ALL.into_iter().filter(|e| e.on_tools()).collect();
    assert_eq!(on_tools, [Event::BeforeTool, Event::AfterTool]);
}

#[test]
fn a_tool_other_than_any_is_only_read_on_a_tool_event() {
    let stop = WHOLE.replace("before-tool", "stop");
    assert_eq!(read(&fields(&stop)), None);
    let any = stop.replace("hook-tools: shell", "hook-tools: any");
    assert_eq!(read(&fields(&any)).unwrap().event, Event::Stop);
}

#[test]
fn the_timeout_is_one_whole_number_between_one_and_the_most() {
    for ok in ["1", "600"] {
        let lines = WHOLE.replace("10", ok);
        assert!(read(&fields(&lines)).is_some(), "{ok}");
    }
    for bad in ["-1", "1.5", "ten", "", "99999999999"] {
        let lines = WHOLE.replace("hook-timeout: 10", &format!("hook-timeout: {bad}"));
        assert_eq!(read(&fields(&lines)), None, "{bad}");
    }
}

#[test]
fn the_agents_are_a_non_empty_list_of_plain_words_without_repeats() {
    for bad in ["[]", "[Codex]", "[a b]", "[co/dex]"] {
        let lines = WHOLE.replace("[claude-code, codex]", bad);
        assert_eq!(read(&fields(&lines)), None, "{bad}");
    }
    let long = format!("[{}]", "a".repeat(41));
    assert_eq!(
        read(&fields(&WHOLE.replace("[claude-code, codex]", &long))),
        None
    );
}

#[test]
fn field_names_are_read_in_any_case_and_a_trigger_is_none_without_any_hook_field() {
    let loud = WHOLE.replace("hook-event", "Hook-Event");
    assert!(read(&fields(&loud)).is_some());
    assert_eq!(read(&fields("title: x\n")), None);
}

#[test]
fn a_hook_name_stops_at_sixty_four_and_the_extension_is_the_last_dot() {
    let ok = format!("{}.sh", "a".repeat(61));
    assert_eq!(ok.len(), 64);
    assert!(hook_name(&ok));
    assert!(!hook_name(&format!("a{ok}")));
    assert!(hook_name("a.sh.py"));
    assert!(!hook_name("a.py.exe"));
    assert_eq!(extension("a.sh.py"), Some("py"));
    assert_eq!(extension(".sh"), None);
    assert_eq!(extension("noext"), None);
}
