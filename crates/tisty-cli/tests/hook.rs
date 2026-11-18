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
            "propose",
            json!({ "title": title, "list": "Personal", "steps": ["one", "two"] }),
        );
    }
    let first = home.person().id_of("Ship the hooks");
    agent.call("tick", json!({ "task": first.to_string(), "step": "one" }));
    let mut other = home.connect("codex");
    other.call(
        "propose",
        json!({ "title": "Codex's own work", "steps": ["x"] }),
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
fn what_the_person_moved_since_the_last_catch_up_comes_with_it() {
    let home = Home::new();
    home.person().lets_the_agent_in();
    let mut agent = home.connect("claude-code");
    agent.call("catch_up", json!({}));
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
        "propose",
        json!({ "title": "Secret errand", "list": "Personal", "steps": ["one"] }),
    );
    let secret = home.person().id_of("Secret errand");
    agent.call("catch_up", json!({}));
    std::thread::sleep(std::time::Duration::from_millis(20));
    home.person().hides(secret);
    let today = jiff::Zoned::now().date();
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
        child.stdin.take().unwrap().write_all(b"not json").unwrap();
        let said = child.wait_with_output().unwrap();
        assert!(said.status.success(), "{args:?}: {said:?}");
        assert!(said.stdout.is_empty(), "{args:?}: {said:?}");
    }
}
