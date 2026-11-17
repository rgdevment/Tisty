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
        let person = served.person();
        person.begins("algo mio");
        person.lets_the_agent_in();
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn catch_up(&self) -> Value {
        let root = self.home.path();
        let asked = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": "catch_up", "arguments": {} },
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
        let said: Value =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
        said["result"].clone()
    }

    fn opened_here(&self, file: &str) {
        let at = self.home.path().join("cache").join("opened.json");
        std::fs::create_dir_all(at.parent().unwrap()).unwrap();
        let now = jiff::Timestamp::now().as_second();
        std::fs::write(at, json!({ file: now }).to_string()).unwrap();
    }
}

fn said(caught: &Value) -> String {
    caught["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn with_no_weekly_review_set_catch_up_says_nothing_of_one() {
    let served = Served::new();

    let caught = served.catch_up();

    assert!(caught["structuredContent"]["review"].is_null(), "{caught}");
    assert!(
        !said(&caught).contains("weekly review"),
        "{}",
        said(&caught)
    );
}

#[test]
fn a_review_due_today_comes_with_what_waits_and_how_to_help() {
    let served = Served::new();
    let person = served.person();
    let review = person.captures("Weekly review");
    person.tags(review, "weekly-review");
    person.repeats(review, "every week");
    person.dates(review, "today");
    person.captures("Call Jaime");
    let old = person.wrote_doc_days_ago("# Recetas de la abuela\n", 90);
    person.writes_doc("# Garantías de la casa\n");

    let caught = served.catch_up();

    let glance = &caught["structuredContent"]["review"];
    assert_eq!(glance["title"], json!("Weekly review"), "{caught}");
    assert_eq!(
        glance["waiting"],
        json!({ "unlisted": 2, "ideas": 0, "marked": 0, "forgotten": 1 }),
        "the review itself is not one of the tasks to decide"
    );
    let text = said(&caught);
    assert!(text.contains("weekly review, «Weekly review»"), "{text}");
    assert!(text.contains("is due today"), "{text}");
    assert!(
        text.contains("never archive, delete, mark or close"),
        "{text}"
    );
    let told = glance["said"].as_str().unwrap_or_default();
    assert!(
        told.contains("never archive, delete, mark or close"),
        "a client that reads only the structured answer is told too: {told}"
    );

    served.opened_here(&old);

    let caught = served.catch_up();
    assert_eq!(
        caught["structuredContent"]["review"]["waiting"]["forgotten"],
        json!(0),
        "an opening on this computer counts: {caught}"
    );
}

#[test]
fn a_review_whose_day_has_not_come_is_not_due() {
    let served = Served::new();
    let person = served.person();
    let review = person.captures("Weekly review");
    person.tags(review, "weekly-review");
    person.repeats(review, "every week");
    person.dates(review, "in 3 days");

    let caught = served.catch_up();

    assert!(caught["structuredContent"]["review"].is_null(), "{caught}");
}
