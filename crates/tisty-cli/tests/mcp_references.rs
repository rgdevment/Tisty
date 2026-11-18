use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

const A_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
];

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

    fn connect(&self) -> Connection {
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
        let into = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        Connection {
            child,
            into,
            from,
            next: 0,
        }
    }
}

struct Connection {
    child: Child,
    into: ChildStdin,
    from: BufReader<ChildStdout>,
    next: u64,
}

impl Connection {
    fn call(&mut self, name: &str, args: Value) -> Value {
        self.next += 1;
        let asked = json!({
            "jsonrpc": "2.0", "id": self.next, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        });
        writeln!(self.into, "{asked}").unwrap();
        self.into.flush().unwrap();
        let mut line = String::new();
        self.from.read_line(&mut line).unwrap();
        let said: Value = serde_json::from_str(&line).unwrap();
        assert_ne!(said["result"]["isError"], true, "{name}: {said}");
        said["result"].clone()
    }

    fn body_of(&mut self, doc: &str) -> String {
        let said = self.call("read_doc", json!({ "doc": doc }));
        said["structuredContent"]["body"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn shown(said: &Value) -> &Value {
    &said["structuredContent"]
}

fn text(said: &Value) -> &str {
    said["content"][0]["text"].as_str().unwrap_or_default()
}

fn proposes_work(talk: &mut Connection) -> String {
    let said = talk.call(
        "propose",
        json!({ "title": "Ship the export", "steps": ["Write the parser"], "list": "Work" }),
    );
    shown(&said)["id"].as_str().unwrap().to_string()
}

#[test]
fn a_note_links_a_task_named_by_id_and_a_document_named_by_its_exact_title() {
    let served = Served::new();
    let bank = served.person().captures("Call the bank");
    let decisions = served
        .person()
        .writes_doc("# Brainsty: decisiones\n\nWhat holds.\n");
    let mut talk = served.connect();
    let task = proposes_work(&mut talk);

    let said = talk.call(
        "note",
        json!({ "task": task, "body": format!("Ask {bank} first, as «Brainsty: decisiones» says.") }),
    );

    let linked = &shown(&said)["references"]["linked"];
    assert_eq!(linked[0]["task"], json!(bank.to_string()));
    assert_eq!(linked[0]["title"], json!("Call the bank"));
    assert_eq!(linked[1]["doc"], json!(decisions));
    assert!(
        text(&said).contains("Tisty linked what this names by id or exact title"),
        "{}",
        text(&said)
    );
    let read = talk.call("read", json!({ "task": task, "notes": 0 }));
    let journal = read["structuredContent"]["journal"].to_string();
    assert!(
        journal.contains(&format!("[Call the bank](tisty:task/{bank})")),
        "{journal}"
    );
    assert!(
        journal.contains(&format!("«[Brainsty: decisiones](tisty:doc/{decisions})»")),
        "{journal}"
    );
}

#[test]
fn a_title_two_documents_carry_is_proposed_and_left_as_written() {
    let served = Served::new();
    served.person().writes_doc("# Vet visits\n\nFirst.\n");
    served.person().writes_doc("# Vet visits\n\nSecond.\n");
    let mut talk = served.connect();
    talk.call("folder", json!({ "name": "Personal" }));

    let said = talk.call(
        "write_doc",
        json!({ "body": "# Rina\n\nBook «Vet visits» soon.\n", "folder": "Personal" }),
    );

    let maybe = &shown(&said)["references"]["maybe"];
    assert_eq!(maybe[0]["said"], json!("Vet visits"));
    assert_eq!(maybe[0]["could_be"].as_array().map(Vec::len), Some(2));
    let doc = shown(&said)["doc"].as_str().unwrap().to_string();
    assert!(talk.body_of(&doc).contains("Book «Vet visits» soon."));
}

#[test]
fn a_hidden_task_carrying_the_same_title_changes_nothing_in_the_answer() {
    let served = Served::new();
    served.person().writes_doc("# Vet visits\n\nThe card.\n");
    let mut talk = served.connect();
    talk.call("folder", json!({ "name": "Personal" }));
    let before = talk.call(
        "write_doc",
        json!({ "body": "# Rina\n\nBook «Vet visits» soon.\n", "folder": "Personal" }),
    );
    let twin = served.person().captures("Vet visits");
    served.person().hides(twin);

    let after = talk.call(
        "write_doc",
        json!({ "body": "# Bruno\n\nBook «Vet visits» soon.\n", "folder": "Personal" }),
    );

    assert_eq!(
        shown(&after)["references"],
        shown(&before)["references"],
        "a hidden task neither counts as a second candidate nor shows"
    );
    assert!(!after.to_string().contains(&twin.to_string()));
}

#[test]
fn a_line_the_person_wrote_is_kept_as_they_wrote_it_when_the_document_is_written_again() {
    let served = Served::new();
    served
        .person()
        .writes_doc("# Brainsty: decisiones\n\nWhat holds.\n");
    let plans = served
        .person()
        .writes_doc("# Plans\n\nRead «Brainsty: decisiones» later.\n");
    let mut talk = served.connect();
    let print = talk.call("read_doc", json!({ "doc": plans }))["structuredContent"]["print"]
        .as_str()
        .unwrap()
        .to_string();

    talk.call(
        "write_doc",
        json!({
            "doc": plans,
            "print": print,
            "body": "# Plans\n\nRead «Brainsty: decisiones» later.\n\nThen «Brainsty: decisiones» again.\n",
        }),
    );

    let body = talk.body_of(&plans);
    assert!(
        body.contains("Read «Brainsty: decisiones» later."),
        "{body}"
    );
    assert!(
        body.contains("Then «[Brainsty: decisiones](tisty:doc/"),
        "{body}"
    );
}

#[test]
fn attaching_a_picture_named_by_its_file_asks_what_it_shows() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Album\n\nPhotos.\n");
    let dir = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();
    let at = dir.path().join("IMG_2034.png");
    std::fs::write(&at, A_PNG).unwrap();
    let mut talk = served.connect();

    let unnamed = talk.call(
        "attach",
        json!({ "doc": doc, "path": at.to_str().unwrap() }),
    );
    let named = talk.call(
        "attach",
        json!({ "doc": doc, "path": at.to_str().unwrap(), "label": "Rina on her chair" }),
    );

    let ask = &shown(&unnamed)["ask"];
    assert_eq!(ask["pictures"].as_array().map(Vec::len), Some(1));
    assert!(
        ask["said"].as_str().unwrap().contains("`edit_doc`"),
        "{ask}"
    );
    assert!(text(&unnamed).ends_with(ask["said"].as_str().unwrap()));
    assert_eq!(shown(&named).get("ask"), None);
}

#[test]
fn a_picture_written_without_words_is_asked_about_and_the_open_task_question_waits() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes_work(&mut talk);
    talk.call("folder", json!({ "name": "Notes" }));
    talk.call("folder", json!({ "name": "Drafts" }));

    let pictured = talk.call(
        "write_doc",
        json!({ "body": "# Scan\n\n![](scan-3f9a1c2b.png)\n", "folder": "Notes" }),
    );
    let next = talk.call("folder", json!({ "name": "Receipts" }));

    assert_eq!(
        shown(&pictured)["ask"]["pictures"],
        json!(["scan-3f9a1c2b.png"])
    );
    assert_eq!(
        shown(&next)["ask"]["task"],
        json!(task),
        "the open task's question rides on the next answer instead"
    );
}
