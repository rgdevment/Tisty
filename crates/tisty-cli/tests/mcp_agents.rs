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
    root: ulid::Ulid,
    claude: ulid::Ulid,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        let (root, claude) = person.keeps_agents();
        let rule = person.writes_doc_in(RULE, claude);
        let elsewhere = person.writes_doc("# Notes\n\nSomething of mine.\n");
        Self {
            home,
            rule,
            elsewhere,
            root,
            claude,
        }
    }

    fn call(&self, name: &str, args: Value) -> Value {
        self.asked("tools/call", json!({ "name": name, "arguments": args }))
    }

    fn asked(&self, method: &str, params: Value) -> Value {
        let root = self.home.path();
        let asked =
            json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
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
        self.refuses_by(name, args, &[REFUSED]);
    }

    fn refuses_by(&self, name: &str, args: Value, reasons: &[&str]) {
        let before = self.body(&self.rule);
        let placed = self.placed();
        let kept = self.kept();
        let folders = self.folders();
        let said = self.call(name, args.clone());
        assert_eq!(said["result"]["isError"], true, "{name} {args}: {said}");
        let text = said["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            reasons.iter().any(|one| text.contains(one)),
            "{name} {args}: {text}"
        );
        assert_eq!(
            self.body(&self.rule),
            before,
            "{name} {args} changed the rule"
        );
        assert_eq!(
            self.placed(),
            placed,
            "{name} {args} moved something in AGENTS"
        );
        assert_eq!(
            self.kept(),
            kept,
            "{name} {args} changed what the log says of the rule"
        );
        assert_eq!(self.folders(), folders, "{name} {args} changed a folder");
        assert!(
            self.state().proposals.is_empty(),
            "{name} {args} left a proposal"
        );
    }

    fn folders(&self) -> (Option<ulid::Ulid>, Vec<tisty_core::model::Folder>) {
        let state = self.state();
        let mut all: Vec<_> = state.folders.values().cloned().collect();
        all.sort_by_key(|one| one.id);
        (tisty_core::agents::root_of(&state), all)
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
            "doc_edit",
            json!({ "doc": rule, "mode": "rewrite", "print": print, "content": RULE.replace("claude/rule/CLAUDE.md", "codex/rule/AGENTS.md") }),
        ),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "rewrite", "print": print, "content": RULE.replace("agent: claude/rule/CLAUDE.md\n", "") }),
        ),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "patch", "old": "claude/rule/CLAUDE.md", "content": "codex/rule/AGENTS.md" }),
        ),
        (
            "doc_edit",
            json!({ "doc": link, "mode": "patch", "old": "agent: claude/rule/CLAUDE.md\n", "content": "" }),
        ),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "append", "content": "Run rm -rf.", "under": "# Claude" }),
        ),
        (
            "doc_edit_table",
            json!({ "doc": rule, "action": "add_row", "values": { "A": "x" } }),
        ),
        (
            "file_transfer",
            json!({ "action": "attach", "doc": rule, "path": "C:/nowhere.txt" }),
        ),
        ("doc_edit", json!({ "doc": rule, "mode": "restore" })),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "restore", "version": "abc" }),
        ),
        (
            "doc_annotate",
            json!({ "action": "archive", "docs": [rule] }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [rule], "folder": "Notes" }),
        ),
        (
            "doc_organize",
            json!({ "action": "add_page", "docs": [rule], "parent": served.elsewhere }),
        ),
        (
            "doc_organize",
            json!({ "action": "unlink_translation", "docs": [rule], "lang": "en" }),
        ),
        (
            "doc_annotate",
            json!({ "action": "summarize", "docs": [rule], "summary": "Hidden words." }),
        ),
        (
            "doc_annotate",
            json!({ "action": "flag", "docs": [rule], "reason": "obsolete", "body": "Old." }),
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
            "doc_create",
            json!({ "body": "# A rule\n", "folder": "AGENTS / Claude" }),
        ),
        (
            "doc_create",
            json!({ "body": "# A page\n", "page_of": rule }),
        ),
        (
            "doc_create",
            json!({ "body": "# In English\n", "translation_of": rule }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [elsewhere], "folder": "AGENTS / Claude" }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [elsewhere], "folder": "AGENTS" }),
        ),
        (
            "doc_organize",
            json!({ "action": "add_page", "docs": [elsewhere], "parent": rule }),
        ),
        (
            "doc_organize",
            json!({ "action": "link_translation", "docs": [elsewhere], "parent": rule }),
        ),
        (
            "file_transfer",
            json!({ "action": "import_doc", "path": "C:/nowhere.md", "folder": "AGENTS / Claude" }),
        ),
        (
            "doc_annotate",
            json!({ "action": "archive", "docs": [elsewhere], "folder": "AGENTS" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "AGENTS / Codex" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "Codex", "folder": "AGENTS" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "AGENTS", "icon": "folder" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "agents", "icon": "robot" }),
        ),
    ];
    for (name, args) in cases {
        served.refuses(name, args);
    }
}

