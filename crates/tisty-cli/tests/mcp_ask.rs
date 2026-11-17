use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

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

    // One process is one connection, and what it keeps open lives as long as it does.
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

    fn files_a_folder(&mut self, name: &str) -> Value {
        self.call("folder", json!({ "name": name }))
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn ask(said: &Value) -> Option<&Value> {
    said["structuredContent"].get("ask")
}

fn proposes_work(talk: &mut Connection) -> String {
    let said = talk.call(
        "propose",
        json!({
            "title": "Ship the export",
            "steps": ["Write the parser", "Wire the menu"],
            "list": "Work",
        }),
    );
    said["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn a_task_left_unwritten_while_the_agent_writes_elsewhere_is_asked_about_once() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes_work(&mut talk);

    assert_eq!(ask(&talk.files_a_folder("Notes")), None);
    assert_eq!(ask(&talk.files_a_folder("Drafts")), None);
    let said = talk.files_a_folder("Receipts");

    let asked = ask(&said).expect("the third write elsewhere asks");
    assert_eq!(asked["task"], json!(task));
    assert_eq!(asked["title"], json!("Ship the export"));
    assert_eq!(asked["left"], json!(2));
    let text = said["content"][0]["text"].as_str().unwrap();
    assert!(
        text.ends_with(asked["said"].as_str().unwrap()),
        "a client that reads only the text hears it too: {text}"
    );
    assert_eq!(
        ask(&talk.files_a_folder("Letters")),
        None,
        "once per stretch"
    );
}

#[test]
fn ticking_the_task_quiets_it_until_the_next_stretch() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes_work(&mut talk);
    for name in ["Notes", "Drafts", "Receipts"] {
        talk.files_a_folder(name);
    }

    let ticked = talk.call("tick", json!({ "task": task, "step": "Write the parser" }));
    assert_eq!(ask(&ticked), None);
    assert_eq!(ask(&talk.files_a_folder("Letters")), None);
    assert_eq!(ask(&talk.files_a_folder("Bills")), None);

    let asked = talk.files_a_folder("Photos");
    assert_eq!(ask(&asked).unwrap()["left"], json!(1));
}

#[test]
fn the_person_turning_it_off_quiets_a_connection_already_open() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes_work(&mut talk);
    talk.files_a_folder("Notes");
    talk.files_a_folder("Drafts");

    served.person().lets_agents_be_asked(false);
    assert_eq!(ask(&talk.files_a_folder("Receipts")), None);
    assert_eq!(ask(&talk.files_a_folder("Letters")), None);

    served.person().lets_agents_be_asked(true);
    talk.call(
        "note",
        json!({ "task": task, "body": "Chose CSV over XLSX: the bank reads it." }),
    );
    for name in ["Bills", "Photos"] {
        assert_eq!(ask(&talk.files_a_folder(name)), None);
    }
    assert!(ask(&talk.files_a_folder("Maps")).is_some());
}

#[test]
fn a_task_said_done_is_never_asked_about() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes_work(&mut talk);
    talk.call(
        "tick",
        json!({ "task": task, "steps": ["Write the parser", "Wire the menu"] }),
    );
    talk.call(
        "say_done",
        json!({ "task": task, "body": "Both steps are in; the export opens in the bank." }),
    );

    for name in ["Notes", "Drafts", "Receipts", "Letters"] {
        assert_eq!(ask(&talk.files_a_folder(name)), None);
    }
}

#[test]
fn a_task_the_person_wrote_and_the_agent_read_is_the_one_asked_about() {
    let served = Served::new();
    let theirs = served.person().captures("Renew the passport");
    served.person().adds_step(theirs, "Book the appointment");
    let mut talk = served.connect();

    talk.call("read", json!({ "task": theirs.to_string() }));
    for name in ["Notes", "Drafts"] {
        talk.files_a_folder(name);
    }
    let said = talk.files_a_folder("Receipts");

    assert_eq!(ask(&said).unwrap()["task"], json!(theirs.to_string()));
}

#[test]
fn a_note_filed_for_the_person_first_does_not_take_the_place_of_the_work() {
    let served = Served::new();
    let mut talk = served.connect();
    talk.call(
        "propose",
        json!({ "title": "Call the bank back", "list": "Work" }),
    );
    let task = proposes_work(&mut talk);
    for name in ["Notes", "Drafts"] {
        talk.files_a_folder(name);
    }

    let said = talk.files_a_folder("Receipts");

    assert_eq!(ask(&said).unwrap()["task"], json!(task));
}

#[test]
fn every_connection_keeps_its_own_open_task() {
    let served = Served::new();
    let mut first = served.connect();
    proposes_work(&mut first);
    let mut second = served.connect();

    for name in ["Notes", "Drafts", "Receipts"] {
        assert_eq!(ask(&second.files_a_folder(name)), None);
    }
}
