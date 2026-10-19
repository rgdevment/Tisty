use std::io::Write;
use std::process::{Command, Stdio};

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
        let person = served.person();
        person.begins("algo mio");
        person.lets_the_agent_in();
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn command(&self) -> Command {
        let root = self.home.path();
        let mut command = Command::new(env!("CARGO_BIN_EXE_tisty"));
        command
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL");
        command
    }

    fn call(&self, name: &str, args: serde_json::Value) -> serde_json::Value {
        let asked = serde_json::json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
        .to_string();
        let mut child = self
            .command()
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

    /// Made the way the window will make it: the file first, then the log learns of it.
    fn written_for(&self, task: &str, words: &str) {
        let data = self.home.path().join("data");
        let store = data.join("store");
        let events = tisty_core::store::read_all(&store).unwrap();
        let state = tisty_core::State::replay(&events);
        let task: tisty_core::model::TaskId = task.parse().unwrap();
        let op =
            tisty_core::task_docs::made(&data.join("docs"), &state, task, task, "", None).unwrap();
        let tisty_core::Op::DocAdd { d, .. } = &op else {
            panic!("a task's document is added");
        };
        let file = d.file.clone();
        tisty_core::docs::write(
            &data.join("docs"),
            &file,
            &format!("{}\n{words}\n", tisty_core::task_docs::seed(task)),
        )
        .unwrap();
        let device = events.last().unwrap().device.clone();
        tisty_core::Store::open(&store, device)
            .unwrap()
            .append(op)
            .unwrap();
    }
}

#[test]
fn a_task_is_found_by_the_words_of_its_document_and_the_document_is_not_listed_on_its_own() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "Ask about the tick tablet at the visit.");

    let found = served.call("find", serde_json::json!({ "query": "tick tablet" }));
    let said = &found["result"]["structuredContent"];
    assert_eq!(said["matches"][0]["id"], task.as_str(), "{found}");
    assert_eq!(said["matches"][0]["in"], "document", "{found}");
    assert_eq!(
        said["docsTotal"], 0,
        "the task's document showed as a document: {found}"
    );

    let listed = served.call("docs", serde_json::json!({}));
    let docs = listed["result"]["structuredContent"]["docs"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(docs.is_empty(), "{listed}");
}