#[test]
fn a_body_with_an_agent_line_is_never_written_anywhere() {
    let served = Served::new();
    let made = served.call("doc_create", json!({ "body": RULE }));
    assert_eq!(made["result"]["isError"], true, "{made}");
    let text = made["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("`agent:` line"), "{text}");

    let over = served.call(
        "doc_edit",
        json!({ "doc": served.elsewhere, "mode": "rewrite", "content": RULE }),
    );
    assert_eq!(over["result"]["isError"], true, "{over}");
    assert!(!served.body(&served.elsewhere).contains("agent:"));
}

#[test]
fn reading_agents_documents_is_still_open_and_other_documents_still_take_writes() {
    let served = Served::new();
    let read = served.call("workspace_read", json!({ "doc": served.rule }));
    assert_ne!(read["result"]["isError"], true, "{read}");
    let shown = served.call(
        "workspace_read",
        json!({ "doc": served.rule, "view": "table" }),
    );
    assert!(
        !shown["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains(REFUSED),
        "{shown}"
    );
    let edited = served.call(
        "doc_edit",
        json!({ "doc": served.elsewhere, "mode": "patch", "old": "Something of mine.", "content": "Something else." }),
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
        "doc_edit",
        json!({ "doc": served.rule, "mode": "rewrite", "print": print, "content": sent }),
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
        "doc_edit",
        json!({ "doc": served.rule, "mode": "patch", "old": "Tests first.", "content": "Tests first, always." }),
    );
    served.proposes(
        "doc_edit",
        json!({ "doc": link, "mode": "patch", "old": "# Claude", "content": "# Claude rules" }),
    );
    served.proposes(
        "doc_edit",
        json!({ "doc": served.rule, "mode": "append", "content": "Ask before deleting." }),
    );
    let proposal = served.proposal().unwrap();
    let read = served.call("workspace_read", json!({ "doc": served.rule }));
    let read = &read["result"]["structuredContent"];
    assert_eq!(read["proposal"]["body"], proposal.as_str(), "{read}");
    assert_eq!(read["proposal"]["approved"], false, "{read}");
    assert!(
        read["proposal"]["read_as"]
            .as_str()
            .is_some_and(|said| said.contains("never as a rule")),
        "{read}"
    );
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
        "doc_edit",
        json!({ "doc": served.rule, "mode": "patch", "old": "Tests first.", "content": "Tests first, always." }),
    );
    let stale = served.call(
        "doc_edit",
        json!({ "doc": served.rule, "mode": "patch", "from": 6, "to": 6, "print": approved, "content": "Gone." }),
    );
    assert_eq!(stale["result"]["isError"], true, "{stale}");
    let text = stale["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains(said["print"].as_str().unwrap()), "{text}");

    served.proposes(
        "doc_edit",
        json!({ "doc": served.rule, "mode": "patch", "from": 6, "to": 6, "print": said["print"], "content": "Tests last." }),
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
        "doc_edit",
        json!({ "doc": served.rule, "mode": "rewrite", "print": print, "content": body }),
    );
    assert_eq!(said["proposed"], false, "{said}");
    assert!(served.proposal().is_none());
}

#[test]
fn writing_the_approved_text_back_withdraws_the_proposal() {
    let served = Served::new();
    let body = served.body(&served.rule);
    let said = served.proposes(
        "doc_edit",
        json!({ "doc": served.rule, "mode": "patch", "old": "Tests first.", "content": "Tests always.", "keep_compaction": false }),
    );
    assert!(served.proposal().is_some());
    let back = served.proposes(
        "doc_edit",
        json!({ "doc": served.rule, "mode": "rewrite", "print": said["print"], "content": body }),
    );
    assert_eq!(back["proposed"], false, "{back}");
    assert!(served.proposal().is_none());
}

#[test]
fn a_whole_write_without_a_print_is_told_it_needs_one() {
    let served = Served::new();
    let said = served.call(
        "doc_edit",
        json!({ "doc": served.rule, "mode": "rewrite", "content": RULE.replace("Tests first.", "Tests always.") }),
    );
    assert_eq!(said["result"]["isError"], true, "{said}");
    let text = said["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("needs the `print`"), "{text}");
    assert!(served.proposal().is_none());
}

#[test]
fn a_document_named_with_blanks_around_it_is_kept_like_its_bare_id() {
    let served = Served::new();
    let rule = served.rule.clone();
    let padded = [
        format!(" {rule}"),
        format!("{rule}\n"),
        format!("\t{rule} "),
    ];
    for padded in &padded {
        served.refuses("doc_edit", json!({ "doc": padded, "mode": "restore" }));
        served.refuses(
            "doc_annotate",
            json!({ "action": "archive", "docs": [padded] }),
        );
        served.refuses(
            "doc_organize",
            json!({ "action": "file", "docs": [padded], "folder": "AGENTS" }),
        );
        served.refuses(
            "doc_annotate",
            json!({ "action": "summarize", "docs": [padded], "summary": "Hidden words." }),
        );
        served.refuses(
            "doc_annotate",
            json!({ "action": "flag", "docs": [padded], "reason": "obsolete", "body": "Old." }),
        );
    }
    served.proposes(
        "doc_edit",
        json!({ "doc": padded[0], "mode": "patch", "old": "Tests first.", "content": "Tests never." }),
    );
    served.proposes(
        "doc_edit",
        json!({ "doc": padded[1], "mode": "append", "content": "One." }),
    );
    served.proposes(
        "doc_edit",
        json!({ "doc": padded[2], "mode": "append", "content": "Two." }),
    );
    let proposal = served.proposal().unwrap();
    assert!(proposal.contains("Tests never."), "{proposal}");
    assert!(proposal.ends_with("\n\nOne.\n\nTwo.\n"), "{proposal}");
}

#[test]
fn a_folder_named_with_hidden_marks_is_kept_like_agents() {
    let served = Served::new();
    let person = person::Person::at(served.home.path());
    person.makes_folder("Claude", None, None);
    let claude = served.claude.to_string();
    let cases: Vec<(&str, Value)> = vec![
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "AGEN\tTS", "icon": "home" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "AGENTS\u{2066}", "icon": "home" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "\u{202e}AGENTS", "icon": "home" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "ÁGENTS", "icon": "home" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "Claude", "icon": "home" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "Claude", "folder": " ", "icon": "home" }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "Codex", "folder": format!(" {claude}") }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [served.elsewhere.clone()], "folder": format!(" {claude}") }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [served.elsewhere.clone()], "folder": format!("{}\n", served.root) }),
        ),
    ];
    for (name, args) in cases {
        served.refuses(name, args);
    }
}

