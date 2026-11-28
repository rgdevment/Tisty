use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Output, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

struct Home {
    home: TempDir,
}

impl Home {
    fn new() -> Self {
        let home = Self {
            home: tempfile::tempdir().unwrap(),
        };
        home.person().begins("algo mio");
        home
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn tisty(&self, args: &[&str]) -> Command {
        let root = self.home.path();
        let mut tisty = Command::new(env!("CARGO_BIN_EXE_tisty"));
        tisty
            .args(args)
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        tisty
    }

    fn hook(&self, client: &str, moment: &str, heard: &Value) -> Output {
        let mut child = self.tisty(&["hook", client, moment]).spawn().unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(heard.to_string().as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    fn connect(&self, client: &str) -> Connection {
        let mut child = self.tisty(&["mcp"]).stderr(Stdio::null()).spawn().unwrap();
        let into = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        let mut connection = Connection {
            child,
            into,
            from,
            next: 0,
        };
        connection.ask(
            "initialize",
            json!({ "protocolVersion": "2025-06-18", "capabilities": {},
                    "clientInfo": { "name": client, "version": "1" } }),
        );
        connection
    }
}

struct Connection {
    child: Child,
    into: ChildStdin,
    from: BufReader<ChildStdout>,
    next: u64,
}

impl Connection {
    fn ask(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        let asked =
            json!({ "jsonrpc": "2.0", "id": self.next, "method": method, "params": params });
        writeln!(self.into, "{asked}").unwrap();
        self.into.flush().unwrap();
        let mut line = String::new();
        self.from.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }

    fn call(&mut self, name: &str, args: Value) -> Value {
        let said = self.ask("tools/call", json!({ "name": name, "arguments": args }));
        assert_ne!(said["result"]["isError"], true, "{name}: {said}");
        said["result"]["structuredContent"].clone()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn context(said: &Output) -> String {
    assert!(said.status.success(), "{said:?}");
    let told: Value = serde_json::from_slice(&said.stdout).unwrap();
    assert_eq!(told["hookSpecificOutput"]["hookEventName"], "SessionStart");
    told["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap()
        .to_string()
}

fn started(home: &Home) -> String {
    context(&home.hook(
        "claude-code",
        "start",
        &json!({ "session_id": "s-start", "source": "startup" }),
    ))
}

#[test]
fn a_session_starts_knowing_the_open_tasks_this_client_left() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    for title in ["Ship the hooks", "Measure the hooks"] {
        agent.call(
            "task_create",
            json!({ "for": "agent", "title": title, "list": "Personal", "steps": ["one", "two"] }),
        );
    }
    let first = home.person().id_of("Ship the hooks");
    agent.call(
        "task_edit_steps",
        json!({ "action": "tick", "task": first.to_string(), "step": "one" }),
    );
    let mut other = home.connect("codex");
    other.call(
        "task_create",
        json!({ "for": "agent", "title": "Codex's own work", "steps": ["x"] }),
    );

    let said = started(&home);
    let mine: Vec<&str> = said
        .lines()
        .skip_while(|one| !one.starts_with("Your last open tasks"))
        .skip(1)
        .take_while(|one| one.starts_with("- "))
        .collect();

    assert_eq!(mine.len(), 2, "{said}");
    assert!(mine[0].contains("Ship the hooks"), "{said}");
    assert!(
        mine[0].contains("1 of 2 steps ticked, next \"two\""),
        "{said}"
    );
    assert!(mine[1].contains("Measure the hooks"), "{said}");
    assert!(said.contains("by Codex"), "{said}");
    assert!(said.chars().count() <= 2_000);
}

#[test]
fn a_session_does_not_start_with_the_tasks_filed_for_the_person() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    agent.call(
        "task_create",
        json!({ "for": "agent", "title": "Ship the hooks", "list": "Personal", "steps": ["one"] }),
    );
    agent.call(
        "task_create",
        json!({ "for": "person", "title": "Try the hooks by hand", "list": "Personal", "steps": ["open it"] }),
    );

    let said = started(&home);

    assert!(said.contains("Ship the hooks"), "{said}");
    assert!(!said.contains("Try the hooks by hand"), "{said}");
}

#[test]
fn a_person_task_made_by_handing_steps_over_does_not_start_a_session_either() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    agent.call(
        "task_create",
        json!({ "for": "agent", "title": "Ship the hooks", "list": "Personal", "steps": ["one", "try it"] }),
    );
    let id = home.person().id_of("Ship the hooks").to_string();
    agent.call(
        "task_edit_steps",
        json!({ "action": "remove", "task": id, "step": "try it", "to_person": true }),
    );

    let said = started(&home);

    assert!(said.contains("Ship the hooks"), "{said}");
    assert!(!said.contains("your part"), "{said}");
}

#[test]
fn the_hook_and_catch_up_say_the_same_of_the_task_left_open() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    agent.call(
        "task_create",
        json!({ "for": "agent", "title": "Ship the hooks", "list": "Personal", "steps": ["one", "two"] }),
    );
    let id = home.person().id_of("Ship the hooks").to_string();
    agent.call(
        "task_edit_steps",
        json!({ "action": "tick", "task": id, "step": "one", "state": "Blocked: waiting for the signing key." }),
    );

    let started = started(&home);
    let caught = agent.ask(
        "tools/call",
        json!({ "name": "workspace_catch_up", "arguments": {} }),
    );
    let last = &caught["result"]["structuredContent"]["last"];
    let said = caught["result"]["content"][0]["text"].as_str().unwrap();

    let progress =
        "blocked, 1 of 2 steps ticked, next \"two\"; state: Blocked: waiting for the signing key.";
    assert!(started.contains(progress), "{started}");
    assert!(said.contains(progress), "{said}");
    assert_eq!(last["stage"], "blocked", "{last}");
    assert_eq!(last["steps"], json!([1, 2]), "{last}");
    assert_eq!(
        last["said"], "blocked, 1 of 2 steps ticked, next \"two\"",
        "a reader of the JSON alone is told in words too: {last}"
    );
    assert_eq!(
        last["state"], "Blocked: waiting for the signing key.",
        "{last}"
    );
}

#[test]
fn what_the_person_moved_since_the_last_catch_up_comes_with_it() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    agent.call("workspace_catch_up", json!({}));
    std::thread::sleep(std::time::Duration::from_millis(20));
    home.person().captures("Buy the new cable");

    let said = started(&home);

    assert!(said.contains("Since you were last here"), "{said}");
    assert!(said.contains("Buy the new cable"), "{said}");
}

#[test]
fn a_hidden_task_and_the_diary_never_reach_a_starting_session() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    agent.call(
        "task_create",
        json!({ "for": "agent", "title": "Secret errand", "list": "Personal", "steps": ["one"] }),
    );
    let secret = home.person().id_of("Secret errand");
    agent.call("workspace_catch_up", json!({}));
    std::thread::sleep(std::time::Duration::from_millis(20));
    home.person().hides(secret);
    let today = jiff::ZonedDateTime::now().date();
    home.person()
        .writes_in_the_diary(today, "Dear diary, nobody reads this");

