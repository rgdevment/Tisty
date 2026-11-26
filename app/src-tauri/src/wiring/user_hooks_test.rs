use std::path::Path;

use super::*;
use crate::wiring::places::Os;

fn place(home: &Path) -> Place {
    Place {
        distro: None,
        os: Os::Linux,
        home: home.to_path_buf(),
        config: home.join(".config"),
        dsh: home.join(".dsh"),
        appdata: None,
        local: None,
        bins: Vec::new(),
        command: "/opt/tisty".to_string(),
        open: true,
        packaged: false,
        mount: String::new(),
    }
}

const GUARD: Hook = Hook {
    name: "guard.sh",
    event: Event::BeforeTool,
    tools: Tools::Shell,
    timeout: 5,
};

#[test]
fn a_users_hook_never_reads_as_tistys_own_and_the_other_way_round() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in [
        Kind::Claude,
        Kind::Codex,
        Kind::Cursor,
        Kind::Gemini,
        Kind::Copilot,
        Kind::Antigravity,
    ] {
        let own = hooks::with(kind, "", &here).unwrap();
        let both = with(kind, &own, &here, &GUARD).unwrap();
        assert!(
            hooks::hooked(kind, &both) && !hooks::apart(kind, &both),
            "{kind:?}"
        );
        let own_gone = hooks::without(kind, &both).unwrap();
        assert!(
            standing_in(kind, &own_gone, &here, GUARD.name).present,
            "{kind:?}"
        );
        let user_gone = without(kind, &both, GUARD.name).unwrap();
        assert!(
            !standing_in(kind, &user_gone, &here, GUARD.name).present,
            "{kind:?}"
        );
        assert!(hooks::hooked(kind, &user_gone), "{kind:?}");
    }
}

const KINDS: [Kind; 6] = [
    Kind::Claude,
    Kind::Codex,
    Kind::Cursor,
    Kind::Copilot,
    Kind::Antigravity,
    Kind::Gemini,
];

const NAMES: [&str; 9] = [
    "guard.sh",
    "a.sh",
    "a.sh.py",
    "claude-code.sh",
    "codex.sh",
    "cursor.sh",
    "copilot.sh",
    "antigravity.sh",
    "gemini-cli.sh",
];

const MOMENTS: [(Event, Tools); 4] = [
    (Event::Start, Tools::Any),
    (Event::BeforeTool, Tools::Shell),
    (Event::AfterTool, Tools::Write),
    (Event::Stop, Tools::Any),
];

fn hook(name: &str, at: usize) -> Hook<'_> {
    let (event, tools) = MOMENTS[at % MOMENTS.len()];
    Hook {
        name,
        event,
        tools,
        timeout: 5,
    }
}

fn all_registered(kind: Kind, here: &Place) -> String {
    let mut text = hooks::with(kind, "", here).unwrap();
    for (at, name) in NAMES.iter().enumerate() {
        text =
            with(kind, &text, here, &hook(name, at)).unwrap_or_else(|| panic!("{kind:?} {name}"));
    }
    text
}

fn parsed(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

#[test]
fn hooks_named_like_a_client_or_like_each_others_start_never_read_as_each_other_or_as_tistys() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in KINDS {
        let own_only = hooks::with(kind, "", &here).unwrap();
        let text = all_registered(kind, &here);
        assert!(
            hooks::hooked(kind, &text) && !hooks::apart(kind, &text),
            "{kind:?}"
        );
        for name in NAMES {
            let seen = standing_in(kind, &text, &here, name);
            assert!(seen.present && !seen.apart, "{kind:?} {name}");
        }
        let mut left = text.clone();
        for (at, name) in NAMES.iter().enumerate() {
            left = without(kind, &left, name).unwrap();
            assert!(
                !standing_in(kind, &left, &here, name).present,
                "{kind:?} {name}"
            );
            for later in &NAMES[at + 1..] {
                assert!(
                    standing_in(kind, &left, &here, later).present,
                    "{kind:?} {later}"
                );
            }
            assert!(hooks::hooked(kind, &left), "{kind:?} {name}");
        }
        assert_eq!(parsed(&left), parsed(&own_only), "{kind:?}");

        let own_gone = hooks::without(kind, &text).unwrap();
        assert!(!hooks::hooked(kind, &own_gone), "{kind:?}");
        for name in NAMES {
            assert!(
                standing_in(kind, &own_gone, &here, name).present,
                "{kind:?} {name}"
            );
        }
        let back = hooks::with(kind, &own_gone, &here).unwrap();
        assert!(hooks::hooked(kind, &back), "{kind:?}");
        for name in NAMES {
            assert!(
                standing_in(kind, &back, &here, name).present,
                "{kind:?} {name}"
            );
        }
    }
}