const NAMES_A_DOCUMENT: [&str; 10] = [
    "doc",
    "docs",
    "page_of",
    "parent",
    "translation_of",
    "of",
    "after",
    "before",
    "order",
    "instead",
];
const NAMES_A_FOLDER: [&str; 2] = ["folder", "inside"];
const PROPOSES: [&str; 1] = ["doc_edit"];
// A task, step, note, path, heading, cell or setting: never a document or a folder.
const NAMES_NEITHER: [&str; 64] = [
    "action",
    "add_reminders",
    "branch",
    "content",
    "mode",
    "outcome",
    "position",
    "pr",
    "for",
    "to_person",
    "again",
    "anyway",
    "archived",
    "at",
    "body",
    "color",
    "column",
    "compact",
    "date",
    "deadline",
    "description",
    "do",
    "down",
    "echo",
    "even_if_more",
    "from",
    "icon",
    "keep_compaction",
    "label",
    "lang",
    "list",
    "new",
    "note",
    "notes",
    "old",
    "part_of",
    "parts",
    "path",
    "person_asked",
    "print",
    "priority",
    "reason",
    "remind",
    "replace",
    "replaces",
    "row",
    "section",
    "source",
    "state",
    "step",
    "steps",
    "summary",
    "table",
    "tags",
    "task",
    "tasks",
    "text",
    "title",
    "to",
    "type",
    "under",
    "value",
    "values",
    "version",
];

