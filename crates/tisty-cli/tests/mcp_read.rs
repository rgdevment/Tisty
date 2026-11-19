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
            into: Some(into),
            from,
            next: 0,
        }
    }
}

struct Connection {
    child: Child,
    into: Option<ChildStdin>,
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
        let into = self.into.as_mut().unwrap();
        writeln!(into, "{asked}").unwrap();
        into.flush().unwrap();
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

// Ending at the close of its input, it writes what coverage counted; a killed process never does.
impl Drop for Connection {
    fn drop(&mut self) {
        drop(self.into.take());
        let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while matches!(self.child.try_wait(), Ok(None)) && std::time::Instant::now() < until {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn shown(said: &Value) -> &Value {
    &said["structuredContent"]
}

fn text(said: &Value) -> &str {
    said["content"][0]["text"].as_str().unwrap()
}

fn proposed(talk: &mut Connection, title: &str) -> String {
    let said = talk.call(
        "propose",
        json!({ "title": title, "list": "Personal", "steps": ["one", "two"] }),
    );
    shown(&said)["id"].as_str().unwrap().to_string()
}

#[test]
fn read_takes_a_task_or_a_doc_and_never_both_nor_neither() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recetas\n\narroz\n");
    let mut talk = served.connect();
    let task = proposed(&mut talk, "Plan the week");

    let both = talk.refused("read", json!({ "task": task, "doc": doc }));
    assert!(both.contains("one thing at a time"), "{both}");
    let neither = talk.refused("read", json!({}));
    assert!(
        neither.contains("needs a `task` id or a `doc` id"),
        "{neither}"
    );

    let by_task = talk.call("read", json!({ "task": task }));
    assert_eq!(shown(&by_task)["title"], "Plan the week");
    let by_doc = talk.call("read", json!({ "doc": doc }));
    assert_eq!(shown(&by_doc)["body"], "# Recetas\n\narroz\n");
}

#[test]
fn a_field_of_one_kind_sent_with_the_other_is_refused_and_named() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recetas\n\narroz\n");
    let mut talk = served.connect();
    let task = proposed(&mut talk, "Plan the week");

    for (field, value) in [("fields", json!(["steps"])), ("notes", json!(2))] {
        let why = talk.refused("read", json!({ "doc": doc, field: value }));
        assert!(
            why.starts_with(&format!("`{field}` is for reading a `task`")),
            "{why}"
        );
    }
    for (field, value) in [
        ("section", json!(0)),
        ("from", json!(1)),
        ("to", json!(2)),
        ("chars", json!(10)),
        ("cursor", json!(2)),
        ("as", json!("markdown")),
        ("marks", json!(true)),
    ] {
        let why = talk.refused("read", json!({ "task": task, field: value }));
        assert!(
            why.starts_with(&format!("`{field}` is for reading a `doc`")),
            "{why}"
        );
    }
}

#[test]
fn read_doc_answers_only_with_where_it_went() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recetas\n\narroz\n");
    let mut talk = served.connect();
    let why = talk.refused("read_doc", json!({ "doc": doc }));
    assert!(
        why.starts_with("`read_doc` moved into `read`: call `read` with the same `doc`"),
        "{why}"
    );
}

#[test]
fn a_hidden_tasks_page_read_as_a_document_is_not_there() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposed(&mut talk, "Private errand");
    talk.call(
        "append_doc",
        json!({ "doc": task, "body": "Something to keep." }),
    );
    let state = served.person().state();
    let id: tisty_core::TaskId = task.parse().unwrap();
    let page = tisty_core::task_docs::of(&state, id).unwrap().file.clone();
    served.person().hides(id);

    let hidden = talk.answer("read", json!({ "doc": page }));
    let missing = talk.answer("read", json!({ "doc": "zzzzzzzz-9999" }));
    assert_eq!(hidden["isError"], true, "{hidden}");
    assert_eq!(
        text(&hidden).replace(&page, "X"),
        text(&missing).replace("zzzzzzzz-9999", "X"),
        "the same answer as a document that does not exist"
    );
}

#[test]
fn a_task_read_again_with_its_print_says_only_that_nothing_changed() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposed(&mut talk, "Plan the week");

    let first = talk.call("read", json!({ "task": task }));
    let print = shown(&first)["print"].as_str().unwrap().to_string();
    assert!(
        print.contains('.'),
        "the print carries when it was read: {print}"
    );

    let again = talk.call("read", json!({ "task": task, "print": print }));
    assert_eq!(shown(&again)["unchanged"], true, "{again}");
    assert_eq!(shown(&again)["status"], "open");
    assert!(shown(&again).get("steps").is_none(), "{again}");
    assert!(
        text(&again).contains("changed since you read it, "),
        "{again}"
    );

    talk.call("tick", json!({ "task": task, "step": "one" }));
    let moved = talk.call("read", json!({ "task": task, "print": print }));
    assert!(shown(&moved).get("unchanged").is_none(), "{moved}");
    assert!(shown(&moved)["steps"].is_array(), "{moved}");
}