    let said = started(&home);

    assert!(!said.contains("Secret errand"), "{said}");
    assert!(!said.contains(&secret.to_string()), "{said}");
    assert!(!said.to_lowercase().contains("diary"), "{said}");
    assert!(!said.to_lowercase().contains("diario"), "{said}");
}

#[test]
fn a_closed_door_starts_a_session_with_nothing() {
    let home = Home::new();

    let said = home.hook("claude-code", "start", &json!({ "session_id": "s" }));

    assert!(said.status.success());
    assert!(said.stdout.is_empty(), "{said:?}");
}

#[test]
fn a_turn_that_edited_and_wrote_nothing_is_handed_back_through_the_binary() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let session = json!({ "session_id": "s-turn", "tool_name": "Write" });

    let edited = home.hook("claude-code", "tool", &session);
    let stopped = home.hook(
        "claude-code",
        "stop",
        &json!({ "session_id": "s-turn", "stop_hook_active": false }),
    );
    let again = home.hook(
        "claude-code",
        "stop",
        &json!({ "session_id": "s-turn", "stop_hook_active": true }),
    );

    assert!(edited.status.success() && edited.stdout.is_empty());
    let handed: Value = serde_json::from_slice(&stopped.stdout).unwrap();
    assert_eq!(handed["decision"], "block");
    assert!(again.status.success() && again.stdout.is_empty());
}

