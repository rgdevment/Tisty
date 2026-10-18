use std::io::Write;
use std::process::{Command, Stdio};

use tempfile::TempDir;

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
        };
        served.cli(&["algo mio"]);
        served.cli(&["agent", "--on"]);
        served
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

    fn cli(&self, args: &[&str]) -> String {
        let out = self
            .command()
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
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
        let op = tisty_core::task_docs::made(&data.join("docs"), &state, task, task, "", None)
            .unwrap()
            .op;
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
    let read = served.call("read_doc", serde_json::json!({ "doc": file }));
    let body = read["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap_or_default();
    assert!(body.contains("The person's own words."), "{read}");
}