#[test]
fn a_hook_already_written_as_it_would_be_comes_back_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in KINDS {
        let once = with(kind, "", &here, &hook("a.sh", 1)).unwrap();
        assert_eq!(
            with(kind, &once, &here, &hook("a.sh", 1)).as_deref(),
            Some(once.as_str()),
            "{kind:?}"
        );
        assert!(
            !standing_in(kind, &once, &here, "a.sh.py").present,
            "{kind:?}"
        );
        assert!(!hooks::hooked(kind, &once), "{kind:?}");
    }
}

#[test]
fn a_hook_already_there_stays_byte_for_byte_beside_others_even_when_not_first() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in KINDS {
        let text = all_registered(kind, &here);
        for (at, name) in NAMES.iter().enumerate() {
            let same = with(kind, &text, &here, &hook(name, at)).unwrap();
            assert_eq!(same, text, "{kind:?} {name}");
        }
    }
}

#[test]
fn a_changed_timeout_event_or_matcher_rewrites_the_hook() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in KINDS {
        let once = with(kind, "", &here, &hook("a.sh", 1)).unwrap();
        let slower = Hook {
            timeout: 50,
            ..hook("a.sh", 1)
        };
        let later = with(kind, &once, &here, &slower).unwrap();
        assert_ne!(later, once, "{kind:?} timeout");
        let other_event = with(kind, &once, &here, &hook("a.sh", 3)).unwrap();
        assert_ne!(other_event, once, "{kind:?} event");
        let other_tools = Hook {
            tools: Tools::Write,
            ..hook("a.sh", 1)
        };
        let matched = with(kind, &once, &here, &other_tools).unwrap();
        assert_ne!(matched, once, "{kind:?} tools");
        for text in [&later, &other_event, &matched] {
            assert!(standing_in(kind, text, &here, "a.sh").present, "{kind:?}");
            let gone = without(kind, text, "a.sh").unwrap();
            assert!(!standing_in(kind, &gone, &here, "a.sh").present, "{kind:?}");
        }
    }
}

fn persons_own(kind: Kind) -> &'static str {
    match kind {
        Kind::Claude | Kind::Codex => {
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command","command":"echo mine"}]}]}}"#
        }
        Kind::Gemini => {
            r#"{"hooks":{"AfterAgent":[{"hooks":[{"type":"command","command":"echo mine"}]}]}}"#
        }
        Kind::Cursor => r#"{"version":1,"hooks":{"stop":[{"command":"echo mine"}]}}"#,
        Kind::Copilot => r#"{"version":1,"hooks":{"agentStop":[{"command":"echo mine"}]}}"#,
        Kind::Antigravity => r#"{"mine":{"enabled":true}}"#,
    }
}

#[test]
fn what_the_person_wrote_beside_tistys_hooks_and_hers_stays_through_both_ways() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in KINDS {
        let mine = persons_own(kind);
        let own = hooks::with(kind, mine, &here).unwrap();
        let both = with(kind, &own, &here, &hook("guard.sh", 1)).unwrap();
        let only_user = without(kind, &both, "guard.sh").unwrap();
        let only_own = hooks::without(kind, &only_user).unwrap();
        assert_eq!(parsed(&only_own), parsed(mine), "{kind:?}");
        let user_first = with(kind, mine, &here, &hook("guard.sh", 1)).unwrap();
        let back = without(kind, &user_first, "guard.sh").unwrap();
        assert_eq!(parsed(&back), parsed(mine), "{kind:?}");
    }
}