#[test]
fn anything_unexpected_exits_quietly() {
    let home = Home::new();
    for args in [
        vec!["hook"],
        vec!["hook", "nobody", "stop"],
        vec!["hook", "claude-code"],
        vec!["hook", "claude-code", "stop", "extra"],
    ] {
        let mut child = home.tisty(&args).spawn().unwrap();
        // A hook that gives up on its arguments may leave before reading, closing the pipe first.
        let _ = child.stdin.take().unwrap().write_all(b"not json");
        let said = child.wait_with_output().unwrap();
        assert!(said.status.success(), "{args:?}: {said:?}");
        assert!(said.stdout.is_empty(), "{args:?}: {said:?}");
    }
}

#[test]
fn a_start_heard_twice_in_one_session_speaks_once_and_another_session_still_hears_it() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let heard = json!({ "session_id": "s-twice", "source": "startup" });

    let first = context(&home.hook("claude-code", "start", &heard));
    // A start is kept for its 30 s window, and two runs may straddle a window's edge, but three never straddle two.
    let again: Vec<Output> = (0..2)
        .map(|_| home.hook("claude-code", "start", &heard))
        .collect();
    let other = home.hook(
        "claude-code",
        "start",
        &json!({ "session_id": "s-other", "source": "startup" }),
    );

    assert!(first.starts_with("Tisty holds"), "{first}");
    assert!(again.iter().all(|one| one.status.success()));
    assert!(again.iter().any(|one| one.stdout.is_empty()), "{again:?}");
    assert!(context(&other).starts_with("Tisty holds"));
}

#[test]
fn the_switch_turned_off_in_the_settings_silences_a_start_through_the_binary() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let config = home.home.path().join("config").join("config.toml");
    let kept = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("turn_hooks = false\n{kept}")).unwrap();

    let said = home.hook(
        "claude-code",
        "start",
        &json!({ "session_id": "s-off", "source": "startup" }),
    );

    assert!(said.status.success());
    assert!(said.stdout.is_empty(), "{said:?}");
}

fn outside_file() -> String {
    match cfg!(windows) {
        true => r"X:\Nowhere\Person\Docs\notes.md".to_string(),
        false => "/nowhere/person/Docs/notes.md".to_string(),
    }
}

#[test]
fn a_new_text_file_outside_is_denied_once_through_the_binary() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let heard = json!({
        "session_id": "s-before",
        "tool_name": "Write",
        "tool_input": { "file_path": outside_file(), "content": "x" },
    });

    let first = home.hook("claude-code", "before", &heard);
    let second = home.hook("claude-code", "before", &heard);

    assert!(first.status.success(), "{first:?}");
    let denied: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(denied["hookSpecificOutput"]["permissionDecision"], "deny");
    assert!(
        second.status.success() && second.stdout.is_empty(),
        "{second:?}"
    );
}

#[test]
fn garbage_before_a_call_is_silent_through_the_binary() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    for raw in [&b"not json"[..], b"", b"{\"session_id\":\"s\"}"] {
        let mut child = home
            .tisty(&["hook", "claude-code", "before"])
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(raw).unwrap();
        let said = child.wait_with_output().unwrap();
        assert!(said.status.success(), "{raw:?}: {said:?}");
        assert!(said.stdout.is_empty(), "{raw:?}: {said:?}");
    }
}

const PUBLISHED: &str = "You just pushed or opened or merged a pull request";
const EVERY_STEP: &str = "is ticked, but it is not said done";
const EDITED: &str = "You edited files this turn and wrote nothing in Tisty";

