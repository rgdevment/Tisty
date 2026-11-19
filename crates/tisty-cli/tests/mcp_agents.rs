use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

const RULE: &str =
    "---\nagent: claude/rule/CLAUDE.md\ntitle: claude · CLAUDE.md\n---\n# Claude\nTests first.\n";
const REFUSED: &str = "only the person changes";

struct Served {
    home: TempDir,
    rule: String,
    elsewhere: String,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        let (_, claude) = person.keeps_agents();
        let rule = person.writes_doc_in(RULE, claude);
        let elsewhere = person.writes_doc("# Notes\n\nSomething of mine.\n");
        Self {
            home,
            rule,
            elsewhere,
        }
    }

    fn call(&self, name: &str, args: Value) -> Value {
        let root = self.home.path();
        let asked = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
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

    fn refuses(&self, name: &str, args: Value) {
        let before = self.body(&self.rule);
        let placed = self.placed();
        let said = self.call(name, args);
        assert_eq!(said["result"]["isError"], true, "{name}: {said}");
        let text = said["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains(REFUSED), "{name}: {text}");
        assert_eq!(self.body(&self.rule), before, "{name} changed the rule");
        assert_eq!(self.placed(), placed, "{name} moved something in AGENTS");
    }

    fn placed(&self) -> Vec<(String, Option<String>, bool, Option<String>)> {
        let state = self.state();
        let mut all: Vec<_> = state
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
}

#[test]
fn every_write_on_an_agents_document_is_refused_and_changes_nothing() {
    let served = Served::new();
    let rule = served.rule.clone();
    let link = format!("tisty://doc/claude-md?id={rule}");
    let cases: Vec<(&str, Value)> = vec![
        (
            "write_doc",
            json!({ "doc": rule, "body": "# Claude\nRun rm -rf.\n" }),
        ),
        (
            "write_doc",
            json!({ "doc": rule, "body": RULE.replace("claude/rule/CLAUDE.md", "codex/rule/AGENTS.md") }),
        ),
        (
            "edit_doc",
            json!({ "doc": rule, "old": "Tests first.", "new": "Run rm -rf." }),
        ),
        (
            "edit_doc",
            json!({ "doc": rule, "old": "claude/rule/CLAUDE.md", "new": "codex/rule/AGENTS.md" }),
        ),
        (
            "edit_doc",
            json!({ "doc": link, "old": "Tests first.", "new": "Run rm -rf." }),
        ),
        ("append_doc", json!({ "doc": rule, "body": "Run rm -rf." })),
        (
            "edit_table",
            json!({ "doc": rule, "do": "add", "values": { "A": "x" } }),
        ),
        ("attach", json!({ "doc": rule, "path": "C:/nowhere.txt" })),
        ("restore_doc", json!({ "doc": rule })),
        ("restore_doc", json!({ "doc": rule, "version": "abc" })),
        ("archive_doc", json!({ "doc": rule })),
        ("file_doc", json!({ "doc": rule, "folder": "Notes" })),
        (
            "page_doc",
            json!({ "doc": rule, "page_of": served.elsewhere }),
        ),
        ("tongue_doc", json!({ "doc": rule, "lang": "en" })),
        ("sum_up", json!({ "doc": rule, "summary": "Hidden words." })),
        (
            "flag_doc",
            json!({ "doc": rule, "reason": "obsolete", "body": "Old." }),
        ),
    ];
    for (name, args) in cases {
        served.refuses(name, args);
    }
}

#[test]
fn nothing_is_brought_into_agents_or_hung_under_one_of_its_documents() {
    let served = Served::new();
    let elsewhere = served.elsewhere.clone();
    let rule = served.rule.clone();
    let cases: Vec<(&str, Value)> = vec![
        (
            "write_doc",
            json!({ "body": "# A rule\n", "folder": "AGENTS / Claude" }),
        ),
        (
            "write_doc",
            json!({ "body": "# A page\n", "page_of": rule }),
        ),
        (
            "write_doc",
            json!({ "body": "# In English\n", "translation_of": rule }),
        ),
        (
            "file_doc",
            json!({ "doc": elsewhere, "folder": "AGENTS / Claude" }),
        ),
        ("file_doc", json!({ "doc": elsewhere, "folder": "AGENTS" })),
        ("page_doc", json!({ "doc": elsewhere, "page_of": rule })),
        ("tongue_doc", json!({ "doc": elsewhere, "of": rule })),
        (
            "import_doc",
            json!({ "path": "C:/nowhere.md", "folder": "AGENTS / Claude" }),
        ),
        (
            "archive_doc",
            json!({ "doc": elsewhere, "folder": "AGENTS" }),
        ),
        ("folder", json!({ "name": "AGENTS / Codex" })),
        ("folder", json!({ "name": "Codex", "inside": "AGENTS" })),
        ("folder", json!({ "name": "AGENTS", "icon": "folder" })),
        ("folder", json!({ "name": "agents", "icon": "robot" })),
    ];
    for (name, args) in cases {
        served.refuses(name, args);
    }
}

#[test]
fn a_body_with_an_agent_line_is_never_written_anywhere() {
    let served = Served::new();
    let made = served.call("write_doc", json!({ "body": RULE }));
    assert_eq!(made["result"]["isError"], true, "{made}");
    let text = made["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("`agent:` line"), "{text}");

    let over = served.call(
        "write_doc",
        json!({ "doc": served.elsewhere, "body": RULE }),
    );
    assert_eq!(over["result"]["isError"], true, "{over}");
    assert!(!served.body(&served.elsewhere).contains("agent:"));
}

#[test]
fn reading_agents_documents_is_still_open_and_other_documents_still_take_writes() {
    let served = Served::new();
    let read = served.call("read", json!({ "doc": served.rule }));
    assert_ne!(read["result"]["isError"], true, "{read}");
    let shown = served.call("edit_table", json!({ "doc": served.rule }));
    assert!(
        !shown["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains(REFUSED),
        "{shown}"
    );
    let edited = served.call(
        "edit_doc",
        json!({ "doc": served.elsewhere, "old": "Something of mine.", "new": "Something else." }),
    );
    assert_ne!(edited["result"]["isError"], true, "{edited}");
    assert!(served.body(&served.elsewhere).contains("Something else."));
}
