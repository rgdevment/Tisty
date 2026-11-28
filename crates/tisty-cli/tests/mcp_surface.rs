use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
        };
        served.person().begins("algo mio");
        served.person().lets_the_agent_in();
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn call(&self, name: &str, args: Value) -> Value {
        let init = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "test", "version": "1" }
            }
        })
        .to_string();
        let req = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": name, "arguments": args }
        })
        .to_string();
        let root = self.home.path();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
            .arg("mcp")
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        {
            let mut stdin = child.stdin.take().unwrap();
            writeln!(stdin, "{init}").unwrap();
            writeln!(stdin, "{req}").unwrap();
        }
        let out = child.wait_with_output().unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|one| one["id"] == 2)
            .unwrap()
    }
}

fn text_of(answer: &Value) -> &str {
    answer["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
}

fn refused(answer: &Value) -> bool {
    answer["result"]["isError"] == json!(true)
}

#[test]
fn every_legacy_tool_name_returns_its_replacement_and_mutates_nothing() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Plan\n\nStep one.\n");
    let task = served.person().captures("Check logs").to_string();
    let ops_before = served.person().ops().len();
    let body_before = std::fs::read_to_string(
        served
            .home
            .path()
            .join("data/docs")
            .join(format!("{doc}.md")),
    )
    .unwrap();

    let legacy: [(&str, &str); 42] = [
        ("catch_up", "workspace_catch_up"),
        ("brain", "workspace_search"),
        ("find", "workspace_search"),
        ("docs", "workspace_search"),
        ("lists", "workspace_search"),
        ("tags", "workspace_search"),
        ("outline_doc", "workspace_read"),
        ("read", "workspace_read"),
        ("read_doc", "workspace_read"),
        ("rules", "workspace_guide"),
        ("language", "workspace_guide"),
        ("help", "workspace_guide"),
        ("propose", "task_create"),
        ("describe", "task_update"),
        ("rename", "task_update"),
        ("reschedule", "task_update"),
        ("remind", "task_update"),
        ("move", "task_update"),
        ("plan", "task_edit_steps"),
        ("tick", "task_edit_steps"),
        ("untick", "task_edit_steps"),
        ("reword_step", "task_edit_steps"),
        ("unplan", "task_edit_steps"),
        ("note", "task_note"),
        ("reword_note", "task_note"),
        ("say_done", "task_resolve"),
        ("say_not_doing", "task_resolve"),
        ("write_doc", "doc_create"),
        ("edit_doc", "doc_edit"),
        ("append_doc", "doc_edit"),
        ("restore_doc", "doc_edit"),
        ("edit_table", "doc_edit_table"),
        ("folder", "doc_organize"),
        ("file_doc", "doc_organize"),
        ("page_doc", "doc_organize"),
        ("tongue_doc", "doc_organize"),
        ("archive_doc", "doc_annotate"),
        ("flag_doc", "doc_annotate"),
        ("sum_up", "doc_annotate"),
        ("attach", "file_transfer"),
        ("import_doc", "file_transfer"),
        ("export_doc", "file_transfer"),
    ];

    for (old, replacement) in legacy {
        let answer = served.call(old, json!({ "doc": doc, "task": task }));
        assert!(refused(&answer), "{old}: {answer}");
        let msg = text_of(&answer);
        assert!(
            msg.contains(&format!("`{old}` moved into `{replacement}`")),
            "{old} -> {replacement}: {msg}"
        );
    }

    assert_eq!(served.person().ops().len(), ops_before);
    let body_after = std::fs::read_to_string(
        served
            .home
            .path()
            .join("data/docs")
            .join(format!("{doc}.md")),
    )
    .unwrap();
    assert_eq!(body_after, body_before);
}