fn said(out: &Output) -> String {
    assert!(out.status.success(), "{out:?}");
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stopped(home: &Home, client: &str, session: &Value) -> String {
    said(&home.hook(client, "stop", session))
}

fn in_session(session: &str) -> Value {
    json!({ "session_id": session, "stop_hook_active": false })
}

fn a_tick_of(session: &str, task: &str) -> Value {
    json!({ "session_id": session, "tool_name": "mcp__tisty__task_edit_steps",
            "tool_input": { "action": "tick", "task": task, "step": "one" } })
}

fn an_agent_task(home: &Home, agent: &mut Connection, title: &str, steps: &[&str]) -> String {
    agent.call(
        "task_create",
        json!({ "for": "agent", "title": title, "list": "Personal", "steps": steps }),
    );
    home.person().id_of(title).to_string()
}

fn every_step_ticked(agent: &mut Connection, task: &str, steps: &[&str]) {
    agent.call(
        "task_edit_steps",
        json!({ "action": "tick", "task": task, "steps": steps }),
    );
}

fn shell_push(client: &str, session: &str) -> Value {
    match client {
        "cursor" => json!({ "conversation_id": session, "command": "git push origin feat/x" }),
        "copilot" => json!({ "sessionId": session, "toolName": "bash",
                             "toolArgs": json!({ "command": "git push" }).to_string() }),
        "antigravity" => json!({ "conversation_id": session,
                                 "toolCall": { "name": "run_command",
                                               "args": { "CommandLine": "git push" } } }),
        _ => json!({ "session_id": session, "tool_name": "Bash",
                     "tool_input": { "command": "git push" } }),
    }
}

fn stop_of(client: &str, session: &str) -> Value {
    match client {
        "cursor" | "antigravity" => json!({ "conversation_id": session }),
        "copilot" => json!({ "sessionId": session }),
        _ => in_session(session),
    }
}

#[test]
fn claude_code_is_asked_what_it_finished_right_after_a_push_and_the_stop_does_not_ask_again() {
    let home = Home::new();
    home.person().lets_the_agent_in();

    let after = said(&home.hook("claude-code", "shell", &shell_push("claude-code", "s-push")));
    let told: Value = serde_json::from_str(&after).unwrap();
    assert_eq!(told["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    let context = told["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains(PUBLISHED), "{context}");
    assert!(context.contains("`task_resolve`"), "{context}");

    assert_eq!(stopped(&home, "claude-code", &in_session("s-push")), "");
}

#[test]
fn claude_code_is_asked_after_a_pull_request_is_opened_or_merged_too() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    for (n, command) in ["gh pr create --fill", "gh -R a/b pr merge 3 --squash"]
        .into_iter()
        .enumerate()
    {
        let session = format!("s-pr-{n}");
        let heard = json!({ "session_id": session, "tool_name": "Bash",
                            "tool_input": { "command": command } });
        let after = said(&home.hook("claude-code", "shell", &heard));
        assert!(after.contains(PUBLISHED), "{command}: {after}");
    }
}

#[test]
fn a_shell_command_that_does_not_publish_says_nothing_and_leaves_the_stop_quiet() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    for command in ["git status", "gh pr view 3", "echo \"git push\"", "ls"] {
        let heard = json!({ "session_id": "s-calm", "tool_name": "Bash",
                            "tool_input": { "command": command } });
        assert_eq!(
            said(&home.hook("claude-code", "shell", &heard)),
            "",
            "{command}"
        );
    }
    assert_eq!(stopped(&home, "claude-code", &in_session("s-calm")), "");
}

#[test]
fn a_push_inside_a_here_document_asks_nothing() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let heard = json!({ "session_id": "s-doc", "tool_name": "Bash",
                        "tool_input": { "command": "cat <<EOF > plan.md\ngit push\nEOF" } });

    assert_eq!(said(&home.hook("claude-code", "shell", &heard)), "");
    assert_eq!(stopped(&home, "claude-code", &in_session("s-doc")), "");
}

#[test]
fn the_other_clients_say_nothing_at_the_shell_and_ask_once_at_the_stop() {
    for client in ["codex", "cursor", "gemini-cli", "copilot", "antigravity"] {
        let home = Home::new();
        home.person().lets_the_agent_in();
        let session = format!("s-{client}");

        let after = said(&home.hook(client, "shell", &shell_push(client, &session)));
        assert_eq!(after, "", "{client} at the shell");

        let first = stopped(&home, client, &stop_of(client, &session));
        assert!(first.contains(PUBLISHED), "{client} at the stop: {first}");
        let second = stopped(&home, client, &stop_of(client, &session));
        assert_eq!(second, "", "{client} asked twice");
    }
}

#[test]
fn the_stop_after_a_push_hands_the_question_back_in_each_clients_shape() {
    let shape = |client: &str| -> Value {
        let home = Home::new();
        home.person().lets_the_agent_in();
        let session = format!("s-{client}");
        home.hook(client, "shell", &shell_push(client, &session));
        serde_json::from_str(&stopped(&home, client, &stop_of(client, &session))).unwrap()
    };
    assert_eq!(shape("codex")["decision"], "block");
    assert_eq!(shape("gemini-cli")["decision"], "block");
    assert_eq!(shape("copilot")["decision"], "block");
    assert_eq!(shape("antigravity")["decision"], "continue");
    assert!(
        shape("cursor")["followup_message"]
            .as_str()
            .unwrap()
            .contains(PUBLISHED)
    );
}