impl Served {
    fn tools_that_write(&self) -> Vec<(String, Vec<String>, Taken)> {
        let listed = self.asked("tools/list", json!({}));
        listed["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|one| one["annotations"]["readOnlyHint"] != true)
            .map(|one| {
                let taken = one["inputSchema"]["properties"].as_object().unwrap();
                let shapes = taken
                    .iter()
                    .map(|(key, shape)| (key.clone(), kinds(shape)))
                    .collect();
                let name = one["name"].as_str().unwrap().to_string();
                (name, taken.keys().cloned().collect(), shapes)
            })
            .collect()
    }

    // Refused at the door, or left to the proposal path; either way nothing approved changes.
    fn keeps_agents(&self, name: &str, args: Value, may_propose: bool) {
        let before = self.body(&self.rule);
        let placed = self.placed();
        let kept = self.kept();
        let folders = self.folders();
        let said = self.call(name, args.clone());
        let text = said["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        let refused = said["result"]["isError"] == true && text.contains(REFUSED);
        assert!(refused || may_propose, "{name} {args}: {said}");
        assert_eq!(
            self.body(&self.rule),
            before,
            "{name} {args} changed the rule"
        );
        assert_eq!(
            self.placed(),
            placed,
            "{name} {args} moved something in AGENTS"
        );
        assert_eq!(
            self.kept(),
            kept,
            "{name} {args} changed what the log says of the rule"
        );
        assert_eq!(self.folders(), folders, "{name} {args} changed a folder");
    }
}

type Taken = std::collections::BTreeMap<String, Vec<String>>;

fn kinds(shape: &Value) -> Vec<String> {
    match (&shape["type"], shape.get("anyOf").or(shape.get("oneOf"))) {
        (Value::String(one), _) => vec![one.clone()],
        (Value::Array(many), _) => many
            .iter()
            .filter_map(|one| one.as_str().map(String::from))
            .collect(),
        (_, Some(Value::Array(many))) => many.iter().flat_map(kinds).collect(),
        _ => Vec::new(),
    }
}

fn documents_as_an_agent_could_write_them(rule: &str) -> Vec<(String, bool)> {
    let mut all: Vec<(String, bool)> = [
        format!(" {rule}"),
        format!("{rule}\n"),
        format!("\t{rule} "),
        format!("tisty://doc/claude-md?id={rule}"),
        format!(" tisty://doc/notes?id={rule}\n"),
    ]
    .into_iter()
    .map(|one| (one, true))
    .collect();
    all.extend(
        [
            format!("\u{2066}{rule}"),
            format!("{rule}\u{200b}"),
            format!("\u{202e}{rule}"),
            rule.to_uppercase(),
            format!("tisty://doc/claude-md?id=\u{2066}{rule}"),
        ]
        .into_iter()
        .map(|one| (one, false)),
    );
    all
}

fn folders_as_an_agent_could_write_them(served: &Served) -> Vec<String> {
    vec![
        " AGENTS".into(),
        "AGENTS\n".into(),
        "agents".into(),
        "\tAgents / CLAUDE ".into(),
        "AGENTS\u{2066}".into(),
        "AGEN\tTS".into(),
        "\u{202e}AGENTS".into(),
        "ÁGENTS / Claude".into(),
        "AGE\u{200b}NTS".into(),
        format!(" {}", served.root),
        format!("{}\n", served.claude),
        served.claude.to_string().to_lowercase(),
    ]
}

fn with(keys: &[String], key: &str, said: Value, elsewhere: &str) -> Value {
    let takes = |one: &str| one != key && keys.iter().any(|k| k == one);
    let mut args = serde_json::Map::new();
    args.insert(key.to_string(), said);
    let around = [
        ("doc", json!(elsewhere)),
        ("docs", json!([elsewhere])),
        ("action", json!("attach")),
        ("mode", json!("patch")),
        ("body", json!("# Placed")),
        ("content", json!("Tests always.")),
        ("path", json!("C:/nowhere.md")),
        ("name", json!("Codex")),
        ("old", json!("Tests first.")),
    ];
    for (one, value) in around {
        if takes(one) && (!["doc", "docs"].contains(&one) || NAMES_A_FOLDER.contains(&key)) {
            args.insert(one.to_string(), value);
        }
    }
    Value::Object(args)
}

#[test]
fn every_tool_that_writes_keeps_agents_however_a_name_is_wrapped() {
    let served = Served::new();
    let tools = served.tools_that_write();
    assert_eq!(tools.len(), 11, "{tools:?}");
    for (name, keys, _) in &tools {
        for key in keys {
            let placed = NAMES_A_DOCUMENT.contains(&key.as_str())
                || NAMES_A_FOLDER.contains(&key.as_str())
                || (name == "doc_organize" && key == "name")
                || NAMES_NEITHER.contains(&key.as_str());
            assert!(
                placed,
                "{name} takes `{key}`: say whether it names a document, a folder or neither"
            );
        }
    }

    for (name, keys, shapes) in &tools {
        for key in keys
            .iter()
            .filter(|key| NAMES_A_DOCUMENT.contains(&key.as_str()))
        {
            assert!(
                !shapes[key].is_empty(),
                "{name} `{key}` has no type to send"
            );
            let takes = |kind: &str| shapes[key].iter().any(|one| one == kind);
            for (said, exactly) in documents_as_an_agent_could_write_them(&served.rule) {
                let may_propose = exactly && key == "doc" && PROPOSES.contains(&name.as_str());
                if takes("string") {
                    let args = with(keys, key, json!(said), &served.elsewhere);
                    served.keeps_agents(name, args, may_propose);
                }
                if takes("array") {
                    let args = with(keys, key, json!([said]), &served.elsewhere);
                    served.keeps_agents(name, args, false);
                }
            }
        }
        for key in keys
            .iter()
            .filter(|key| NAMES_A_FOLDER.contains(&key.as_str()))
        {
            for said in folders_as_an_agent_could_write_them(&served) {
                let args = with(keys, key, json!(said), &served.elsewhere);
                served.keeps_agents(name, args, false);
            }
        }
    }
    for said in folders_as_an_agent_could_write_them(&served) {
        served.keeps_agents(
            "doc_organize",
            json!({ "action": "create_folder", "name": said, "icon": "home" }),
            false,
        );
        served.keeps_agents(
            "doc_organize",
            json!({ "action": "create_folder", "name": format!("{said} / Codex") }),
            false,
        );
    }
    served.keeps_agents(
        "doc_organize",
        json!({ "action": "create_folder", "name": "Claude", "icon": "home" }),
        false,
    );
}

#[test]
fn a_folder_outside_agents_still_takes_a_look_and_a_new_folder() {
    let served = Served::new();
    let person = person::Person::at(served.home.path());
    let notes = person.makes_folder("Notes", None, None);
    let said = served.call(
        "doc_organize",
        json!({ "action": "create_folder", "name": " Notes ", "icon": "home" }),
    );
    assert_ne!(said["result"]["isError"], true, "{said}");
    assert_eq!(served.state().folders[&notes].icon.as_deref(), Some("home"));
    let made = served.call(
        "doc_organize",
        json!({ "action": "create_folder", "name": "Claude", "folder": "Notes" }),
    );
    assert_ne!(made["result"]["isError"], true, "{made}");
    assert_eq!(made["result"]["structuredContent"]["made"], true, "{made}");
    let filed = served.call(
        "doc_organize",
        json!({ "action": "file", "docs": [format!(" {}\n", served.elsewhere)], "folder": "\tNotes / Claude " }),
    );
    assert_ne!(filed["result"]["isError"], true, "{filed}");

    let mine = person.makes_folder("Claude", None, None);
    let filed = served.call(
        "doc_organize",
        json!({ "action": "file", "docs": [served.elsewhere], "folder": "Claude" }),
    );
    assert_ne!(filed["result"]["isError"], true, "{filed}");
    let state = served.state();
    let elsewhere = state.docs.values().find(|one| one.file == served.elsewhere);
    assert_eq!(elsewhere.unwrap().folder, Some(mine));
}

// Every way the root could be taken off the top, renamed, re-iconed, archived, deleted or written over.
fn unhooking(served: &Served, outside: ulid::Ulid) -> Vec<(&'static str, tisty_core::Op)> {
    use tisty_core::Op;
    use tisty_core::event::{FolderAdd, Look, Name};
    let root = served.root;
    let under = |folder| tisty_core::event::Filed {
        folder: Some(Some(folder)),
        page_of: None,
        order: None,
    };
    vec![
        (
            "moved into another folder",
            Op::FolderMove {
                id: root,
                d: under(outside),
            },
        ),
        (
            "renamed",
            Op::FolderRename {
                id: root,
                d: Name {
                    name: "Notes".into(),
                },
            },
        ),
        (
            "re-iconed",
            Op::FolderLook {
                id: root,
                d: Look {
                    icon: Some(Some("home".into())),
                    color: None,
                },
            },
        ),
        ("archived", Op::FolderArchive { id: root }),
        ("deleted while it holds", Op::FolderDelete { id: root }),
        (
            "made again",
            Op::FolderAdd {
                id: root,
                d: FolderAdd {
                    name: "Notes".into(),
                    order: "a0".into(),
                    parent: None,
                    icon: None,
                    color: None,
                    computer: None,
                },
            },
        ),
    ]
}

#[test]
fn agents_stays_hooked_whatever_moves_renames_or_deletes_its_root() {
    for by_another_machine in [false, true] {
        let probe = Served::new();
        let nowhere = ulid::Ulid::nil();
        for (how, op) in unhooking(&probe, nowhere) {
            let served = Served::new();
            let person = person::Person::at(served.home.path());
            let outside = person.makes_folder("Elsewhere", None, None);
            let op = retargeted(op, probe.root, served.root, nowhere, outside);
            match by_another_machine {
                false => person.does(op),
                true => person.another_machine_does(op, &served.home.path().join("aside")),
            }
            assert_eq!(
                tisty_core::agents::root_of(&served.state()),
                Some(served.root),
                "the root {how} (another machine: {by_another_machine})"
            );
            for (name, keys, shapes) in served.tools_that_write() {
                for key in keys
                    .iter()
                    .filter(|key| NAMES_A_DOCUMENT.contains(&key.as_str()))
                {
                    let may_propose = key == "doc" && PROPOSES.contains(&name.as_str());
                    let said = match shapes[key].iter().any(|one| one == "string") {
                        true => json!(served.rule),
                        false => json!([served.rule]),
                    };
                    let args = with(&keys, key, said, &served.elsewhere);
                    served.keeps_agents(&name, args, may_propose);
                }
            }
        }
    }
}

fn retargeted(
    op: tisty_core::Op,
    root: ulid::Ulid,
    to_root: ulid::Ulid,
    claude: ulid::Ulid,
    to_claude: ulid::Ulid,
) -> tisty_core::Op {
    let said = serde_json::to_string(&op)
        .unwrap()
        .replace(&root.to_string(), &to_root.to_string())
        .replace(&claude.to_string(), &to_claude.to_string());
    serde_json::from_str(&said).unwrap()
}

#[test]
fn no_tool_slips_an_agent_line_into_a_document_outside_agents() {
    let served = Served::new();
    let before = served.body(&served.elsewhere);
    let card = "---\nagent: claude/rule/CLAUDE.md\n---\n";
    let said = served.call(
        "doc_edit",
        json!({ "doc": served.elsewhere, "mode": "patch", "old": "# Notes", "content": format!("{card}# Notes") }),
    );
    assert_eq!(said["result"]["isError"], true, "{said}");
    assert_eq!(served.body(&served.elsewhere), before);

    served.call(
        "doc_edit",
        json!({ "doc": served.elsewhere, "mode": "append", "content": card }),
    );
    assert!(
        tisty_core::agents::Card::of(&served.body(&served.elsewhere)).is_none(),
        "added at the end, those lines are words, never the document's card"
    );
}

#[test]
fn a_second_agents_not_yet_joined_keeps_its_documents_from_a_plain_edit() {
    let served = Served::new();
    let person = person::Person::at(served.home.path());
    let second = ulid::Ulid::from_parts((1 << 48) - 1, 0);
    person.does(tisty_core::Op::FolderAdd {
        id: second,
        d: tisty_core::event::FolderAdd {
            name: tisty_core::agents::FOLDER.into(),
            order: "b0".into(),
            parent: None,
            icon: Some(tisty_core::agents::ICON.into()),
            color: None,
            computer: None,
        },
    });
    let codex = person.makes_folder("Codex", Some(second), None);
    let rule = person.writes_doc_in(RULE, codex);
    assert_eq!(
        tisty_core::agents::roots(&served.state()),
        vec![served.root, second]
    );
    let before = served.body(&rule);
    let said = served.call(
        "doc_edit",
        json!({ "doc": rule, "mode": "patch", "old": "Tests first.", "content": "Skip the tests." }),
    );
    assert_eq!(served.body(&rule), before, "{said}");
    let state = served.state();
    let kept = state.docs.values().find(|one| one.file == rule).unwrap();
    match said["result"]["isError"] == true {
        true => assert!(!state.proposals.contains_key(&kept.id), "{said}"),
        false => {
            let waiting = state.proposals.get(&kept.id).map(|one| one.body.as_str());
            assert_eq!(
                waiting,
                Some(RULE.replace("Tests first.", "Skip the tests.").as_str()),
                "{said}"
            );
        }
    }
    for name in ["Codex", "CO\u{200b}DEX ", "agents / codex"] {
        let looked = served.call(
            "doc_organize",
            json!({ "action": "create_folder", "name": name, "icon": "folder" }),
        );
        let text = looked["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains(REFUSED), "{name:?}: {looked}");
        assert_eq!(
            served.state().folders[&codex].icon,
            None,
            "{name:?}: {looked}"
        );
    }
}

#[test]
fn no_tool_deletes_a_document_of_agents_or_takes_a_file_from_an_agents_folder() {
    let served = Served::new();
    let listed = served.asked("tools/list", json!({}));
    let names: Vec<String> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|one| one["name"].as_str().unwrap().to_string())
        .collect();
    assert!(!names.is_empty());
    let reaching: Vec<&String> = names
        .iter()
        .filter(|name| {
            ["drop", "delete", "remove", "loose", "agents_"]
                .iter()
                .any(|word| name.contains(word))
        })
        .collect();
    assert!(reaching.is_empty(), "{reaching:?}");

    let rule = served.rule.clone();
    for window_only in ["agents_drop", "agents_reach", "agents_loose", "doc_drop"] {
        let said = served.call(
            window_only,
            json!({ "ids": [rule.clone()], "id": rule, "away": true }),
        );
        assert!(
            said["error"].is_object() || said["result"]["isError"] == true,
            "{window_only}: {said}"
        );
        assert!(
            served.state().docs.values().any(|one| one.file == rule),
            "{window_only} deleted it"
        );
    }
    served.refuses(
        "doc_annotate",
        json!({ "action": "archive", "docs": [rule] }),
    );
    served.refuses(
        "doc_annotate",
        json!({ "action": "flag", "docs": [rule], "reason": "obsolete", "body": "Old." }),
    );
}

fn marked_rule(computer: &str) -> String {
    RULE.replace(
        "claude/rule/CLAUDE.md",
        &format!("[claude/rule/CLAUDE.md, on-{computer}]"),
    )
}

fn text_of(said: &Value) -> &str {
    said["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
}

#[test]
fn no_edit_outside_agents_changes_adds_or_removes_the_computer_mark() {
    let served = Served::new();
    let person = person::Person::at(served.home.path());
    let (one, two) = ("dev_aaaa1111", "dev_bbbb2222");
    let plain = person.writes_doc(RULE);
    let marked = person.writes_doc(&marked_rule(one));
    let flagged = person.writes_doc(&RULE.replacen(
        "agent: claude/rule/CLAUDE.md",
        "agent: [claude/rule/CLAUDE.md, crlf]",
        1,
    ));
    let cases = [
        (&marked, format!("on-{one}"), format!("on-{two}")),
        (
            &plain,
            "agent: claude/rule/CLAUDE.md".to_string(),
            format!("agent: [claude/rule/CLAUDE.md, on-{one}]"),
        ),
        (
            &marked,
            format!("[claude/rule/CLAUDE.md, on-{one}]"),
            "[claude/rule/CLAUDE.md, crlf]".to_string(),
        ),
        (&flagged, "crlf]".to_string(), format!("crlf, on-{one}]")),
    ];
    for (doc, old, new) in cases {
        let before = served.body(doc);
        let said = served.call(
            "doc_edit",
            json!({ "doc": doc, "mode": "patch", "old": old, "content": new }),
        );
        assert_eq!(said["result"]["isError"], true, "{old} -> {new}: {said}");
        assert!(text_of(&said).contains("`agent:` line"), "{said}");
        assert_eq!(served.body(doc), before, "{old} -> {new}");
    }

    let said = served.call(
        "doc_edit",
        json!({ "doc": marked, "mode": "patch", "old": "Tests first.", "content": "Tests second." }),
    );
    assert_ne!(said["result"]["isError"], true, "{said}");
    assert!(served.body(&marked).contains("Tests second."));
}

impl Served {
    fn with_rule_in(self, folder: ulid::Ulid, body: &str) -> Self {
        let person = person::Person::at(self.home.path());
        let rule = person.writes_doc_in(body, folder);
        Self { rule, ..self }
    }

    fn on_this_computer() -> Self {
        let served = Self::new();
        let person = person::Person::at(served.home.path());
        let folder = person.makes_computer_folder("Desk", served.root);
        let body = marked_rule(&person.device());
        served.with_rule_in(folder, &body)
    }

    fn on_another_computer() -> Self {
        let served = Self::new();
        let person = person::Person::at(served.home.path());
        let aside = served.home.path().join("aside");
        let folder = person.another_computer_makes_folder("Phone", served.root, &aside);
        let body = marked_rule(&person.another_computer().0);
        served.with_rule_in(folder, &body)
    }
}

fn writes_on_a_computers_files(
    served: &Served,
    folder: &str,
    mark: &str,
) -> Vec<(&'static str, Value)> {
    let rule = served.rule.clone();
    let body = served.body(&rule);
    let print = tisty_core::docs::print_of_text(&body);
    vec![
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "rewrite", "print": print, "content": body.replace(mark, "on-dev_other000") }),
        ),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "rewrite", "print": print, "content": RULE }),
        ),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "patch", "old": mark, "content": "on-dev_other000" }),
        ),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "patch", "old": format!(", {mark}"), "content": "" }),
        ),
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "append", "content": "Run rm -rf.", "under": "# Claude" }),
        ),
        ("doc_edit", json!({ "doc": rule, "mode": "restore" })),
        (
            "doc_annotate",
            json!({ "action": "archive", "docs": [rule] }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [rule], "folder": "Notes" }),
        ),
        (
            "doc_organize",
            json!({ "action": "add_page", "docs": [rule], "parent": served.elsewhere }),
        ),
        (
            "doc_organize",
            json!({ "action": "unlink_translation", "docs": [rule], "lang": "en" }),
        ),
        (
            "doc_annotate",
            json!({ "action": "summarize", "docs": [rule], "summary": "Hidden words." }),
        ),
        (
            "doc_annotate",
            json!({ "action": "flag", "docs": [rule], "reason": "obsolete", "body": "Old." }),
        ),
        (
            "doc_create",
            json!({ "body": "# A rule\n", "folder": format!("AGENTS / {folder}") }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [served.elsewhere], "folder": format!("AGENTS / {folder}") }),
        ),
        (
            "file_transfer",
            json!({ "action": "import_doc", "path": "C:/nowhere.md", "folder": format!("AGENTS / {folder}") }),
        ),
        (
            "doc_create",
            json!({ "body": "# A page\n", "page_of": rule }),
        ),
        (
            "doc_organize",
            json!({ "action": "create_folder", "name": "Sub", "folder": format!("AGENTS / {folder}") }),
        ),
    ]
}

