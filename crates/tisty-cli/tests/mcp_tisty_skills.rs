use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;
use tisty_core::agents::{self, Bundled, GLOBAL, Kind, Ledger, Standing};

mod person;

const SAID: &str = "---\nname: tisty\n---\n\nZorblax: the word only Tisty's own skill carries.\n";
const OURS: &str = "tisty:skill/tisty/SKILL.md";
const NOBODY: &str = "dev_nobody-0001";

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        let (_, claude) = person.keeps_agents();
        person.writes_doc_in(
            "---\nagent: claude/rule/CLAUDE.md\ntitle: claude · CLAUDE.md\n---\n# Claude\nTests first.\n",
            claude,
        );
        person.writes_doc("# Notes\n\nSomething of mine.\n");
        let served = Self { home };
        served.tistys_own_skill_written();
        served
    }

    fn data(&self) -> std::path::PathBuf {
        self.home.path().join("data")
    }

    fn agent_home(&self) -> std::path::PathBuf {
        self.home.path().join("agent-home")
    }

    // What the window does with this computer's yes and the switch on, done here through the same core.
    fn tistys_own_skill_written(&self) {
        let mut ledger =
            Ledger::fresh(&tisty_core::machine::here().unwrap_or_else(|| "machine-test".into()));
        ledger.consent(true);
        ledger.take_tisty_skills(true);
        let state = person::Person::at(self.home.path()).state();
        let mut gate = agents::gate(&agents::Asked {
            state: &state,
            docs: &self.data().join("docs"),
            root: agents::root_of(&state),
            ledger: &ledger,
            me: &tisty_core::event::DeviceId("dev_here".into()),
        });
        let body = agents::doc_of(GLOBAL, Kind::Skill, "tisty/SKILL.md", SAID.as_bytes()).unwrap();
        gate.admit_builtins(
            &ledger,
            &[Bundled {
                skill: "tisty".into(),
                name: "tisty/SKILL.md".into(),
                body,
            }],
        );
        let root = self.agent_home().join(".claude");
        let at = root.join("skills/tisty/SKILL.md");
        let mut target = gate.target(&ledger, root, at.clone(), "claude");
        for piece in gate.admitted() {
            target.put(piece);
        }
        assert_eq!(
            agents::round(&self.data(), &mut ledger, &[target]),
            [Standing::Wrote]
        );
        assert_eq!(std::fs::read_to_string(at).unwrap(), SAID);
    }

    fn call(&self, name: &str, args: Value) -> Value {
        let root = self.home.path();
        let asked = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
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
}

fn never_names_them(said: &Value) {
    let text = said.to_string();
    for word in [OURS, "tisty:skill/", "Zorblax", "zorblax"] {
        assert!(!text.contains(word), "{word} in {text}");
    }
}

fn as_nobody(said: &Value, asked: &str) -> String {
    said.to_string().replace(asked, NOBODY)
}

#[test]
fn brain_never_sees_tistys_own_skills() {
    let served = Served::new();
    for args in [
        json!({}),
        json!({ "query": "tisty skill" }),
        json!({ "scope": "either" }),
    ] {
        never_names_them(&served.call("brain", args));
    }
    let searched =
        &served.call("brain", json!({ "query": "Zorblax" }))["result"]["structuredContent"];
    assert_eq!(searched["docsTotal"], 0, "{searched}");
    assert_eq!(searched["total"], 0, "{searched}");
}

#[test]
fn reading_or_exporting_one_of_tistys_own_skills_answers_as_for_a_document_nobody_has() {
    let served = Served::new();
    let into = tempfile::tempdir().unwrap();
    for (tool, extra) in [
        ("read", json!({})),
        ("brain", json!({})),
        ("export_doc", json!({ "into": into.path() })),
    ] {
        let ask = |doc: &str| {
            let mut args = extra.clone();
            args["doc"] = json!(doc);
            served.call(tool, args)
        };
        let ours = ask(OURS);
        let nobodys = ask(NOBODY);
        assert_eq!(as_nobody(&ours, OURS), nobodys.to_string(), "{tool}");
        never_names_them(&nobodys);
    }
    assert!(std::fs::read_dir(into.path()).unwrap().next().is_none());
}

#[test]
fn no_tool_that_writes_can_reach_tistys_own_skills() {
    let served = Served::new();
    let at = served.agent_home().join(".claude/skills/tisty/SKILL.md");
    for (tool, extra) in [
        (
            "write_doc",
            json!({ "print": "0", "body": "# Mine\n\nZorblax gone.\n" }),
        ),
        ("edit_doc", json!({ "old": "Zorblax", "new": "Mine" })),
        ("append_doc", json!({ "body": "Mine." })),
        ("archive_doc", json!({})),
        ("flag_doc", json!({ "why": "duplicate" })),
        ("restore_doc", json!({})),
    ] {
        let ask = |doc: &str| {
            let mut args = extra.clone();
            args["doc"] = json!(doc);
            served.call(tool, args)
        };
        let ours = ask(OURS);
        let nobodys = ask(NOBODY);
        assert_eq!(as_nobody(&ours, OURS), nobodys.to_string(), "{tool}");
        assert!(
            ours.to_string().contains("isError\":true") || ours.get("error").is_some(),
            "{tool} refuses: {ours}"
        );
        never_names_them(&nobodys);
    }
    assert_eq!(std::fs::read_to_string(&at).unwrap(), SAID);
    assert!(Path::new(&at).is_file());
}