#[test]
fn a_second_push_in_the_same_session_is_asked_about_again_at_the_stop() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let session = "s-twice";
    home.hook("codex", "shell", &shell_push("codex", session));
    assert!(stopped(&home, "codex", &in_session(session)).contains(PUBLISHED));
    assert_eq!(stopped(&home, "codex", &in_session(session)), "");

    home.hook("codex", "shell", &shell_push("codex", session));
    assert!(stopped(&home, "codex", &in_session(session)).contains(PUBLISHED));
}

#[test]
fn a_stop_that_is_already_a_continuation_asks_about_nothing() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    home.hook("codex", "shell", &shell_push("codex", "s-active"));

    let active = json!({ "session_id": "s-active", "stop_hook_active": true });
    assert_eq!(stopped(&home, "codex", &active), "");
    assert_eq!(stopped(&home, "codex", &in_session("s-active")), "");
}

#[test]
fn a_closed_door_asks_nothing_after_a_push() {
    let home = Home::new();
    for client in ["claude-code", "codex"] {
        assert_eq!(
            said(&home.hook(client, "shell", &shell_push(client, "s-closed"))),
            ""
        );
        assert_eq!(stopped(&home, client, &in_session("s-closed")), "");
    }
}

#[test]
fn turn_hooks_off_asks_nothing_after_a_push_or_at_the_stop() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let config = home.home.path().join("config").join("config.toml");
    let kept = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("turn_hooks = false\n{kept}")).unwrap();

    let after = said(&home.hook("claude-code", "shell", &shell_push("claude-code", "s-off2")));
    assert_eq!(after, "");
    assert_eq!(stopped(&home, "claude-code", &in_session("s-off2")), "");
}

#[test]
fn a_task_with_every_step_ticked_is_asked_to_be_said_done_once_at_the_stop() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one", "two"]);
    every_step_ticked(&mut agent, &task, &["one", "two"]);
    home.hook("claude-code", "tool", &a_tick_of("s-ready", &task));

    let asked = stopped(&home, "claude-code", &in_session("s-ready"));
    let told: Value = serde_json::from_str(&asked).unwrap();
    assert_eq!(told["decision"], "block");
    let reason = told["reason"].as_str().unwrap();
    assert!(reason.contains("Ship the hooks"), "{reason}");
    assert!(reason.contains(EVERY_STEP), "{reason}");
    assert!(reason.contains("`task_resolve`"), "{reason}");

    assert_eq!(stopped(&home, "claude-code", &in_session("s-ready")), "");
}

#[test]
fn a_task_with_a_step_left_is_not_asked_to_be_said_done() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one", "two"]);
    every_step_ticked(&mut agent, &task, &["one"]);
    home.hook("claude-code", "tool", &a_tick_of("s-left", &task));

    assert_eq!(stopped(&home, "claude-code", &in_session("s-left")), "");
}

#[test]
fn a_task_without_steps_is_not_asked_to_be_said_done() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &[]);
    home.hook("claude-code", "tool", &a_tick_of("s-bare", &task));

    assert_eq!(stopped(&home, "claude-code", &in_session("s-bare")), "");
}

#[test]
fn a_task_already_said_done_is_not_asked_again() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one"]);
    every_step_ticked(&mut agent, &task, &["one"]);
    agent.call(
        "task_resolve",
        json!({ "outcome": "done", "task": task, "body": "built and proven", "branch": "feat/x" }),
    );
    home.hook("claude-code", "tool", &a_tick_of("s-said", &task));

    assert_eq!(stopped(&home, "claude-code", &in_session("s-said")), "");
}

#[test]
fn a_task_for_the_person_is_never_asked_to_be_said_done() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    agent.call(
        "task_create",
        json!({ "for": "person", "title": "Try it by hand", "list": "Personal",
                "steps": ["one"] }),
    );
    let task = home.person().id_of("Try it by hand");
    home.person().ticks(task, 1);
    home.hook(
        "claude-code",
        "tool",
        &a_tick_of("s-theirs", &task.to_string()),
    );

    assert_eq!(stopped(&home, "claude-code", &in_session("s-theirs")), "");
}

#[test]
fn a_task_the_person_wrote_is_never_asked_to_be_said_done() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let task = home.person().id_of("algo mio");
    home.person().adds_step(task, "one");
    home.person().ticks(task, 1);
    home.hook(
        "claude-code",
        "tool",
        &a_tick_of("s-persons", &task.to_string()),
    );

    assert_eq!(stopped(&home, "claude-code", &in_session("s-persons")), "");
}