#[test]
fn an_antigravity_hook_turned_off_by_hand_is_apart_and_never_touched() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    let kind = Kind::Antigravity;
    let text = with(kind, "", &here, &hook("guard.sh", 1)).unwrap();
    let mut doc = parsed(&text);
    assert!(doc.get("tisty-guard.sh").is_some(), "{text}");
    doc["tisty-guard.sh"]["enabled"] = json!(false);
    let off = serde_json::to_string_pretty(&doc).unwrap();
    assert!(standing_in(kind, &off, &here, "guard.sh").apart);
    assert_eq!(with(kind, &off, &here, &hook("guard.sh", 1)), None);
    let other = with(kind, &off, &here, &hook("other.sh", 1)).unwrap();
    assert!(standing_in(kind, &other, &here, "other.sh").present);
    assert!(!standing_in(kind, &other, &here, "other.sh").apart);
}

#[test]
fn a_users_hook_run_from_another_install_is_present_but_not_here() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    let text = with(Kind::Claude, "", &here, &hook("guard.sh", 1)).unwrap();
    let rewired = text.replace("/opt/tisty", "/elsewhere/tisty");
    assert_ne!(rewired, text);
    let seen = standing_in(Kind::Claude, &rewired, &here, "guard.sh");
    assert!(seen.present);
    assert!(!seen.here);
}

#[test]
fn a_hook_is_offered_only_where_the_agent_has_that_moment_and_those_tools() {
    for event in [
        Event::Start,
        Event::BeforeTool,
        Event::AfterTool,
        Event::Stop,
    ] {
        for kind in KINDS {
            assert!(takes(kind, event, Tools::Any), "{kind:?} {event:?}");
        }
    }
    assert!(!takes(Kind::Antigravity, Event::Prompt, Tools::Any));
    for kind in [
        Kind::Claude,
        Kind::Codex,
        Kind::Cursor,
        Kind::Copilot,
        Kind::Gemini,
    ] {
        assert!(takes(kind, Event::Prompt, Tools::Any), "{kind:?}");
    }
    assert!(!takes(Kind::Copilot, Event::BeforeTool, Tools::Mcp));
    for kind in [
        Kind::Claude,
        Kind::Codex,
        Kind::Gemini,
        Kind::Antigravity,
        Kind::Cursor,
    ] {
        assert!(takes(kind, Event::BeforeTool, Tools::Mcp), "{kind:?}");
    }
    for kind in KINDS {
        for tools in [Tools::Shell, Tools::Write] {
            assert!(takes(kind, Event::BeforeTool, tools), "{kind:?} {tools:?}");
        }
    }
}

#[test]
fn a_moment_an_agent_lacks_changes_nothing_in_its_file() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    let prompt = Hook {
        name: "ask.sh",
        event: Event::Prompt,
        tools: Tools::Any,
        timeout: 5,
    };
    assert_eq!(with(Kind::Antigravity, "{}", &here, &prompt), None);
    let mcp = Hook {
        name: "m.sh",
        event: Event::BeforeTool,
        tools: Tools::Mcp,
        timeout: 5,
    };
    assert_eq!(with(Kind::Copilot, "{}", &here, &mcp), None);
}

#[test]
fn cursor_places_a_hook_by_its_own_events() {
    let at = |event, tools| {
        let found = placed(Kind::Cursor, event, tools).unwrap();
        (found.event, found.matcher)
    };
    assert_eq!(
        at(Event::BeforeTool, Tools::Shell),
        ("beforeShellExecution", None)
    );
    assert_eq!(
        at(Event::BeforeTool, Tools::Mcp),
        ("beforeMCPExecution", None)
    );
    assert_eq!(
        at(Event::BeforeTool, Tools::Write),
        ("preToolUse", Some("Write"))
    );
    assert_eq!(at(Event::AfterTool, Tools::Write), ("afterFileEdit", None));
    assert_eq!(at(Event::Stop, Tools::Any), ("stop", None));
}