#[test]
fn a_narrower_read_of_the_same_task_is_another_print() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposed(&mut talk, "Plan the week");

    let whole = talk.call("read", json!({ "task": task }));
    let print = shown(&whole)["print"].as_str().unwrap().to_string();
    let narrow = talk.call(
        "read",
        json!({ "task": task, "fields": ["steps"], "print": print }),
    );
    assert!(shown(&narrow).get("unchanged").is_none(), "{narrow}");
    assert!(shown(&narrow)["steps"].is_array(), "{narrow}");
}

#[test]
fn a_print_without_its_time_still_says_unchanged() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposed(&mut talk, "Plan the week");
    let first = talk.call("read", json!({ "task": task }));
    let print = shown(&first)["print"].as_str().unwrap();
    let bare = print.split('.').next().unwrap();

    let again = talk.call("read", json!({ "task": task, "print": bare }));
    assert_eq!(shown(&again)["unchanged"], true, "{again}");
    assert!(
        text(&again).contains("changed since you read it:"),
        "{again}"
    );
}

#[test]
fn a_document_read_again_with_its_print_says_only_that_nothing_changed() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recetas\n\narroz\n");
    let mut talk = served.connect();

    let first = talk.call("read", json!({ "doc": doc }));
    let print = shown(&first)["print"].as_str().unwrap().to_string();
    let again = talk.call("read", json!({ "doc": doc, "print": print }));
    assert_eq!(shown(&again)["unchanged"], true, "{again}");
    assert!(shown(&again).get("body").is_none(), "{again}");
    assert!(text(&again).contains("without `print`"), "{again}");

    served
        .person()
        .saves_doc(&doc, "# Recetas\n\narroz y porotos\n");
    let moved = talk.call("read", json!({ "doc": doc, "print": print }));
    assert_eq!(shown(&moved)["changed"], true, "{moved}");
    assert_eq!(shown(&moved)["body"], "# Recetas\n\narroz y porotos\n");
    assert!(text(&moved).starts_with("(It was written since the print you sent"));
}

#[test]
fn a_part_asked_with_the_print_still_comes_whole() {
    let served = Served::new();
    let doc = served
        .person()
        .writes_doc("# Recetas\n\narroz\n\n## Postres\n\nflan\n");
    let mut talk = served.connect();
    let first = talk.call("read", json!({ "doc": doc, "section": 0 }));
    let print = shown(&first)["print"].as_str().unwrap().to_string();

    for part in [
        json!({ "section": 1 }),
        json!({ "from": 5, "to": 7 }),
        json!({ "chars": 50 }),
        json!({ "marks": true }),
        json!({ "as": "markdown" }),
    ] {
        let mut args = part.clone();
        args["doc"] = json!(doc);
        args["print"] = json!(print);
        let said = talk.call("read", args);
        assert!(shown(&said).get("unchanged").is_none(), "{part}: {said}");
        assert!(shown(&said).get("changed").is_none(), "{part}: {said}");
    }
}

#[test]
fn an_old_print_beside_a_cursor_or_a_first_budget_says_it_changed() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recetas\n\narroz\n");
    let mut talk = served.connect();
    let first = talk.call("read", json!({ "doc": doc }));
    let print = shown(&first)["print"].as_str().unwrap().to_string();
    served
        .person()
        .saves_doc(&doc, "# Recetas\n\narroz y porotos\n");

    for asked in [json!({ "cursor": 2 }), json!({ "chars": 50, "cursor": 1 })] {
        let mut args = asked.clone();
        args["doc"] = json!(doc);
        args["print"] = json!(print);
        let said = talk.call("read", args);
        assert_eq!(shown(&said)["changed"], true, "{asked}: {said}");
    }
}

#[test]
fn a_document_carried_out_with_an_old_print_says_it_changed() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recetas\n\narroz\n");
    let mut talk = served.connect();
    let first = talk.call("read", json!({ "doc": doc }));
    let print = shown(&first)["print"].as_str().unwrap().to_string();
    served
        .person()
        .saves_doc(&doc, "# Recetas\n\narroz y porotos\n");

    let carried = talk.call(
        "read",
        json!({ "doc": doc, "as": "markdown", "print": print }),
    );
    assert_eq!(shown(&carried)["changed"], true, "{carried}");
}

