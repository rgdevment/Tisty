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

    fn page(&self, task: &str) -> String {
        let state = self.person().state();
        let id = person::task(task);
        let kept = tisty_core::task_docs::of(&state, id).expect("the task has a page");
        let root = self.home.path().join("data");
        tisty_core::docs::read(
            &tisty_core::Paths::new(root, self.home.path().join("config")).docs(),
            &kept.file,
        )
        .unwrap()
    }

    fn file(&self, task: &str) -> String {
        let state = self.person().state();
        tisty_core::task_docs::of(&state, person::task(task))
            .unwrap()
            .file
            .clone()
    }
}

struct Connection {
    child: Child,
    into: ChildStdin,
    from: BufReader<ChildStdout>,
    next: u64,
}

impl Connection {
    fn answer(&mut self, name: &str, args: Value) -> Value {
        self.next += 1;
        let asked = json!({
            "jsonrpc": "2.0", "id": self.next, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        });
        writeln!(self.into, "{asked}").unwrap();
        self.into.flush().unwrap();
        let mut line = String::new();
        self.from.read_line(&mut line).unwrap();
        serde_json::from_str::<Value>(&line).unwrap()["result"].clone()
    }

    fn call(&mut self, name: &str, args: Value) -> Value {
        let said = self.answer(name, args);
        assert_ne!(said["isError"], true, "{name}: {said}");
        said
    }

    fn refused(&mut self, name: &str, args: Value) -> String {
        let said = self.answer(name, args);
        assert_eq!(said["isError"], true, "{name} should be refused: {said}");
        said["content"][0]["text"].as_str().unwrap().to_string()
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

fn proposes(talk: &mut Connection) -> String {
    let said = talk.call(
        "propose",
        json!({
            "title": "Move payments to v2",
            "steps": ["Sign", "Upload", "Resume a cut upload"],
            "list": "Work",
        }),
    );
    shown(&said)["id"].as_str().unwrap().to_string()
}

const STATE: &str = "Signing works; upload is next.\n- **Decided:** v2 signatures only, the old ones are dropped.\n- **Ruled out:** retrying old signatures.";

#[test]
fn one_tick_marks_the_step_and_leaves_the_state_on_the_page() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    talk.call(
        "note",
        json!({ "task": task, "body": "Tried the old signatures first." }),
    );

    let said = talk.call(
        "tick",
        json!({ "task": task, "step": "Sign", "state": STATE }),
    );

    assert_eq!(shown(&said)["ticked"], 1);
    assert_eq!(shown(&said)["stated"], true);
    let text = said["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("Left its state on its page, under «State»."),
        "{text}"
    );
    let page = served.page(&task);
    let state = tisty_core::task_state::said(&page).unwrap();
    assert!(state.contains("v2 signatures only"), "{page}");
    assert!(
        state.starts_with("**"),
        "the state is signed and dated: {state}"
    );
    assert!(
        !page.contains("Tried the old signatures first."),
        "the page made for it links nothing of the journal in: {page}"
    );
    let held = served.person().state();
    let after = &held.tasks[&person::task(&task)];
    assert!(after.steps[0].done);
    assert!(
        after.stated_at.is_some(),
        "the trail says when the state was written"
    );
    assert!(!tisty_core::task_state::since(after).unwrap().old());
}

#[test]
fn a_state_alone_ticks_nothing_and_replaces_the_last_whole() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    talk.call("tick", json!({ "task": task, "state": "First take." }));

    let said = talk.call("tick", json!({ "task": task, "state": "Second take." }));

    assert_eq!(shown(&said)["stated"], true);
    assert_eq!(shown(&said)["left"], 3);
    assert_eq!(shown(&said)["next"], "Sign");
    let page = served.page(&task);
    assert!(!page.contains("First take."), "{page}");
    assert_eq!(page.matches("## State").count(), 1, "{page}");
    assert!(
        served.person().state().tasks[&person::task(&task)]
            .steps
            .iter()
            .all(|step| !step.done)
    );
}