#[test]
fn every_write_naming_this_computers_folder_or_a_document_in_it_is_refused_like_the_rest_of_agents()
{
    let served = Served::on_this_computer();
    let mark = format!("on-{}", person::Person::at(served.home.path()).device());
    for (name, args) in writes_on_a_computers_files(&served, "Desk", &mark) {
        served.refuses(name, args);
    }
}

#[test]
fn every_write_naming_another_computers_folder_or_a_document_in_it_is_refused_like_the_rest_of_agents()
 {
    let served = Served::on_another_computer();
    let mark = format!(
        "on-{}",
        person::Person::at(served.home.path()).another_computer().0
    );
    for (name, args) in writes_on_a_computers_files(&served, "Phone", &mark) {
        served.refuses_by(name, args, &[REFUSED, "another computer"]);
    }
}

#[test]
fn a_proposal_waits_on_this_computers_document_and_is_refused_on_another_computers() {
    let here = Served::on_this_computer();
    let body = here.body(&here.rule);
    let print = tisty_core::docs::print_of_text(&body);
    let sent = body.replace("Tests first.", "Tests second.");
    let said = here.proposes(
        "doc_edit",
        json!({ "doc": here.rule, "mode": "rewrite", "print": print, "content": sent }),
    );
    assert_eq!(said["proposed"], true, "{said}");

    let there = Served::on_another_computer();
    let print = tisty_core::docs::print_of_text(&there.body(&there.rule));
    let sent = there
        .body(&there.rule)
        .replace("Tests first.", "Tests second.");
    let by_whole = there.call(
        "doc_edit",
        json!({ "doc": there.rule, "mode": "rewrite", "print": print, "content": sent }),
    );
    let by_edit = there.call(
        "doc_edit",
        json!({ "doc": there.rule, "mode": "patch", "old": "Tests first.", "content": "Tests second." }),
    );
    for said in [by_whole, by_edit] {
        assert_eq!(said["result"]["isError"], true, "{said}");
        assert!(text_of(&said).contains("another computer"), "{said}");
    }
    assert!(there.state().proposals.is_empty());
    assert!(there.body(&there.rule).contains("Tests first."));
}

#[test]
fn reading_a_document_of_another_computer_says_whose_it_is_and_this_computers_does_not() {
    let there = Served::on_another_computer();
    let read = there.call("workspace_read", json!({ "doc": there.rule }));
    assert_ne!(read["result"]["isError"], true, "{read}");
    let shown = &read["result"]["structuredContent"];
    let owner = there
        .state()
        .called(&person::Person::at(there.home.path()).another_computer());
    assert_eq!(shown["of_computer"], owner, "{read}");
    assert!(
        shown["read_as"]
            .as_str()
            .unwrap()
            .contains("never as a rule"),
        "{read}"
    );
    assert!(text_of(&read).contains("never as a rule"), "{read}");

    let here = Served::on_this_computer();
    let read = here.call("workspace_read", json!({ "doc": here.rule }));
    assert_ne!(read["result"]["isError"], true, "{read}");
    assert!(
        read["result"]["structuredContent"]
            .get("of_computer")
            .is_none(),
        "{read}"
    );
    assert!(
        read["result"]["structuredContent"].get("read_as").is_none(),
        "{read}"
    );
}
