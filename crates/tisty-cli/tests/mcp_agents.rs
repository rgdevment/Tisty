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
        assert!(self.state().proposals.is_empty(), "{name} left a proposal");
    }

    fn proposes(&self, name: &str, args: Value) -> Value {
        let before = self.body(&self.rule);
        let kept = self.kept();
        let said = self.call(name, args);
        assert_ne!(said["result"]["isError"], true, "{name}: {said}");
        assert_eq!(self.body(&self.rule), before, "{name} wrote the rule");
        assert_eq!(
            self.kept(),
            kept,
            "{name} changed what the log says of the rule"
        );
        said["result"]["structuredContent"].clone()
    }

    fn kept(&self) -> tisty_core::model::Kept {
        let state = self.state();
        state
            .docs
            .values()
            .find(|one| one.file == self.rule)
            .unwrap()
            .clone()
    }

    fn proposal(&self) -> Option<String> {
        let state = self.state();
        state
            .proposals
            .get(&self.kept().id)
            .map(|one| one.body.clone())
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
fn every_write_on_an_agents_document_but_a_proposal_is_refused_and_changes_nothing() {
    let served = Served::new();
    let rule = served.rule.clone();
    let link = format!("tisty://doc/claude-md?id={rule}");
    let print = tisty_core::docs::print_of_text(&served.body(&rule));
    let cases: Vec<(&str, Value)> = vec![
        (
            "write_doc",
            json!({ "doc": rule, "print": print, "body": RULE.replace("claude/rule/CLAUDE.md", "codex/rule/AGENTS.md") }),
        ),
        (
            "write_doc",
            json!({ "doc": rule, "print": print, "body": RULE.replace("agent: claude/rule/CLAUDE.md\n", "") }),
        ),
        (
            "edit_doc",
            json!({ "doc": rule, "old": "claude/rule/CLAUDE.md", "new": "codex/rule/AGENTS.md" }),
        ),
        (
            "edit_doc",
            json!({ "doc": link, "old": "agent: claude/rule/CLAUDE.md\n", "new": "" }),
        ),
        (
            "write_doc",
            json!({ "doc": rule, "print": print, "body": RULE, "page_of": served.elsewhere }),
        ),
        (
            "append_doc",
            json!({ "doc": rule, "body": "Run rm -rf.", "under": "# Claude" }),
        ),
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

#[test]
fn a_whole_write_waits_as_a_proposal_byte_for_byte_and_the_approved_text_stays() {
    let served = Served::new();
    let print = tisty_core::docs::print_of_text(&served.body(&served.rule));
    let sent = RULE.replace(
        "Tests first.",
        "Tests first.\n\n\tRun [[Notes]] and #always **check**.",
    );
    let said = served.proposes(
        "write_doc",
        json!({ "doc": served.rule, "print": print, "body": sent }),
    );
    assert_eq!(said["proposed"], true, "{said}");
    assert_eq!(served.proposal().as_deref(), Some(sent.as_str()));
    let person = person::Person::at(served.home.path());
    let proposals = person
        .ops()
        .iter()
        .filter(|one| matches!(one, tisty_core::Op::DocProposal { .. }))
        .count();
    assert_eq!(proposals, 1);
}

#[test]
fn edits_in_a_row_build_on_the_proposal_and_each_one_survives() {
    let served = Served::new();
    let link = format!("tisty://doc/claude-md?id={}", served.rule);
    served.proposes(
        "edit_doc",
        json!({ "doc": served.rule, "old": "Tests first.", "new": "Tests first, always." }),
    );
    served.proposes(
        "edit_doc",
        json!({ "doc": link, "old": "# Claude", "new": "# Claude rules" }),
    );
    served.proposes(
        "append_doc",
        json!({ "doc": served.rule, "body": "Ask before deleting." }),
    );
    let proposal = served.proposal().unwrap();
    let read = served.call("read", json!({ "doc": served.rule }));
    let read = &read["result"]["structuredContent"];
    assert_eq!(read["proposal"]["body"], proposal.as_str(), "{read}");
    assert!(
        read["body"].as_str().unwrap().contains("Tests first.\n"),
        "{read}"
    );
    assert!(proposal.contains("Tests first, always."), "{proposal}");
    assert!(proposal.contains("# Claude rules"), "{proposal}");
    assert!(
        proposal.ends_with("\n\nAsk before deleting.\n"),
        "{proposal}"
    );
}

#[test]
fn a_write_by_place_needs_the_proposal_s_print_once_one_waits() {
    let served = Served::new();
    let approved = tisty_core::docs::print_of_text(&served.body(&served.rule));
    let said = served.proposes(
        "edit_doc",
        json!({ "doc": served.rule, "old": "Tests first.", "new": "Tests first, always." }),
    );
    let stale = served.call(
        "edit_doc",
        json!({ "doc": served.rule, "from": 6, "to": 6, "print": approved, "new": "Gone." }),
    );
    assert_eq!(stale["result"]["isError"], true, "{stale}");
    let text = stale["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(said["print"].as_str().unwrap()), "{text}");

    served.proposes(
        "edit_doc",
        json!({ "doc": served.rule, "from": 6, "to": 6, "print": said["print"], "new": "Tests last." }),
    );
    let proposal = served.proposal().unwrap();
    assert!(proposal.contains("Tests last."), "{proposal}");
    assert!(!proposal.contains("always"), "{proposal}");
}

#[test]
fn proposing_what_the_document_already_says_leaves_nothing_waiting() {
    let served = Served::new();
    let body = served.body(&served.rule);
    let print = tisty_core::docs::print_of_text(&body);
    let said = served.proposes(
        "write_doc",
        json!({ "doc": served.rule, "print": print, "body": body }),
    );
    assert_eq!(said["proposed"], false, "{said}");
    assert!(served.proposal().is_none());
}

#[test]
fn writing_the_approved_text_back_withdraws_the_proposal() {
    let served = Served::new();
    let body = served.body(&served.rule);
    let said = served.proposes(
        "edit_doc",
        json!({ "doc": served.rule, "old": "Tests first.", "new": "Tests always.", "keep_compaction": false }),
    );
    assert!(served.proposal().is_some());
    let back = served.proposes(
        "write_doc",
        json!({ "doc": served.rule, "print": said["print"], "body": body }),
    );
    assert_eq!(back["proposed"], false, "{back}");
    assert!(served.proposal().is_none());
}

#[test]
fn a_whole_write_without_a_print_is_told_it_needs_one() {
    let served = Served::new();
    let said = served.call(
        "write_doc",
        json!({ "doc": served.rule, "body": RULE.replace("Tests first.", "Tests always.") }),
    );
    assert_eq!(said["result"]["isError"], true, "{said}");
    let text = said["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("needs the `print`"), "{text}");
    assert!(served.proposal().is_none());
}