#[test]
fn only_a_task_written_through_a_tisty_write_tool_is_asked_about() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one"]);
    every_step_ticked(&mut agent, &task, &["one"]);

    let read = json!({ "session_id": "s-read", "tool_name": "mcp__tisty__workspace_read",
                       "tool_input": { "task": task } });
    home.hook("claude-code", "tool", &read);

    assert_eq!(stopped(&home, "claude-code", &in_session("s-read")), "");
}

#[test]
fn a_continuation_stop_does_not_ask_to_say_done_and_a_later_turn_that_writes_still_does() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one"]);
    every_step_ticked(&mut agent, &task, &["one"]);
    home.hook("claude-code", "tool", &a_tick_of("s-cont", &task));

    let active = json!({ "session_id": "s-cont", "stop_hook_active": true });
    assert_eq!(stopped(&home, "claude-code", &active), "");
    assert_eq!(stopped(&home, "claude-code", &in_session("s-cont")), "");
    home.hook("claude-code", "tool", &a_tick_of("s-cont", &task));
    assert!(stopped(&home, "claude-code", &in_session("s-cont")).contains(EVERY_STEP));
}

#[test]
fn turn_hooks_off_asks_nothing_about_a_task_ready_to_be_said_done() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one"]);
    every_step_ticked(&mut agent, &task, &["one"]);
    let config = home.home.path().join("config").join("config.toml");
    let kept = std::fs::read_to_string(&config).unwrap();
    std::fs::write(&config, format!("turn_hooks = false\n{kept}")).unwrap();
    home.hook("claude-code", "tool", &a_tick_of("s-off3", &task));

    assert_eq!(stopped(&home, "claude-code", &in_session("s-off3")), "");
}

#[test]
fn a_closed_door_asks_nothing_about_a_task_ready_to_be_said_done() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one"]);
    every_step_ticked(&mut agent, &task, &["one"]);
    home.hook("claude-code", "tool", &a_tick_of("s-door", &task));
    home.person().shuts_the_agent_out();

    assert_eq!(stopped(&home, "claude-code", &in_session("s-door")), "");
}

#[test]
fn the_hand_back_of_a_turn_that_wrote_nothing_wins_over_the_question_to_say_done() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one", "two"]);
    every_step_ticked(&mut agent, &task, &["one"]);
    home.hook("claude-code", "tool", &a_tick_of("s-win", &task));
    assert_eq!(stopped(&home, "claude-code", &in_session("s-win")), "");

    home.person().ticks(task.parse().unwrap(), 2);
    let edit = json!({ "session_id": "s-win", "tool_name": "Write" });
    home.hook("claude-code", "tool", &edit);

    let handed: Value =
        serde_json::from_str(&stopped(&home, "claude-code", &in_session("s-win"))).unwrap();
    assert_eq!(handed["decision"], "block");
    let reason = handed["reason"].as_str().unwrap();
    assert!(reason.contains(EDITED), "{reason}");
    assert!(!reason.contains(EVERY_STEP), "{reason}");
}

#[test]
fn the_question_to_say_done_is_asked_only_in_a_turn_that_wrote_in_tisty() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one", "two"]);
    every_step_ticked(&mut agent, &task, &["one"]);
    home.hook("claude-code", "tool", &a_tick_of("s-turns", &task));
    assert_eq!(stopped(&home, "claude-code", &in_session("s-turns")), "");

    home.person().ticks(task.parse().unwrap(), 2);
    assert_eq!(
        stopped(&home, "claude-code", &in_session("s-turns")),
        "",
        "a turn that wrote nothing in Tisty is not asked"
    );

    home.hook("claude-code", "tool", &a_tick_of("s-turns", &task));
    let asked = stopped(&home, "claude-code", &in_session("s-turns"));
    assert!(asked.contains(EVERY_STEP), "{asked}");
}

#[test]
fn a_task_ticked_complete_by_the_person_in_a_turn_without_a_tisty_write_is_not_asked_about() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    let task = an_agent_task(&home, &mut agent, "Ship the hooks", &["one"]);
    home.hook("claude-code", "tool", &a_tick_of("s-quiet", &task));
    assert_eq!(stopped(&home, "claude-code", &in_session("s-quiet")), "");

    home.person().ticks(task.parse().unwrap(), 1);

    assert_eq!(stopped(&home, "claude-code", &in_session("s-quiet")), "");
}