#[test]
fn v151_features_work_end_to_end_via_fifteen_tools() {
    let served = Served::new();

    let agent_task = served.call(
        "task_create",
        json!({
            "for": "agent",
            "title": "Ship the redesign",
            "steps": ["verify schema", "merge in UI"]
        }),
    );
    assert!(!refused(&agent_task), "{agent_task}");
    let agent_id = agent_task["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let person_task = served.call(
        "task_create",
        json!({ "for": "person", "title": "Sign release" }),
    );
    assert!(!refused(&person_task), "{person_task}");
    let person_id = person_task["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let read_person = served.call(
        "workspace_read",
        json!({ "task": person_id, "fields": ["for", "title"] }),
    );
    assert!(!refused(&read_person), "{read_person}");
    assert_eq!(
        read_person["result"]["structuredContent"]["for"],
        json!("person")
    );
    assert_eq!(
        read_person["result"]["structuredContent"]["title"],
        json!("Sign release")
    );

    let handed = served.call(
        "task_edit_steps",
        json!({
            "task": agent_id,
            "action": "remove",
            "step": "merge in UI",
            "to_person": true,
            "title": "Merge release in UI"
        }),
    );
    assert!(!refused(&handed), "{handed}");
    assert!(handed["result"]["structuredContent"]["to"].is_string());

    let ticked = served.call(
        "task_edit_steps",
        json!({ "task": agent_id, "action": "tick", "step": "verify schema" }),
    );
    assert!(!refused(&ticked), "{ticked}");

    let resolved = served.call(
        "task_resolve",
        json!({
            "task": agent_id,
            "outcome": "done",
            "body": "Verified cleanly.",
            "branch": "feat/mcp-15",
            "pr": "#42"
        }),
    );
    assert!(!refused(&resolved), "{resolved}");
    assert_eq!(
        resolved["result"]["structuredContent"]["branch"],
        json!("feat/mcp-15")
    );
    assert_eq!(resolved["result"]["structuredContent"]["pr"], json!("#42"));

    let mut parent: Option<String> = None;
    for level in 1..=6 {
        let name = format!("L{level}");
        let mut args = serde_json::Map::from_iter([
            ("action".into(), json!("create_folder")),
            ("name".into(), json!(name)),
        ]);
        if let Some(ref p) = parent {
            args.insert("folder".into(), json!(p));
        }
        let made = served.call("doc_organize", Value::Object(args));
        assert!(!refused(&made), "level {level}: {made}");
        parent = Some(match parent {
            Some(p) => format!("{p} / L{level}"),
            None => format!("L{level}"),
        });
    }
    assert_eq!(served.person().state().folders.len(), 6);
}

#[test]
fn task_update_rejects_person_dates_atomically_across_fields() {
    let served = Served::new();
    let res = served.call(
        "task_create",
        json!({ "for": "person", "title": "Original human title" }),
    );
    let task_id = res["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let failed = served.call(
        "task_update",
        json!({
            "task": task_id,
            "title": "Mutated title",
            "date": "2026-12-05"
        }),
    );
    assert!(refused(&failed), "{failed}");

    let id: tisty_core::model::TaskId = task_id.parse().unwrap();
    let state = served.person().state();
    assert_eq!(state.tasks[&id].title, "Original human title");
    assert_eq!(state.tasks[&id].date, None);
}

#[test]
fn doc_annotate_flag_on_single_doc_is_idempotent() {
    let served = Served::new();
    let keeper = served.person().writes_doc("# Current Spec\n\nLive.\n");
    let old = served.person().writes_doc("# Old Spec\n\nStale.\n");

    let args = json!({
        "action": "flag",
        "docs": [old],
        "reason": "duplicate_of",
        "instead": keeper,
        "body": "Superseded by Current Spec."
    });

    let first = served.call("doc_annotate", args.clone());
    assert!(!refused(&first), "{first}");
    assert_eq!(first["result"]["structuredContent"]["flagged"], json!(true));

    let second = served.call("doc_annotate", args);
    assert!(!refused(&second), "{second}");
    assert_eq!(
        second["result"]["structuredContent"]["flagged"],
        json!(false)
    );
    assert_eq!(
        second["result"]["structuredContent"]["skipped"],
        json!([old])
    );

    let state = served.person().state();
    let kept = state.docs.values().find(|d| d.file == old).unwrap();
    let flag = kept.flagged.as_ref().unwrap();
    assert_eq!(flag.reason.as_deref(), Some("duplicate_of"));
    assert_eq!(flag.instead.as_deref(), Some(keeper.as_str()));
}