#[test]
fn read_brings_the_state_and_only_the_notes_after_it() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    for n in 0..4 {
        talk.call("note", json!({ "task": task, "body": format!("try {n}") }));
    }
    talk.call(
        "tick",
        json!({ "task": task, "step": "Sign", "state": STATE }),
    );
    talk.call(
        "note",
        json!({ "task": task, "body": "Blocked: the bank's sandbox is down." }),
    );

    let said = talk.call("read", json!({ "task": task }));

    let kept = shown(&said);
    let journal = kept["journal"].as_array().unwrap();
    assert_eq!(journal.len(), 1, "{kept}");
    assert_eq!(journal[0]["body"], "Blocked: the bank's sandbox is down.");
    assert_eq!(kept["earlier"], 4);
    assert!(kept["state"]["written"].is_string(), "{kept}");
    assert!(
        kept["state"].get("body").is_none(),
        "a page that comes whole already holds it: {kept}"
    );
    assert_eq!(kept["state"]["old"], "older than 1 later note(s)");
    let text = said["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("4 note(s) from before its state"), "{text}");

    let log = talk.call("read", json!({ "task": task, "notes": 0 }));
    assert_eq!(shown(&log)["journal"].as_array().unwrap().len(), 5);
}

#[test]
fn a_long_page_brings_its_state_whole_beside_its_sections() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    let analysis = "The retries stopped once the queue kept its own clock. ".repeat(90);
    talk.call("append_doc", json!({ "doc": task, "body": analysis }));
    talk.call("tick", json!({ "task": task, "state": STATE }));

    let said = talk.call("read", json!({ "task": task }));

    let kept = shown(&said);
    assert_eq!(kept["document"]["whole"], false, "{kept}");
    let body = kept["state"]["body"].as_str().unwrap();
    assert!(body.contains("v2 signatures only"), "{body}");
}

#[test]
fn a_state_grows_old_when_steps_move_after_it_and_is_fresh_again_with_the_next() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    talk.call("tick", json!({ "task": task, "state": STATE }));

    talk.call("tick", json!({ "task": task, "step": "Sign" }));
    let said = talk.call("read", json!({ "task": task }));
    assert_eq!(
        shown(&said)["state"]["old"],
        "older than the last change to its steps"
    );

    talk.call(
        "tick",
        json!({ "task": task, "step": "Upload", "state": STATE }),
    );
    let again = talk.call("read", json!({ "task": task }));
    assert!(shown(&again)["state"].get("old").is_none(), "{again}");
}

#[test]
fn the_person_writing_the_state_makes_it_fresh_and_keeps_their_lines() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    talk.call(
        "tick",
        json!({ "task": task, "step": "Sign", "state": STATE }),
    );
    served.person().ticks(person::task(&task), 1);
    let file = served.file(&task);
    let page = served.page(&task);
    let theirs = page.replace(
        "- **Ruled out:** retrying old signatures.",
        "- **Ruled out:** retrying old signatures.\nAsk Rina before the upload.",
    );
    served
        .person()
        .writes_on_its_page(person::task(&task), &file, &theirs);

    let said = talk.call("read", json!({ "task": task }));
    assert!(shown(&said)["state"].get("old").is_none(), "{said}");

    let refused = talk.refused("tick", json!({ "task": task, "state": "All uploaded." }));
    assert!(
        refused.contains("«Ask Rina before the upload.»"),
        "{refused}"
    );
    assert!(
        refused.contains("nothing was ticked or written"),
        "{refused}"
    );
    assert!(served.page(&task).contains("Ask Rina before the upload."));

    let carried = "All uploaded.\nAsk Rina before the upload.";
    talk.call("tick", json!({ "task": task, "state": carried }));
    assert!(served.page(&task).contains("All uploaded."));
}

#[test]
fn a_state_that_cannot_be_kept_ticks_nothing() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);

    let long = "x".repeat(3_001);
    let refused = talk.refused(
        "tick",
        json!({ "task": task, "step": "Sign", "state": long }),
    );
    assert!(refused.contains("at most 3000 characters"), "{refused}");
    assert!(!served.person().state().tasks[&person::task(&task)].steps[0].done);

    talk.call("tick", json!({ "task": task, "state": "Short." }));
    served.person().locks_doc(&served.file(&task));
    let locked = talk.refused(
        "tick",
        json!({ "task": task, "step": "Sign", "state": "Again." }),
    );
    assert!(locked.contains("locked"), "{locked}");
    assert!(!served.person().state().tasks[&person::task(&task)].steps[0].done);
}

#[test]
fn a_routine_keeps_no_state() {
    let served = Served::new();
    let id = served.person().captures("Water the plants");
    served.person().adds_step(id, "Water");
    served.person().repeats(id, "every week");
    let mut talk = served.connect();

    let refused = talk.refused("tick", json!({ "task": id.to_string(), "state": "Done." }));

    assert!(refused.contains("routine"), "{refused}");
}