#[test]
fn an_assistant_cannot_write_a_task_s_words_through_the_document_tools() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "The person's own words.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let refused = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "own", "new": "changed" }),
    );

    assert_eq!(refused["result"]["isError"], true, "{refused}");
    let said = refused["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("task's document"), "{said}");
    let padded = served.call(
        "append_doc",
        serde_json::json!({ "doc": format!(" {file}\t"), "body": "Slipped in." }),
    );
    assert_eq!(padded["result"]["isError"], true, "{padded}");
    let said = padded["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("task's document"), "{said}");
    let read = served.call("read_doc", serde_json::json!({ "doc": file }));
    let body = read["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap_or_default();
    assert!(body.contains("The person's own words."), "{read}");
}

#[test]
fn what_an_assistant_changes_on_a_task_it_filed_reaches_its_page() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(
        &task,
        "## Description\n<!-- What and why -->\n\n## Steps\n<!-- - [ ] The first step -->\n",
    );
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let page = || {
        let read = served.call("read", serde_json::json!({ "task": task }));
        read["result"]["structuredContent"]["document"]["body"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };

    served.call(
        "describe",
        serde_json::json!({ "task": task, "body": "Book it before Friday." }),
    );
    served.call(
        "plan",
        serde_json::json!({ "task": task, "steps": ["Call the vet", "Bring the card"] }),
    );
    served.call(
        "reword_step",
        serde_json::json!({ "task": task, "step": "Call the vet", "text": "Call the vet at nine" }),
    );
    served.call(
        "unplan",
        serde_json::json!({ "task": task, "steps": ["Bring the card"] }),
    );
    served.call(
        "tick",
        serde_json::json!({ "task": task, "steps": ["Call the vet at nine"] }),
    );

    let body = page();
    assert!(
        body.contains("<!-- What and why -->\n\nBook it before Friday."),
        "{body}"
    );
    assert!(body.contains("- [x] Call the vet at nine"), "{body}");
    assert!(!body.contains("Bring the card"), "{body}");
    assert!(!body.contains("- [ ] Call the vet\n"), "{body}");
    assert!(body.starts_with("---\ntask: "), "{body}");
    let read = served.call("read_doc", serde_json::json!({ "doc": file }));
    let kept = read["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap_or_default();
    assert!(kept.contains("- [x] Call the vet at nine"), "{kept}");
}

#[test]
fn a_task_with_no_page_gets_one_from_its_template_the_first_time_it_is_written() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    served.call(
        "describe",
        serde_json::json!({ "task": task, "body": "Book it before Friday." }),
    );

    let read = served.call("read", serde_json::json!({ "task": task }));
    let body = read["result"]["structuredContent"]["document"]["body"]
        .as_str()
        .unwrap_or_default();
    assert!(body.contains("<!--"), "the page took no template: {body}");
    assert!(body.contains("Book it before Friday."), "{body}");
    assert_eq!(body.matches("Book it before Friday.").count(), 1, "{body}");
}

#[test]
fn an_assistant_adds_to_the_page_of_a_task_it_filed_under_its_own_notes_by_the_task_s_id() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "The person's own words.");

    let added = served.call(
        "append_doc",
        serde_json::json!({ "doc": task, "body": "## Price\nForty at the vet.", "under": "Notes" }),
    );

    assert_ne!(added["result"]["isError"], true, "{added}");
    let said = added["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("assistant's notes"), "{said}");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let read = served.call("read_doc", serde_json::json!({ "doc": file }));
    let body = read["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap_or_default();
    assert!(body.contains("The person's own words."), "{read}");
    assert!(body.contains("## Assistant notes"), "{read}");
    assert!(body.contains("### Price\nForty at the vet."), "{read}");
}

#[test]
fn a_task_the_person_kept_to_themselves_takes_no_assistant_notes_on_its_page() {
    let served = Served::new();
    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    let task = found["result"]["structuredContent"]["matches"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let refused = served.call(
        "append_doc",
        serde_json::json!({ "doc": task, "body": "Slipped in." }),
    );

    assert_eq!(refused["result"]["isError"], true, "{refused}");
    let said = refused["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("person's own"), "{said}");
}

#[test]
fn an_old_copy_of_a_task_s_page_is_turned_away_rather_than_erase_what_came_since() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.call(
        "plan",
        serde_json::json!({ "task": task, "steps": ["Call the vet"] }),
    );
    let read = served.call("read", serde_json::json!({ "task": task }));
    let copy = read["result"]["structuredContent"]["document"]["body"]
        .as_str()
        .unwrap()
        .to_string();
    served.call(
        "plan",
        serde_json::json!({ "task": task, "steps": ["Bring the card"] }),
    );
    let at = served.home.path().join("copy.md");
    std::fs::write(&at, &copy).unwrap();

    let turned = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_string_lossy() }),
    );

    assert_eq!(turned["result"]["isError"], true, "{turned}");
    let said = turned["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("leaves out"), "{said}");
    let read = served.call("read", serde_json::json!({ "task": task }));
    let body = read["result"]["structuredContent"]["document"]["body"]
        .as_str()
        .unwrap_or_default();
    assert!(body.contains("- [ ] Bring the card"), "{body}");

    served.call(
        "tick",
        serde_json::json!({ "task": task, "steps": ["Call the vet", "Bring the card"] }),
    );
    let done = served.call(
        "say_done",
        serde_json::json!({ "task": task, "body": "booked" }),
    );
    assert!(done["result"]["isError"].is_null(), "{done}");
    std::fs::write(&at, body).unwrap();
    let late = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_string_lossy() }),
    );
    assert_eq!(late["result"]["isError"], true, "{late}");
    let said = late["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("already been said done"), "{said}");
}