#[test]
fn agents_proposal_flow_and_blocked_mutations() {
    let served = Served::new();
    let (_, claude) = served.person().keeps_agents();
    let rule_body = "---\nagent: claude/rule/CLAUDE.md\n---\n\n# Claude\n\nTests first.\n";
    let rule = served.person().writes_doc_in(rule_body, claude);
    let other = served.person().writes_doc("# Notes\n\nMine.\n");

    let patch = served.call(
        "doc_edit",
        json!({
            "doc": rule,
            "mode": "patch",
            "old": "Tests first.",
            "content": "Tests always."
        }),
    );
    assert!(!refused(&patch), "{patch}");
    assert_eq!(
        patch["result"]["structuredContent"]["proposed"],
        json!(true)
    );

    let append = served.call(
        "doc_edit",
        json!({
            "doc": rule,
            "mode": "append",
            "content": "Stay concise."
        }),
    );
    assert!(!refused(&append), "{append}");
    assert_eq!(
        append["result"]["structuredContent"]["proposed"],
        json!(true)
    );

    let print = append["result"]["structuredContent"]["print"]
        .as_str()
        .unwrap()
        .to_string();
    let rewritten_body = "---\nagent: claude/rule/CLAUDE.md\n---\n\n# Claude\n\nConcise rules.\n";
    let rewrite = served.call(
        "doc_edit",
        json!({
            "doc": rule,
            "mode": "rewrite",
            "print": print,
            "content": rewritten_body
        }),
    );
    assert!(!refused(&rewrite), "{rewrite}");
    assert_eq!(
        rewrite["result"]["structuredContent"]["proposed"],
        json!(true)
    );

    let state = served.person().state();
    let kept = state.docs.values().find(|d| d.file == rule).unwrap();
    assert_eq!(
        state.proposals.get(&kept.id).map(|p| p.body.as_str()),
        Some(rewritten_body)
    );

    for (name, args) in [
        (
            "doc_edit",
            json!({ "doc": rule, "mode": "compact", "print": print, "content": rewritten_body }),
        ),
        ("doc_edit", json!({ "doc": rule, "mode": "restore" })),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [other], "folder": "AGENTS / Claude" }),
        ),
        (
            "doc_annotate",
            json!({ "action": "archive", "docs": [rule] }),
        ),
        (
            "doc_edit_table",
            json!({ "doc": rule, "action": "add_row", "values": { "Col": "val" } }),
        ),
    ] {
        let answer = served.call(name, args);
        assert!(refused(&answer), "{name}: {answer}");
    }
}

#[test]
fn open_diary_allows_append_and_export_while_refusing_rewrites() {
    let served = Served::new();
    let today = jiff::ZonedDateTime::now()
        .in_tz("America/Santiago")
        .unwrap()
        .date();
    let (book, page) = served
        .person()
        .writes_in_the_diary(today, "Morning walk in the park.");
    served.person().opens_the_diary_to_agents(true);

    let appended = served.call(
        "doc_edit",
        json!({
            "doc": book,
            "mode": "append",
            "content": "Evening summary logged."
        }),
    );
    assert!(!refused(&appended), "{appended}");

    let written = std::fs::read_to_string(
        served
            .home
            .path()
            .join("data/docs")
            .join(format!("{page}.md")),
    )
    .unwrap();
    assert!(written.contains("Morning walk in the park."));
    assert!(written.contains("Evening summary logged."));

    let export_dir = tempfile::tempdir().unwrap();
    let exported = served.call(
        "file_transfer",
        json!({
            "action": "export_doc",
            "doc": page,
            "path": export_dir.path().to_string_lossy()
        }),
    );
    assert!(!refused(&exported), "{exported}");

    for (name, args) in [
        (
            "doc_edit",
            json!({ "doc": page, "mode": "patch", "old": "Morning", "content": "Late" }),
        ),
        (
            "doc_edit",
            json!({ "doc": page, "mode": "rewrite", "content": "# Replaced\n" }),
        ),
        (
            "doc_annotate",
            json!({ "action": "flag", "docs": [page], "reason": "obsolete", "body": "old" }),
        ),
    ] {
        let answer = served.call(name, args);
        assert!(refused(&answer), "{name}: {answer}");
    }
}