#[test]
fn a_tasks_page_comes_with_the_print_edit_doc_takes() {
    let served = Served::new();
    let mut talk = served.connect();
    let task = proposed(&mut talk, "Plan the week");
    talk.call(
        "append_doc",
        json!({ "doc": task, "body": "Something to keep." }),
    );

    let read = talk.call("read", json!({ "task": task }));
    let page = &shown(&read)["document"];
    let file = page["file"].as_str().unwrap();
    let as_doc = talk.call("read", json!({ "doc": file }));
    assert_eq!(page["print"], shown(&as_doc)["print"], "{read}");
    assert_ne!(page["print"], shown(&read)["print"]);
}

#[test]
fn an_unchanged_document_marked_since_still_says_so() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recetas\n\narroz\n");
    let mut talk = served.connect();
    let first = talk.call("read", json!({ "doc": doc }));
    let print = shown(&first)["print"].as_str().unwrap().to_string();
    talk.call(
        "flag_doc",
        json!({ "doc": doc, "reason": "obsolete", "body": "Nadie cocina arroz ya." }),
    );

    let again = talk.call("read", json!({ "doc": doc, "print": print }));
    assert_eq!(shown(&again)["unchanged"], true, "{again}");
    assert!(
        shown(&again)["notice"]
            .as_str()
            .unwrap()
            .contains("Nadie cocina arroz ya."),
        "{again}"
    );
}

#[test]
fn a_print_carried_on_with_a_cursor_still_joins_the_parts() {
    let served = Served::new();
    let doc = served
        .person()
        .writes_doc("# Recetas\n\nuno\n\ndos\n\ntres\n\ncuatro\n");
    let mut talk = served.connect();
    let first = talk.call("read", json!({ "doc": doc, "chars": 12 }));
    let print = shown(&first)["print"].as_str().unwrap().to_string();
    let next = shown(&first)["next"].as_u64().unwrap();

    let rest = talk.call(
        "read",
        json!({ "doc": doc, "chars": 200, "cursor": next, "print": print }),
    );
    assert!(shown(&rest).get("unchanged").is_none(), "{rest}");
    assert!(shown(&rest)["body"].as_str().unwrap().contains("cuatro"));
}

#[test]
fn marks_bring_the_highlighted_and_bold_passages_by_section() {
    let served = Served::new();
    let doc = served.person().writes_doc(
        "# Plan\n\nThe walk is **cached**.\n\n## Later\n\nA ==pen=={green} line and plain words.\n",
    );
    let mut talk = served.connect();

    let said = talk.call("read", json!({ "doc": doc, "marks": true }));
    assert_eq!(
        shown(&said)["marks"],
        json!([
            { "at": 0, "section": "Plan", "marks": [{ "line": 3, "said": "**cached**" }] },
            { "at": 1, "section": "Later", "marks": [{ "line": 7, "said": "==pen=={green}" }] },
        ]),
        "{said}"
    );
    assert!(shown(&said).get("body").is_none(), "{said}");
    assert!(shown(&said)["print"].is_string(), "{said}");

    let carried = talk.refused(
        "read",
        json!({ "doc": doc, "marks": true, "as": "markdown" }),
    );
    assert!(carried.contains("takes no `marks`"), "{carried}");
    let lines = talk.refused("read", json!({ "doc": doc, "marks": true, "from": 2 }));
    assert!(
        lines.contains("`marks` reads the whole document"),
        "{lines}"
    );
}

#[test]
fn a_projects_parts_say_once_what_every_part_shares() {
    let served = Served::new();
    let mut talk = served.connect();
    let said = talk.call(
        "propose",
        json!({ "title": "Move house", "list": "Personal", "parts": [
            { "title": "Pack the books", "steps": ["one"] },
            { "title": "Book the van", "steps": ["one"] },
        ]}),
    );
    let whole = shown(&said)["id"].as_str().unwrap().to_string();

    let found = talk.call("brain", json!({ "query": "the", "part_of": whole }));
    let each = &shown(&found)["each"]["matches"];
    assert_eq!(each["list"], "Personal", "{found}");
    assert_eq!(each["part_of"], whole, "{found}");
    for row in shown(&found)["matches"].as_array().unwrap() {
        assert!(
            row.get("list").is_none() && row.get("part_of").is_none(),
            "{row}"
        );
        assert!(row["title"].is_string(), "{row}");
    }
}