#[test]
fn a_task_said_done_takes_no_state() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    talk.call(
        "tick",
        json!({ "task": task, "steps": ["Sign", "Upload", "Resume a cut upload"] }),
    );
    talk.call(
        "say_done",
        json!({ "task": task, "body": "All three work on v2." }),
    );

    let refused = talk.refused("tick", json!({ "task": task, "state": "Done." }));

    assert!(refused.contains("already been said done"), "{refused}");
}

#[test]
fn without_a_written_state_read_builds_one_and_pins_the_closing_note() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    talk.call("tick", json!({ "task": task, "step": "Sign" }));

    let said = talk.call("read", json!({ "task": task }));
    let built = &shown(&said)["state"];
    assert_eq!(built["built"], true);
    assert_eq!(built["done"], 1);
    assert_eq!(built["of"], 3);

    talk.call(
        "tick",
        json!({ "task": task, "steps": ["Upload", "Resume a cut upload"] }),
    );
    talk.call(
        "say_done",
        json!({ "task": task, "body": "All three work on v2, run 4471." }),
    );
    for n in 0..12 {
        talk.call(
            "note",
            json!({ "task": task, "body": format!("later {n}: {}", "x".repeat(400)) }),
        );
    }
    let closed = talk.call("read", json!({ "task": task }));
    let closing = &shown(&closed)["state"]["closing"];
    assert!(
        closing["body"].as_str().unwrap().contains("run 4471"),
        "{closed}"
    );
}

#[test]
fn a_note_replaces_an_agents_earlier_one_which_stays_marked_in_the_history() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    talk.call(
        "note",
        json!({ "task": task, "body": "Decided: CSV for the bank." }),
    );
    let read = talk.call("read", json!({ "task": task }));
    let old = shown(&read)["journal"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let said = talk.call(
        "note",
        json!({ "task": task, "body": "Decided: XLSX, the bank changed its importer.", "replaces": old }),
    );

    assert_eq!(shown(&said)["replaces"], old);
    let text = said["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("read as replaced"), "{text}");
    let now = talk.call("read", json!({ "task": task }));
    let journal = shown(&now)["journal"].as_array().unwrap();
    assert_eq!(journal.len(), 1, "{now}");
    assert_eq!(journal[0]["replaces"], old);
    let log = talk.call("read", json!({ "task": task, "notes": 0 }));
    let all = shown(&log)["journal"].as_array().unwrap();
    assert_eq!(all[0]["replaced"], true, "{log}");
    let held = served.person().state();
    assert_eq!(held.tasks[&person::task(&task)].replaced().len(), 1);

    let twice = talk.refused(
        "note",
        json!({ "task": task, "body": "Decided: JSON.", "replaces": old }),
    );
    assert!(twice.contains("replaced already"), "{twice}");
}

#[test]
fn a_note_never_replaces_the_persons_or_one_it_cannot_find() {
    let served = Served::new();
    let id = served.person().captures("Renew the passport");
    served.person().notes(id, "Book it for March.");
    let mut talk = served.connect();
    let read = talk.call("read", json!({ "task": id.to_string() }));
    let theirs = shown(&read)["journal"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let refused = talk.refused(
        "note",
        json!({ "task": id.to_string(), "body": "April is better.", "replaces": theirs }),
    );
    assert!(refused.contains("the person's"), "{refused}");

    let unknown = talk.refused(
        "note",
        json!({ "task": id.to_string(), "body": "April.", "replaces": "01J00000000000000000000000" }),
    );
    assert!(unknown.contains("no note of"), "{unknown}");
    assert_eq!(served.person().state().tasks[&id].journal().count(), 1);
}

#[test]
fn a_state_answers_the_question_about_the_open_task() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposes(&mut talk);
    for name in ["Notes", "Drafts"] {
        talk.call("folder", json!({ "name": name }));
    }
    let asked = talk.call("folder", json!({ "name": "Receipts" }));
    let said = shown(&asked)["ask"]["said"].as_str().unwrap();
    assert!(said.contains("`state`"), "{said}");

    talk.call("tick", json!({ "task": task, "state": STATE }));

    for name in ["Letters", "Bills"] {
        assert!(
            shown(&talk.call("folder", json!({ "name": name })))
                .get("ask")
                .is_none()
        );
    }
}