#[test]
fn task_update_is_atomic_when_list_or_date_is_invalid() {
    let served = Served::new();
    let created = served.call(
        "task_create",
        json!({ "for": "agent", "title": "Initial agent title" }),
    );
    let task_id = created["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let id: tisty_core::model::TaskId = task_id.parse().unwrap();

    for extra in [
        json!({ "task": task_id, "title": "Mutated", "list": "DoesNotExist" }),
        json!({ "task": task_id, "title": "Mutated", "date": "not-a-date" }),
    ] {
        let failed = served.call("task_update", extra);
        assert!(refused(&failed), "{failed}");
        let state = served.person().state();
        assert_eq!(state.tasks[&id].title, "Initial agent title");
        assert_eq!(state.tasks[&id].list, None);
        assert_eq!(state.tasks[&id].date, None);
    }
}

#[test]
fn doc_annotate_archive_is_atomic_across_multiple_docs() {
    let served = Served::new();
    let valid_doc = served.person().writes_doc("# Keep me\n\nStill needed.\n");

    let failed = served.call(
        "doc_annotate",
        json!({ "action": "archive", "docs": [valid_doc, "missing-doc"] }),
    );
    assert!(refused(&failed), "{failed}");

    let state = served.person().state();
    let kept = state.docs.values().find(|d| d.file == valid_doc).unwrap();
    assert!(
        !kept.archived,
        "valid_doc must stay unarchived on batch failure"
    );
}

#[test]
fn remapped_parameter_refusals_name_surface_parameters() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Spec\n\nDetails.\n");
    let task = served.person().captures("Review spec").to_string();

    let append = served.call("doc_edit", json!({ "doc": doc, "mode": "append" }));
    assert!(refused(&append), "{append}");
    assert!(
        text_of(&append).contains("`content`"),
        "{}",
        text_of(&append)
    );

    let export = served.call(
        "file_transfer",
        json!({ "action": "export_doc", "doc": doc }),
    );
    assert!(refused(&export), "{export}");
    assert!(text_of(&export).contains("`path`"), "{}", text_of(&export));

    let markdown = served.call(
        "workspace_read",
        json!({ "task": task, "view": "markdown" }),
    );
    assert!(refused(&markdown), "{markdown}");
    let msg = text_of(&markdown);
    assert!(msg.contains("`doc`") && !msg.contains("`as`"), "{msg}");
}

#[test]
fn workspace_search_folder_mode_refuses_query() {
    let served = Served::new();
    served.person().makes_folder("X", None, None);

    let failed = served.call(
        "workspace_search",
        json!({ "mode": "folder", "folder": "X", "query": "hello" }),
    );
    assert!(refused(&failed), "{failed}");
}

#[test]
fn workspace_read_refuses_versions_view_with_version() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Notes\n\nBody.\n");

    let failed = served.call(
        "workspace_read",
        json!({ "doc": doc, "view": "versions", "version": "abc123" }),
    );
    assert!(refused(&failed), "{failed}");
    let msg = text_of(&failed);
    assert!(
        msg.contains("`view: \"versions\"`")
            && msg.contains("`version`")
            && msg.contains("`view: \"content\"`"),
        "{msg}"
    );
}

#[test]
fn doc_organize_reorder_pages_refuses_stray_docs_and_position() {
    let served = Served::new();
    let book = served.person().writes_doc("# Book\n\nIntro.\n");
    let page_one = served.person().hangs_page_under(&book, "# One\n\nFirst.\n");
    let page_two = served
        .person()
        .hangs_page_under(&book, "# Two\n\nSecond.\n");

    for (key, value) in [("docs", json!([page_one])), ("position", json!("first"))] {
        let mut args = serde_json::Map::from_iter([
            ("action".into(), json!("reorder_pages")),
            ("parent".into(), json!(book)),
            ("order".into(), json!([page_two, page_one])),
        ]);
        args.insert(key.into(), value);
        let failed = served.call("doc_organize", Value::Object(args));
        assert!(refused(&failed), "{key}: {failed}");
        let msg = text_of(&failed);
        assert!(
            msg.contains("`parent` and `order`") && msg.contains(&format!("not `{key}`")),
            "{key}: {msg}"
        );
    }
}

#[test]
fn workspace_catch_up_uses_no_legacy_tool_names() {
    let served = Served::new();
    let person = served.person();
    for i in 1..=7 {
        let id = person.captures(&format!("Pressing task {i}"));
        person.dates(id, "today");
    }
    for i in 1..=31 {
        person.makes_folder(&format!("Folder {i:02}"), None, None);
    }
    person.writes_doc(
        "---\nview: notes\nbrainstorm: on\n---\n\n# Ideas\n\n> [!IDEA] 2026-06-14 08:12 · pink\n> First idea\n",
    );
    let started = served.call(
        "task_create",
        json!({ "for": "agent", "title": "Active agent task" }),
    );
    assert!(!refused(&started), "{started}");

    let caught = served.call("workspace_catch_up", json!({}));
    assert!(!refused(&caught), "{caught}");
    let kept = &caught["result"]["structuredContent"];
    assert!(kept["pressing_more"].is_number(), "{kept}");
    assert!(kept["folders_more"].is_number(), "{kept}");
    assert!(kept["ideas"].is_object(), "{kept}");
    assert!(kept["last"].is_object(), "{kept}");
    assert!(
        kept["open_by_list"]["inbox"].as_u64().unwrap() >= 7,
        "{kept}"
    );

    let msg = text_of(&caught);
    for legacy in ["`brain`", "`read`", "`rules`", "`to`"] {
        assert!(!msg.contains(legacy), "found legacy {legacy} in: {msg}");
    }
    for current in [
        "`workspace_search`",
        "`workspace_read`",
        "`workspace_guide`",
        "`date_to`",
    ] {
        assert!(msg.contains(current), "missing {current} in: {msg}");
    }
}