#[test]
fn every_event_a_user_hook_lands_on_is_one_the_agent_lists_among_its_own() {
    for kind in KINDS {
        for event in Event::ALL {
            for tools in Tools::ALL {
                if let Some(found) = placed(kind, event, tools) {
                    assert!(
                        events(kind).contains(&found.event),
                        "{kind:?} {}",
                        found.event
                    );
                }
            }
        }
    }
}

fn first_run(doc: &Value, root: &str, event: &str) -> Value {
    let entry = doc[root][event][0].clone();
    match entry.get("hooks") {
        Some(inner) => inner[0].clone(),
        None => entry,
    }
}

#[test]
fn each_agent_gets_the_timeout_in_its_own_unit_with_two_seconds_of_slack() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    let guard = hook("guard.sh", 1);
    let text = |kind| parsed(&with(kind, "", &here, &guard).unwrap());
    let claude = text(Kind::Claude);
    let run = first_run(&claude, "hooks", "PreToolUse");
    assert_eq!(run["timeout"], 7);
    assert_eq!(
        run["args"],
        json!(["run-hook", "--name", "guard.sh", "--agent", "claude-code"])
    );
    assert_eq!(
        claude["hooks"]["PreToolUse"][0]["matcher"],
        "^(Bash|PowerShell)$"
    );
    let gemini = first_run(&text(Kind::Gemini), "hooks", "BeforeTool");
    assert_eq!(gemini["timeout"], 7000);
    assert_eq!(gemini["name"], "tisty-guard.sh");
    assert_eq!(
        first_run(&text(Kind::Codex), "hooks", "PreToolUse")["timeout"],
        7
    );
    let cursor = first_run(&text(Kind::Cursor), "hooks", "beforeShellExecution");
    assert_eq!(cursor["timeout"], 7);
    let said = cursor["command"].as_str().unwrap();
    assert!(
        said.ends_with("run-hook --name guard.sh --agent cursor"),
        "{said}"
    );
    let anti = text(Kind::Antigravity);
    assert_eq!(
        first_run(&anti, "tisty-guard.sh", "PreToolUse")["timeout"],
        7
    );
    assert_eq!(
        anti["tisty-guard.sh"]["PreToolUse"][0]["matcher"],
        "^run_command$"
    );
}

#[test]
fn copilot_before_a_tool_goes_through_a_shell_that_never_fails_and_after_one_does_not() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    let before = parsed(&with(Kind::Copilot, "", &here, &hook("guard.sh", 1)).unwrap());
    let run = first_run(&before, "hooks", "preToolUse");
    assert_eq!(run["timeoutSec"], 7);
    let said = run["bash"].as_str().unwrap();
    assert!(
        said.ends_with("run-hook --name guard.sh --agent copilot || true"),
        "{said}"
    );
    let after = parsed(&with(Kind::Copilot, "", &here, &hook("guard.sh", 2)).unwrap());
    let run = first_run(&after, "hooks", "postToolUse");
    assert_eq!(run["timeoutSec"], 7);
    assert_eq!(run["exec"], "/opt/tisty");
    assert!(run.get("bash").is_none());
}

#[test]
fn a_hook_is_never_added_to_a_list_that_is_not_a_list_or_to_a_file_that_is_not_json() {
    let tmp = tempfile::tempdir().unwrap();
    let here = place(tmp.path());
    for kind in KINDS {
        let at = placed(kind, Event::BeforeTool, Tools::Shell).unwrap().event;
        let root = match kind {
            Kind::Antigravity => "tisty-guard.sh",
            _ => "hooks",
        };
        let broken = json!({ root: { at: "no" } }).to_string();
        assert_eq!(
            with(kind, &broken, &here, &hook("guard.sh", 1)),
            None,
            "{kind:?}"
        );
        assert_eq!(
            with(kind, "not json", &here, &hook("guard.sh", 1)),
            None,
            "{kind:?}"
        );
    }
}

