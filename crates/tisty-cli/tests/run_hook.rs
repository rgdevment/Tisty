use std::io::Write;
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;
use tisty_core::agents::{self, Event, Ledger, Tools, Trigger, hook_doc};

mod person;

const AGENTS: [&str; 6] = [
    "claude-code",
    "codex",
    "cursor",
    "copilot",
    "antigravity",
    "gemini-cli",
];

fn name() -> &'static str {
    match cfg!(windows) {
        true => "guard.ps1",
        false => "guard.sh",
    }
}

fn denying() -> &'static str {
    match cfg!(windows) {
        true => "[Console]::Error.WriteLine('no rm here'); exit 2\n",
        false => "echo 'no rm here' >&2\nexit 2\n",
    }
}

fn passing() -> &'static str {
    "exit 0\n"
}

fn sleeping() -> &'static str {
    match cfg!(windows) {
        true => "Start-Sleep -Seconds 10; exit 2\n",
        false => "sleep 10\nexit 2\n",
    }
}

struct Home {
    home: TempDir,
}

impl Home {
    fn new() -> Self {
        let home = Self {
            home: tempfile::tempdir().unwrap(),
        };
        home.person().begins("algo mio");
        home.person().lets_the_agent_in();
        home
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn data(&self) -> std::path::PathBuf {
        self.home.path().join("data")
    }

    fn registers(&self, script: &str, event: Event, agents: &[&str], timeout: u32) {
        let trigger = Trigger {
            event,
            tools: match event.on_tools() {
                true => Tools::Shell,
                false => Tools::Any,
            },
            timeout,
            agents: agents.iter().map(|one| (*one).to_string()).collect(),
        };
        let body = hook_doc(name(), None, &trigger, script).unwrap();
        let (yes, bytes) = agents::yes_of(&body).unwrap();
        agents::keep_script(&self.data(), &yes, &bytes).unwrap();
        let here = tisty_core::machine::here().unwrap();
        let mut ledger = Ledger::open(&self.data(), &here);
        ledger.register_hook("file-guard", yes);
        ledger.keep(&self.data()).unwrap();
    }

    fn changes_the_script(&self, script: &str) {
        std::fs::write(agents::script_at(&self.data(), name()), script).unwrap();
    }

    fn run(&self, args: &[&str], heard: &str) -> Output {
        let root = self.home.path();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
            .args(args)
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(heard.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn hook(&self, agent: &str) -> Output {
        self.hook_named(name(), agent, "{}")
    }

    fn hook_named(&self, hook: &str, agent: &str, heard: &str) -> Output {
        self.run(&["run-hook", "--name", hook, "--agent", agent], heard)
    }
}

fn silent(out: &Output) -> String {
    assert!(out.status.success(), "{out:?}");
    assert!(
        out.stdout.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn denied(out: &Output) -> Value {
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn a_registered_script_that_exits_two_before_a_tool_is_denied_in_each_agents_own_words() {
    let home = Home::new();
    home.registers(denying(), Event::BeforeTool, &AGENTS, 30);
    let said = |agent| denied(&home.hook(agent));
    for agent in ["claude-code", "codex"] {
        let out = said(agent);
        assert_eq!(out["hookSpecificOutput"]["permissionDecision"], "deny");
        assert_eq!(
            out["hookSpecificOutput"]["permissionDecisionReason"],
            "no rm here"
        );
    }
    assert_eq!(
        said("cursor"),
        json!({ "permission": "deny", "agent_message": "no rm here" })
    );
    for agent in ["antigravity", "gemini-cli"] {
        assert_eq!(
            said(agent),
            json!({ "decision": "deny", "reason": "no rm here" })
        );
    }
    assert_eq!(
        said("copilot"),
        json!({ "permissionDecision": "deny", "permissionDecisionReason": "no rm here" })
    );
}

#[test]
fn a_script_that_exits_zero_says_nothing() {
    let home = Home::new();
    home.registers(passing(), Event::BeforeTool, &AGENTS, 30);
    let agents: &[&str] = match cfg!(windows) {
        true => &["claude-code", "copilot"],
        false => &AGENTS,
    };
    for agent in agents {
        assert_eq!(silent(&home.hook(agent)), "", "{agent}");
    }
}

#[test]
fn a_script_changed_after_it_was_registered_does_not_run_and_says_so() {
    let home = Home::new();
    home.registers(denying(), Event::BeforeTool, &AGENTS, 30);
    home.changes_the_script(passing());
    let said = silent(&home.hook("codex"));
    assert!(said.contains("changed since you registered it"), "{said}");
    home.changes_the_script("[Console]::Error.WriteLine('different'); exit 2\n");
    let said = silent(&home.hook("codex"));
    assert!(said.contains("changed since you registered it"), "{said}");
}

#[test]
fn a_script_that_vanished_does_not_run_and_says_so() {
    let home = Home::new();
    home.registers(denying(), Event::BeforeTool, &AGENTS, 30);
    std::fs::remove_file(agents::script_at(&home.data(), name())).unwrap();
    let said = silent(&home.hook("codex"));
    assert!(said.contains("changed since you registered it"), "{said}");
}

#[test]
fn with_the_door_closed_nothing_runs() {
    let home = Home::new();
    home.registers(denying(), Event::BeforeTool, &AGENTS, 30);
    assert!(!home.hook("codex").stdout.is_empty());
    home.person().shuts_the_agent_out();
    assert_eq!(silent(&home.hook("codex")), "");
}

#[test]
fn a_hook_that_is_not_registered_says_so_and_runs_nothing() {
    let home = Home::new();
    let said = silent(&home.hook("codex"));
    assert!(
        said.contains("is not registered on this computer"),
        "{said}"
    );
    home.registers(denying(), Event::BeforeTool, &AGENTS, 30);
    let other = silent(&home.hook_named("other.ps1", "codex", "{}"));
    assert!(other.contains("other.ps1 is not registered"), "{other}");
}

#[test]
fn only_the_agents_a_hook_was_registered_with_hear_it() {
    let home = Home::new();
    home.registers(denying(), Event::BeforeTool, &["claude-code"], 30);
    assert_eq!(silent(&home.hook("codex")), "");
    assert!(!home.hook("claude-code").stdout.is_empty());
}

#[test]
fn a_claude_code_entry_inside_cursor_answers_as_cursor_unless_cursor_has_its_own() {
    let home = Home::new();
    let in_cursor = r#"{"cursor_version":"1.0"}"#;
    home.registers(denying(), Event::BeforeTool, &["claude-code"], 30);
    let out = denied(&home.hook_named(name(), "claude-code", in_cursor));
    assert_eq!(out["permission"], "deny");
    home.registers(denying(), Event::BeforeTool, &["claude-code", "cursor"], 30);
    let said = silent(&home.hook_named(name(), "claude-code", in_cursor));
    assert_eq!(said, "");
    assert!(!home.hook("cursor").stdout.is_empty());
}

#[test]
fn an_exit_two_on_any_event_but_before_a_tool_blocks_nothing() {
    let home = Home::new();
    for event in [Event::Start, Event::Prompt, Event::AfterTool, Event::Stop] {
        home.registers(denying(), event, &AGENTS, 30);
        assert_eq!(silent(&home.hook("codex")), "", "{event:?}");
    }
}

#[test]
fn a_script_that_outlasts_its_time_is_stopped_and_never_blocks() {
    let home = Home::new();
    home.registers(sleeping(), Event::BeforeTool, &AGENTS, 1);
    let began = std::time::Instant::now();
    assert_eq!(silent(&home.hook("codex")), "");
    assert!(began.elapsed().as_secs() < 6, "{:?}", began.elapsed());
}

#[test]
fn anything_but_the_exact_arguments_does_nothing_and_still_succeeds() {
    let home = Home::new();
    home.registers(denying(), Event::BeforeTool, &AGENTS, 30);
    for args in [
        vec!["run-hook"],
        vec!["run-hook", "--agent", "codex", "--name", name()],
        vec!["run-hook", name(), "codex"],
        vec!["run-hook", "--name", name()],
        vec!["run-hook", "--name", name(), "--agent", "codex", "--more"],
    ] {
        assert_eq!(silent(&home.run(&args, "{}")), "", "{args:?}");
    }
}