#[test]
fn workspace_search_invalid_dates_name_date_from_and_date_to() {
    let served = Served::new();
    for (key, bad) in [("date_from", "not-a-date"), ("date_to", "also-bad")] {
        let failed = served.call("workspace_search", json!({ key: bad }));
        assert!(refused(&failed), "{key}: {failed}");
        let msg = text_of(&failed);
        let legacy = if key == "date_from" { "`from`" } else { "`to`" };
        assert!(
            msg.contains(&format!("`{key}`")) && !msg.contains(legacy),
            "{key}: {msg}"
        );
    }
}

#[test]
fn doc_organize_add_page_refuses_order() {
    let served = Served::new();
    let book = served.person().writes_doc("# Book\n\nIntro.\n");
    let page_one = served.person().hangs_page_under(&book, "# One\n\nFirst.\n");
    let page_two = served
        .person()
        .hangs_page_under(&book, "# Two\n\nSecond.\n");

    let failed = served.call(
        "doc_organize",
        json!({
            "action": "add_page",
            "docs": [page_one],
            "parent": book,
            "order": [page_two, page_one],
        }),
    );
    assert!(refused(&failed), "{failed}");
    let msg = text_of(&failed);
    assert!(
        msg.contains("takes no `order`") && msg.contains("reorder_pages"),
        "{msg}"
    );
}

