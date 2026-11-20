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
        let before = self.body(&self.rule);
        let placed = self.placed();
        let kept = self.kept();
        let folders = self.folders();
        let said = self.call(name, args.clone());
        assert_eq!(said["result"]["isError"], true, "{name} {args}: {said}");
        let text = said["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains(REFUSED), "{name} {args}: {text}");
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
        served.refuses("restore_doc", json!({ "doc": padded }));
        served.refuses("archive_doc", json!({ "doc": padded }));
        served.refuses("file_doc", json!({ "doc": padded, "folder": "AGENTS" }));
        served.refuses(
            "sum_up",
            json!({ "doc": padded, "summary": "Hidden words." }),
        );
        served.refuses(
            "flag_doc",
            json!({ "doc": padded, "reason": "obsolete", "body": "Old." }),
        );
    }
    served.proposes(
        "edit_doc",
        json!({ "doc": padded[0], "old": "Tests first.", "new": "Tests never." }),
    );
    served.proposes("append_doc", json!({ "doc": padded[1], "body": "One." }));
    served.proposes("append_doc", json!({ "doc": padded[2], "body": "Two." }));
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
        ("folder", json!({ "name": "AGEN\tTS", "icon": "home" })),
        (
            "folder",
            json!({ "name": "AGENTS\u{2066}", "icon": "home" }),
        ),
        (
            "folder",
            json!({ "name": "\u{202e}AGENTS", "icon": "home" }),
        ),
        ("folder", json!({ "name": "ÁGENTS", "icon": "home" })),
        ("folder", json!({ "name": "Claude", "icon": "home" })),
        (
            "folder",
            json!({ "name": "Claude", "inside": " ", "icon": "home" }),
        ),
        (
            "folder",
            json!({ "name": "Codex", "inside": format!(" {claude}") }),
        ),
        (
            "file_doc",
            json!({ "doc": served.elsewhere, "folder": format!(" {claude}") }),
        ),
        (
            "file_doc",
            json!({ "doc": served.elsewhere, "folder": format!("{}\n", served.root) }),
        ),
    ];
    for (name, args) in cases {
        served.refuses(name, args);
    }
}

const NAMES_A_DOCUMENT: [&str; 8] = [
    "doc",
    "page_of",
    "translation_of",
    "of",
    "after",
    "before",
    "order",
    "instead",
];
const NAMES_A_FOLDER: [&str; 2] = ["folder", "inside"];
const PROPOSES: [&str; 3] = ["write_doc", "edit_doc", "append_doc"];
// A task, step, note, path, heading, cell or setting: never a document or a folder.
const NAMES_NEITHER: [&str; 54] = [
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
            // It reads the store and writes only a copy outside it, so the door lets it through.
            .filter(|one| one["name"] != "export_doc")
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
        ("body", json!("# Placed")),
        ("path", json!("C:/nowhere.md")),
        ("name", json!("Codex")),
        ("do", json!("add")),
        ("old", json!("Tests first.")),
        ("new", json!("Tests always.")),
    ];
    for (one, value) in around {
        if takes(one) && (one != "doc" || NAMES_A_FOLDER.contains(&key)) {
            args.insert(one.to_string(), value);
        }
    }
    Value::Object(args)
}

#[test]
fn every_tool_that_writes_keeps_agents_however_a_name_is_wrapped() {
    let served = Served::new();
    let tools = served.tools_that_write();
    assert!(tools.len() > 20, "{tools:?}");
    for (name, keys, _) in &tools {
        for key in keys {
            let placed = NAMES_A_DOCUMENT.contains(&key.as_str())
                || NAMES_A_FOLDER.contains(&key.as_str())
                || (name == "folder" && key == "name")
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
        served.keeps_agents("folder", json!({ "name": said, "icon": "home" }), false);
        served.keeps_agents(
            "folder",
            json!({ "name": format!("{said} / Codex") }),
            false,
        );
    }
    served.keeps_agents("folder", json!({ "name": "Claude", "icon": "home" }), false);
}

#[test]
fn a_folder_outside_agents_still_takes_a_look_and_a_new_folder() {
    let served = Served::new();
    let person = person::Person::at(served.home.path());
    let notes = person.makes_folder("Notes", None, None);
    let said = served.call("folder", json!({ "name": " Notes ", "icon": "home" }));
    assert_ne!(said["result"]["isError"], true, "{said}");
    assert_eq!(served.state().folders[&notes].icon.as_deref(), Some("home"));
    let made = served.call("folder", json!({ "name": "Claude", "inside": "Notes" }));
    assert_ne!(made["result"]["isError"], true, "{made}");
    assert_eq!(made["result"]["structuredContent"]["made"], true, "{made}");
    let filed = served.call(
        "file_doc",
        json!({ "doc": format!(" {}\n", served.elsewhere), "folder": "\tNotes / Claude " }),
    );
    assert_ne!(filed["result"]["isError"], true, "{filed}");

    let mine = person.makes_folder("Claude", None, None);
    let filed = served.call(
        "file_doc",
        json!({ "doc": served.elsewhere, "folder": "Claude" }),
    );
    assert_ne!(filed["result"]["isError"], true, "{filed}");
    let state = served.state();
    let elsewhere = state.docs.values().find(|one| one.file == served.elsewhere);
    assert_eq!(elsewhere.unwrap().folder, Some(mine));
}