const SIX: [Kind; 6] = [
    Kind::Claude,
    Kind::Codex,
    Kind::Cursor,
    Kind::Gemini,
    Kind::Copilot,
    Kind::Antigravity,
];

fn moved(home: &Path, command: &str) -> Place {
    Place {
        command: command.to_string(),
        ..place(home)
    }
}

fn written(kind: Kind, here: &Place, text: &str) {
    let at = kind.file(here).unwrap();
    std::fs::create_dir_all(at.parent().unwrap()).unwrap();
    std::fs::write(at, text).unwrap();
}

#[test]
fn a_persons_hook_left_on_a_tisty_that_is_gone_is_told_like_tistys_own() {
    for kind in SIX {
        let tmp = tempfile::tempdir().unwrap();
        let old = moved(tmp.path(), "/opt/old/tisty");
        let here = moved(tmp.path(), "/opt/new/tisty");
        written(kind, &here, &with(kind, "", &old, &GUARD).unwrap());

        let seen = hooks::seen(kind, &here).unwrap();

        assert_eq!(seen.stray.len(), 1, "{kind:?} {seen:?}");
        assert_eq!(seen.stray[0].said, "/opt/old/tisty", "{kind:?}");
        assert!(seen.stray[0].gone, "{kind:?}");
    }
}

#[test]
fn a_persons_hook_on_this_tisty_is_never_told_astray() {
    for kind in SIX {
        let tmp = tempfile::tempdir().unwrap();
        let here = moved(tmp.path(), "/opt/new/tisty");
        written(kind, &here, &with(kind, "", &here, &GUARD).unwrap());

        assert!(
            hooks::seen(kind, &here).unwrap().stray.is_empty(),
            "{kind:?}"
        );
    }
}

#[test]
fn fixing_moves_only_the_program_of_every_persons_hook_and_keeps_the_rest() {
    let other = Hook {
        name: "audit.py",
        event: Event::Stop,
        tools: Tools::Any,
        timeout: 9,
    };
    for kind in SIX {
        let tmp = tempfile::tempdir().unwrap();
        let old = moved(tmp.path(), "/opt/old/tisty");
        let here = moved(tmp.path(), "/opt/new/tisty");
        let own = hooks::with(kind, "", &old).unwrap();
        let both = with(kind, &own, &old, &GUARD).unwrap();
        let was = with(kind, &both, &old, &other).unwrap();
        let named = ["/opt/old/tisty".to_string()];

        let own_fixed = hooks::repointed(kind, &was, &here, &named).unwrap();
        let now = repointed(kind, &own_fixed, &here, &named).unwrap();

        assert!(!now.contains("/opt/old/"), "{kind:?} {now}");
        assert_eq!(
            now,
            was.replace("/opt/old/tisty", "/opt/new/tisty"),
            "{kind:?}"
        );
        for hook in [&GUARD, &other] {
            let one = standing_in(kind, &now, &here, hook.name);
            assert!(
                one.present && one.here && !one.apart,
                "{kind:?} {}",
                hook.name
            );
        }
        assert!(
            hooks::hooked(kind, &now) && !hooks::apart(kind, &now),
            "{kind:?}"
        );
    }
}

#[test]
fn fixing_leaves_a_persons_hook_that_points_at_a_program_not_named() {
    for kind in SIX {
        let tmp = tempfile::tempdir().unwrap();
        let old = moved(tmp.path(), "/opt/old/tisty");
        let here = moved(tmp.path(), "/opt/new/tisty");
        let was = with(kind, "", &old, &GUARD).unwrap();

        let now = repointed(kind, &was, &here, &["/opt/other/tisty".to_string()]).unwrap();

        assert_eq!(now, was, "{kind:?}");
    }
}
