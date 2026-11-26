use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

const RULE: &str =
    "---\nagent: claude/rule/CLAUDE.md\ntitle: claude · CLAUDE.md\n---\n# Claude\nTests first.\n";
const RULE_AGAIN: &str =
    "---\nagent: claude/rule/CLAUDE.md\ntitle: claude · CLAUDE.md\n---\n# Claude\nTests second.\n";
const RULE_LAST: &str =
    "---\nagent: claude/rule/CLAUDE.md\ntitle: claude · CLAUDE.md\n---\n# Claude\nTests third.\n";
const SECRET: &str = "SECRET-COPY-MARKER-7f3a91";
const AGENT_FILE: &str = "/home/rina/.claude/CLAUDE.md";

struct Served {
    home: TempDir,
    rule: String,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        let (_, claude) = person.keeps_agents();
        let rule = person.writes_doc_in(RULE, claude);
        person.rewrites_doc(&rule, RULE_AGAIN);
        person.rewrites_doc(&rule, RULE_LAST);
        Self { home, rule }
    }

    fn data(&self) -> PathBuf {
        self.home.path().join("data")
    }

    fn keeps_a_copy(&self, words: &str) {
        let at = std::path::Path::new(AGENT_FILE);
        let named = tisty_core::attach::printed(at.to_string_lossy().as_bytes());
        let shelf = tisty_core::agents::folder(&self.data())
            .join("kept")
            .join(&named[..24]);
        std::fs::create_dir_all(&shelf).unwrap();
        std::fs::write(shelf.join("where.txt"), AGENT_FILE).unwrap();
        std::fs::write(shelf.join("0000000001"), words).unwrap();
    }

    fn call(&self, args: Value) -> Value {
        let root = self.home.path();
        let hello = json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": { "protocolVersion": "2025-06-18", "capabilities": {},
                        "clientInfo": { "name": "claude-code", "version": "1" } },
        });
        let asked = json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": "read", "arguments": args },
        });
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
        writeln!(child.stdin.take().unwrap(), "{hello}\n{asked}").unwrap();
        let out = child.wait_with_output().unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|one| one["id"] == 2)
            .unwrap()["result"]
            .clone()
    }
}

#[test]
fn a_copy_of_an_agents_file_never_comes_out_through_the_versions_of_its_document() {
    let served = Served::new();
    served.keeps_a_copy(SECRET);
    let kept =
        tisty_core::agents::read_kept(&served.data(), std::path::Path::new(AGENT_FILE), 1).unwrap();
    assert!(String::from_utf8(kept).unwrap().contains(SECRET));

    let listed = served.call(json!({ "doc": served.rule, "versions": true }));
    assert_ne!(listed["isError"], json!(true), "{listed}");
    let versions: Vec<String> = listed["structuredContent"]["versions"]
        .as_array()
        .unwrap_or_else(|| panic!("{listed}"))
        .iter()
        .filter_map(|row| row["version"].as_str().map(str::to_string))
        .collect();
    assert!(!versions.is_empty(), "{listed}");
    assert!(!listed.to_string().contains(SECRET), "{listed}");

    for version in &versions {
        let read = served.call(json!({ "doc": served.rule, "version": version }));
        assert!(!read.to_string().contains(SECRET), "{read}");
    }
    let now = served.call(json!({ "doc": served.rule }));
    assert!(now.to_string().contains("Tests third."), "{now}");
    assert!(!now.to_string().contains(SECRET), "{now}");
}