#[test]
fn refusal_and_hint_prose_matches_consolidated_surface() {
    let served = Served::new();
    let person = served.person();

    // gists.rs: refusing to summarize a task's page
    let task_id = person.captures("Task with page");
    let task_page = served.call(
        "task_update",
        json!({ "task": task_id.to_string(), "description": "# Plan\n\nBody.\n" }),
    );
    assert!(!refused(&task_page), "{task_page}");
    let task_doc = person
        .state()
        .docs
        .values()
        .find(|d| d.task_of == Some(task_id))
        .unwrap()
        .file
        .clone();
    let sum_task = served.call(
        "doc_annotate",
        json!({ "action": "summarize", "docs": [task_doc], "summary": "Nope" }),
    );
    assert!(refused(&sum_task), "{sum_task}");
    assert!(
        text_of(&sum_task).contains("`doc_edit`") && text_of(&sum_task).contains("`task_resolve`"),
        "{sum_task}"
    );
    let legacy_task_doc = tisty_core::docs::create(
        &served.home.path().join("data/docs"),
        &tisty_core::DeviceId(person.device()),
        "# Page\n\nBody.\n",
    )
    .unwrap()
    .id;
    person.does(tisty_core::Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::DocAdd {
            file: legacy_task_doc.clone(),
            order: "a0".into(),
            task_of: Some(task_id),
            ..Default::default()
        },
    });
    let sum_legacy_task = served.call(
        "doc_annotate",
        json!({ "action": "summarize", "docs": [legacy_task_doc], "summary": "Nope" }),
    );
    assert!(refused(&sum_legacy_task), "{sum_legacy_task}");
    assert!(
        text_of(&sum_legacy_task).contains("`task_edit_steps`"),
        "{sum_legacy_task}"
    );

    // papers.rs & filing.rs: empty/missing docs, archive hint, folder path errors
    let empty_archive = served.call("doc_annotate", json!({ "action": "archive", "docs": [] }));
    assert!(refused(&empty_archive) && text_of(&empty_archive).contains("`docs`"));
    let missing_archive = served.call("doc_annotate", json!({ "action": "archive" }));
    assert!(refused(&missing_archive) && text_of(&missing_archive).contains("`docs`"));

    let archived_doc = person.writes_doc("# Archive me\n\nBody.\n");
    let archived = served.call(
        "doc_annotate",
        json!({ "action": "archive", "docs": [archived_doc] }),
    );
    assert!(!refused(&archived), "{archived}");
    let archived_msg = text_of(&archived);
    assert!(
        archived_msg.contains("`workspace_search`") && archived_msg.contains("`doc_annotate`"),
        "{archived_msg}"
    );

    person.makes_folder("Alpha", None, None);
    person.makes_folder("Beta", None, None);
    let clash = served.call(
        "doc_organize",
        json!({ "action": "create_folder", "name": "Alpha/Child", "folder": "Beta" }),
    );
    assert!(refused(&clash) && text_of(&clash).contains("`folder`"));
    let missing_parent = served.call(
        "doc_organize",
        json!({ "action": "create_folder", "name": "NoSuchParent/Child" }),
    );
    assert!(refused(&missing_parent) && text_of(&missing_parent).contains("`folder`"));

    // outward.rs: markdown with section refused, markdown on book warns about pages
    let book = person.writes_doc("# Book\n\nIntro.\n");
    let page_one = person.hangs_page_under(&book, "# One\n\nFirst.\n");
    let unrelated = person.writes_doc("# Other\n\nOutside.\n");
    let md_section = served.call(
        "workspace_read",
        json!({ "doc": book, "view": "markdown", "section": 1 }),
    );
    assert!(refused(&md_section) && text_of(&md_section).contains("`view: \"markdown\"`"));
    let md_book = served.call("workspace_read", json!({ "doc": book, "view": "markdown" }));
    assert!(!refused(&md_book), "{md_book}");
    assert!(text_of(&md_book).contains("`view: \"markdown\"`"));

    // reading.rs: reorder_pages missing parent, add_page self/unrelated/unlinked anchor, reorder non-page/unlinked
    let no_parent = served.call(
        "doc_organize",
        json!({ "action": "reorder_pages", "order": [page_one] }),
    );
    assert!(refused(&no_parent) && text_of(&no_parent).contains("`parent`"));

    let self_anchor = served.call(
        "doc_organize",
        json!({ "action": "add_page", "docs": [page_one], "parent": book, "after": page_one }),
    );
    assert!(refused(&self_anchor) && text_of(&self_anchor).contains("`position`"));

    let bad_anchor = served.call(
        "doc_organize",
        json!({ "action": "add_page", "docs": [page_one], "parent": book, "after": unrelated }),
    );
    assert!(refused(&bad_anchor) && text_of(&bad_anchor).contains("`workspace_read`"));

    let unlinked = person.hangs_page_under(&book, "# Unlinked\n\nSecond.\n");
    person.rewrites_doc(&book, "# Book\n\nIntro without cards.\n");

    let anchor_missing_line = served.call(
        "doc_organize",
        json!({ "action": "add_page", "docs": [page_one], "parent": book, "after": unlinked }),
    );
    assert!(refused(&anchor_missing_line) && text_of(&anchor_missing_line).contains("`position`"));

    let reorder_non_page = served.call(
        "doc_organize",
        json!({ "action": "reorder_pages", "parent": book, "order": [unrelated, page_one] }),
    );
    assert!(refused(&reorder_non_page) && text_of(&reorder_non_page).contains("`doc_organize`"));

    let reorder_unlinked = served.call(
        "doc_organize",
        json!({ "action": "reorder_pages", "parent": book, "order": [page_one, unlinked] }),
    );
    assert!(refused(&reorder_unlinked) && text_of(&reorder_unlinked).contains("`position`"));

    // task_state.rs: replacing unknown note, person's note, and done note
    let note_task = person.captures("Noted task").to_string();
    let bad_replace = served.call(
        "task_note",
        json!({ "action": "add", "task": note_task, "body": "Fix", "replaces": "9999" }),
    );
    assert!(refused(&bad_replace) && text_of(&bad_replace).contains("`workspace_read`"));

    person.notes(note_task.parse().unwrap(), "Person's own note");
    let person_note_id = person.state().tasks[&note_task.parse().unwrap()].log[0]
        .id
        .to_string();
    let replace_person = served.call(
        "task_note",
        json!({ "action": "add", "task": note_task, "body": "Takeover", "replaces": person_note_id }),
    );
    assert!(refused(&replace_person) && text_of(&replace_person).contains("`task_note`"));

    let resolved = served.call(
        "task_resolve",
        json!({ "task": note_task, "outcome": "done", "body": "All checks passed" }),
    );
    assert!(!refused(&resolved), "{resolved}");
    let done_note_id = person.state().tasks[&note_task.parse().unwrap()]
        .log
        .last()
        .unwrap()
        .id
        .to_string();
    let replace_done = served.call(
        "task_note",
        json!({ "action": "add", "task": note_task, "body": "Undo done", "replaces": done_note_id }),
    );
    assert!(refused(&replace_done) && text_of(&replace_done).contains("`task_note`"));
}
