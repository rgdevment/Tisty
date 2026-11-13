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
fn an_assistant_writes_a_task_s_page_with_the_document_tools_but_never_files_it() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "The person's own words.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let body = || {
        let read = served.call("read_doc", serde_json::json!({ "doc": file }));
        read["result"]["structuredContent"]["body"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };

    let edited = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "own", "new": "changed" }),
    );
    assert_ne!(edited["result"]["isError"], true, "{edited}");
    let padded = served.call(
        "append_doc",
        serde_json::json!({ "doc": format!(" {file}\t"), "body": "Added at the end." }),
    );
    assert_ne!(padded["result"]["isError"], true, "{padded}");
    let now = body();
    assert!(now.contains("The person's changed words."), "{now}");
    assert!(now.contains("Added at the end."), "{now}");

    let archived = served.call("archive_doc", serde_json::json!({ "doc": file }));
    assert_eq!(archived["result"]["isError"], true, "{archived}");
    let said = archived["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("task's page"), "{said}");
}

#[test]
fn a_page_that_loses_the_lines_naming_its_task_is_put_back() {
    let served = Served::new();
    let filed = served.call("propose", serde_json::json!({ "title": "Rina's booster" }));
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "The person's own words.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let read = served.call("read_doc", serde_json::json!({ "doc": file }));
    let before = read["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap()
        .to_string();
    let print = read["result"]["structuredContent"]["print"].clone();

    let refused = served.call(
        "write_doc",
        serde_json::json!({ "doc": file, "print": print, "body": "# Rina\n\nAll new words." }),
    );

    assert_eq!(refused["result"]["isError"], true, "{refused}");
    let after = served.call("read_doc", serde_json::json!({ "doc": file }));
    assert_eq!(
        after["result"]["structuredContent"]["body"].as_str(),
        Some(before.as_str()),
        "{after}"
    );
}

#[test]
fn a_box_written_on_the_persons_page_is_a_step_and_its_tick_waits_for_them() {
    let served = Served::new();
    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    let task = found["result"]["structuredContent"]["matches"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let edited = served.call(
        "edit_doc",
        serde_json::json!({
            "doc": file,
            "old": "Mis palabras.",
            "new": "Mis palabras.\n\n- [ ] pagar\n- [x] llamar"
        }),
    );
    assert_ne!(edited["result"]["isError"], true, "{edited}");
    let told = edited["result"]["structuredContent"]["said"]
        .as_str()
        .unwrap_or_default();
    assert!(told.contains("Its boxes went"), "{edited}");

    let read = served.call("read", serde_json::json!({ "task": task }));
    let steps = read["result"]["structuredContent"]["steps"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let texts: Vec<&str> = steps
        .iter()
        .filter_map(|one| one["text"].as_str())
        .collect();
    assert_eq!(texts, ["pagar", "llamar"], "{read}");
    assert!(
        steps.iter().all(|one| one["done"] == false),
        "a tick on the person's task is only the assistant's word: {read}"
    );
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
fn the_persons_task_takes_assistant_notes_on_its_page() {
    let served = Served::new();
    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    let task = found["result"]["structuredContent"]["matches"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let added = served.call(
        "append_doc",
        serde_json::json!({ "doc": task, "body": "Found the receipt." }),
    );

    assert_ne!(added["result"]["isError"], true, "{added}");
    let said = added["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("assistant's notes"), "{said}");
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

#[test]
fn putting_the_persons_page_back_undoes_the_assistants_edit() {
    let served = Served::new();
    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    let task = found["result"]["structuredContent"]["matches"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let body = || {
        let read = served.call("read_doc", serde_json::json!({ "doc": file }));
        read["result"]["structuredContent"]["body"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    };
    let before = body();

    let edited = served.call(
        "edit_doc",
        serde_json::json!({
            "doc": file,
            "old": "Mis palabras.",
            "new": "Mis palabras.\n\n- [x] llamar"
        }),
    );
    assert_ne!(edited["result"]["isError"], true, "{edited}");
    let restored = served.call("restore_doc", serde_json::json!({ "doc": file }));
    assert_ne!(restored["result"]["isError"], true, "{restored}");

    assert_eq!(body(), before);
}

fn persons_task(served: &Served) -> String {
    let found = served.call("find", serde_json::json!({ "query": "algo mio" }));
    found["result"]["structuredContent"]["matches"][0]["id"]
        .as_str()
        .unwrap()
        .to_string()
}

fn page_of(served: &Served, file: &str) -> serde_json::Value {
    served.call("read_doc", serde_json::json!({ "doc": file }))["result"]["structuredContent"]
        .clone()
}

#[test]
fn a_page_behind_its_steps_takes_an_edit_without_touching_them() {
    let served = Served::new();
    let task = persons_task(&served);
    let person = served.person();
    person.adds_step(person::task(&task), "pagar");
    person.adds_step(person::task(&task), "llamar");
    person.ticks(person::task(&task), 1);
    served.written_for(&task, "Mis palabras.\n\n- [ ] pagar");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let edited = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Mis palabras.", "new": "Mis palabras, corregidas." }),
    );

    assert_ne!(edited["result"]["isError"], true, "{edited}");
    let read = served.call("read", serde_json::json!({ "task": task }));
    let steps = read["result"]["structuredContent"]["steps"].clone();
    assert_eq!(steps[0]["text"], "pagar", "{read}");
    assert_eq!(
        steps[0]["done"], true,
        "the stale box took no tick away: {read}"
    );
    assert_eq!(
        steps[1]["text"], "llamar",
        "a step with no line stays: {read}"
    );
}

#[test]
fn a_refused_page_write_keeps_the_step_back_from_before_it() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let first = page_of(&served, &file)["body"]
        .as_str()
        .unwrap()
        .to_string();
    served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Mis palabras.", "new": "Otras palabras." }),
    );
    let print = page_of(&served, &file)["print"].clone();

    let refused = served.call(
        "write_doc",
        serde_json::json!({ "doc": file, "print": print, "body": "Sin cabecera." }),
    );
    assert_eq!(refused["result"]["isError"], true, "{refused}");
    let restored = served.call("restore_doc", serde_json::json!({ "doc": file }));

    assert_ne!(restored["result"]["isError"], true, "{restored}");
    assert_eq!(
        page_of(&served, &file)["body"].as_str(),
        Some(first.as_str())
    );
}

#[test]
fn a_tick_on_a_copy_of_the_persons_page_is_not_the_assistants_word() {
    let served = Served::new();
    let task = persons_task(&served);
    let person = served.person();
    person.adds_step(person::task(&task), "pagar");
    served.written_for(&task, "Mis palabras.\n\n- [ ] pagar");
    let read = served.call("read", serde_json::json!({ "task": task }));
    let copy = read["result"]["structuredContent"]["document"]["body"]
        .as_str()
        .unwrap()
        .replace("- [ ] pagar", "- [x] pagar");
    let at = served.home.path().join("copy.md");
    std::fs::write(&at, copy).unwrap();

    let brought = served.call(
        "import_doc",
        serde_json::json!({ "path": at.to_string_lossy() }),
    );

    assert_ne!(brought["result"]["isError"], true, "{brought}");
    let read = served.call("read", serde_json::json!({ "task": task }));
    let step = &read["result"]["structuredContent"]["steps"][0];
    assert_eq!(step["done"], false, "{read}");
    assert!(step["said_done"].is_null(), "{read}");
}

fn on_disk(served: &Served, file: &str) -> String {
    tisty_core::docs::read(&served.home.path().join("data").join("docs"), file).unwrap()
}

#[test]
fn the_page_of_a_task_the_person_hid_is_not_written_by_its_name() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let before = on_disk(&served, &file);
    served.person().hides(person::task(&task));

    let edited = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Mis palabras.", "new": "- [ ] pagar" }),
    );
    let added = served.call(
        "append_doc",
        serde_json::json!({ "doc": file, "body": "Algo más." }),
    );

    let by_id = served.call(
        "append_doc",
        serde_json::json!({ "doc": task, "body": "Algo más." }),
    );

    served.person().closes(person::task(&task));
    let closed = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Mis palabras.", "new": "Otras." }),
    );

    for refused in [&edited, &added, &by_id, &closed] {
        assert_eq!(refused["result"]["isError"], true, "{refused}");
        let said = refused["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            !said.contains("algo mio"),
            "a hidden task was named: {said}"
        );
    }
    assert_eq!(on_disk(&served, &file), before);
}

#[test]
fn a_written_page_is_named_by_its_task_never_by_its_first_section() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "## Description\n\nMis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let edited = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Mis palabras.", "new": "Mis palabras, cambiadas." }),
    );

    let said = &edited["result"]["structuredContent"];
    assert_eq!(said["title"], "algo mio", "{edited}");
    assert_eq!(
        said["for_the_person"], "the page of the task \"algo mio\", in the inbox",
        "{edited}"
    );
    let told = edited["result"]["content"][0]["text"].as_str().unwrap();
    assert!(told.contains("in the page of \"algo mio\""), "{told}");
    assert!(!told.contains("\"Description\""), "{told}");

    let added = served.call(
        "append_doc",
        serde_json::json!({ "doc": file, "body": "Algo más.", "under": "Description" }),
    );
    let told = added["result"]["content"][0]["text"].as_str().unwrap();
    assert!(
        told.contains("under \"Description\" in the page of \"algo mio\""),
        "{told}"
    );
    assert_eq!(added["result"]["structuredContent"]["title"], "algo mio");

    let print = page_of(&served, &file)["print"].clone();
    let whole = format!(
        "{}\n## Description\n\n| Cosa | Hecha |\n| --- | --- |\n| pan | no |\n",
        tisty_core::task_docs::seed(person::task(&task))
    );
    let written = served.call(
        "write_doc",
        serde_json::json!({ "doc": file, "print": print, "body": whole }),
    );
    let tabled = served.call(
        "edit_table",
        serde_json::json!({ "doc": file, "do": "add", "values": { "Cosa": "sal" } }),
    );
    let restored = served.call("restore_doc", serde_json::json!({ "doc": file }));
    for (said, starts) in [
        (&written, "Wrote the page of \"algo mio\" again"),
        (&tabled, "Changed the table 1 in the page of \"algo mio\""),
        (&restored, ""),
    ] {
        assert_ne!(said["result"]["isError"], true, "{said}");
        let told = said["result"]["content"][0]["text"].as_str().unwrap();
        assert!(told.starts_with(starts), "{told}");
        assert!(!told.contains("\"Description\""), "{told}");
    }
}

#[test]
fn a_locked_page_is_refused_by_its_task_s_name() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "## Description\n\nMis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    served.person().locks_doc(&file);

    let edited = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Mis palabras.", "new": "Otras." }),
    );

    assert_eq!(edited["result"]["isError"], true, "{edited}");
    let said = edited["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("the page of \"algo mio\""), "{said}");
    assert!(!said.contains("\"Description\""), "{said}");
}

#[test]
fn a_property_written_into_a_page_s_front_matter_is_put_back_and_says_where_it_goes() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let before = on_disk(&served, &file);
    let line = format!("task: {task}");

    let dated = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": line, "new": format!("{line}\nscheduled: 2026-12-01") }),
    );

    assert_eq!(dated["result"]["isError"], true, "{dated}");
    let said = dated["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("which tool changes each"), "{said}");
    assert_eq!(on_disk(&served, &file), before);
}

#[test]
fn notes_written_on_a_page_by_its_name_are_kept_as_the_assistant_s() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(
        &task,
        "Mis palabras.\n\n## Assistant notes\n\nUna línea mía.",
    );
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let added = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Una línea mía.", "new": "Una línea mía.\n\nFound the receipt." }),
    );

    assert_ne!(added["result"]["isError"], true, "{added}");
    let left = tisty_core::docs::read_notes_left(&served.home.path().join("data"), &file).unwrap();
    assert!(left.contains("Found the receipt."), "{left}");
    assert!(!left.contains("Una línea mía."), "{left}");
}

fn text_of(said: &serde_json::Value) -> String {
    said["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn no_tool_names_or_writes_a_task_the_person_hid() {
    let served = Served::new();
    let filed = served.call(
        "propose",
        serde_json::json!({
            "title": "secret surprise for Ana",
            "source": "mail:42",
            "date": "2026-12-01",
            "parts": [{ "title": "book the restaurant" }]
        }),
    );
    let said = &filed["result"]["structuredContent"];
    let task = said["id"].as_str().unwrap().to_string();
    let part = said["parts"][0]["id"].as_str().unwrap().to_string();
    let doc = served.person().writes_doc("# Planes\n\nAlgo.\n");
    served.person().hides(person::task(&task));

    let refused = [
        (
            "note",
            serde_json::json!({ "task": task, "body": "found it" }),
        ),
        (
            "reschedule",
            serde_json::json!({ "task": task, "date": "2026-12-02" }),
        ),
        (
            "say_done",
            serde_json::json!({ "task": task, "body": "done" }),
        ),
        (
            "say_not_doing",
            serde_json::json!({ "task": task, "body": "no" }),
        ),
        (
            "flag_doc",
            serde_json::json!({ "doc": doc, "reason": "moved", "instead": task, "body": "x" }),
        ),
    ];
    for (name, args) in refused {
        let answer = served.call(name, args);
        assert_eq!(answer["result"]["isError"], true, "{name}: {answer}");
        assert!(
            !answer.to_string().contains("secret surprise"),
            "{name}: {answer}"
        );
    }
    let found = served.call("find", serde_json::json!({ "source": "mail:42" }));
    assert_eq!(
        found["result"]["structuredContent"]["found"]["hidden"], true,
        "{found}"
    );
    for again in [false, true] {
        let proposed = served.call(
            "propose",
            serde_json::json!({ "title": "otra", "source": "mail:42", "again": again }),
        );
        assert_eq!(
            proposed["result"]["structuredContent"]["proposed"], false,
            "{proposed}"
        );
        assert!(
            !proposed.to_string().contains("secret surprise"),
            "{proposed}"
        );
    }
    let moved = served.call("move", serde_json::json!({ "task": part, "part_of": null }));
    assert_ne!(moved["result"]["isError"], true, "{moved}");
    assert!(!moved.to_string().contains("secret surprise"), "{moved}");
    assert!(text_of(&moved).contains("go from its whole"), "{moved}");
    for answer in [&found, &moved] {
        assert!(!answer.to_string().contains("secret surprise"), "{answer}");
    }
    let state = served.person().state();
    assert!(state.tasks[&person::task(&part)].part_of.is_none());
    let hidden = &state.tasks[&person::task(&task)];
    assert!(hidden.log.is_empty(), "a note reached a hidden task");
    assert_eq!(
        hidden.date.as_ref().map(|day| day.date().to_string()),
        Some("2026-12-01".to_string()),
        "its day moved"
    );
}

#[test]
fn a_person_s_task_page_an_agent_made_is_still_theirs_to_compact() {
    let served = Served::new();
    let task = persons_task(&served);
    let added = served.call(
        "append_doc",
        serde_json::json!({ "doc": task, "body": "Found the receipt." }),
    );
    assert_ne!(added["result"]["isError"], true, "{added}");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let page = page_of(&served, &file);
    let body = format!(
        "{}\nShorter.\n",
        tisty_core::task_docs::seed(person::task(&task))
    );

    let compacted = served.call(
        "write_doc",
        serde_json::json!({ "doc": file, "print": page["print"], "body": body, "compact": true }),
    );

    assert_eq!(compacted["result"]["isError"], true, "{compacted}");
    assert!(text_of(&compacted).contains("person_asked"), "{compacted}");
    let asked = served.call(
        "write_doc",
        serde_json::json!({
            "doc": file, "print": page["print"], "body": body, "compact": true, "person_asked": true
        }),
    );
    assert_ne!(asked["result"]["isError"], true, "{asked}");
    assert!(!text_of(&asked).contains("folder"), "{asked}");
}

#[test]
fn a_box_ticked_on_the_persons_page_answers_with_the_print_it_reads_at() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let edited = served.call(
        "edit_doc",
        serde_json::json!({ "doc": file, "old": "Mis palabras.", "new": "Mis palabras.\n\n- [x] llamar" }),
    );

    assert_ne!(edited["result"]["isError"], true, "{edited}");
    let now = on_disk(&served, &file);
    assert_eq!(
        edited["result"]["structuredContent"]["print"],
        tisty_core::attach::printed(now.as_bytes()),
        "{now}"
    );
}

#[test]
fn every_read_of_a_page_names_it_by_its_task() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "## Description\n\nMis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let read = served.call("read_doc", serde_json::json!({ "doc": file }));
    let outlined = served.call("outline_doc", serde_json::json!({ "doc": file }));
    let summed = served.call(
        "sum_up",
        serde_json::json!({ "doc": file, "summary": "Palabras de la persona." }),
    );
    let tabled = served.call("edit_table", serde_json::json!({ "doc": file }));
    let by_id = served.call("read_doc", serde_json::json!({ "doc": task }));

    assert_eq!(read["result"]["structuredContent"]["title"], "algo mio");
    assert_eq!(outlined["result"]["structuredContent"]["title"], "algo mio");
    assert!(
        text_of(&summed).contains("the page of \"algo mio\""),
        "{summed}"
    );
    assert!(
        text_of(&tabled).contains("the page of \"algo mio\""),
        "{tabled}"
    );
    assert!(text_of(&by_id).contains(&file), "{by_id}");
}

#[test]
fn no_read_of_a_long_document_runs_past_its_caps() {
    let served = Served::new();
    let log: String = (1..=300)
        .map(|n| format!("## Entrada {n}\n\nPasó algo el día {n}.\n\n"))
        .collect();
    let long = format!(
        "# Bitácora\n\n{log}{} aguja {}\n",
        "palabra ".repeat(3_000),
        "palabra ".repeat(3_000)
    );
    let doc = served.person().writes_doc(&long);

    let outlined = served.call("outline_doc", serde_json::json!({ "doc": doc }));
    let whole = served.call("read_doc", serde_json::json!({ "doc": doc }));
    let chars = served.call(
        "read_doc",
        serde_json::json!({ "doc": doc, "chars": 200_000 }),
    );
    let found = served.call("find", serde_json::json!({ "doc": doc, "query": "aguja" }));

    for answer in [&outlined, &whole] {
        let said = &answer["result"]["structuredContent"];
        assert!(said["outline"].as_array().unwrap().len() <= 120, "{answer}");
        assert!(said["outline_left"].as_u64().unwrap() > 0, "{answer}");
    }
    let part = chars["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap();
    assert!(
        part.chars().count() < long.chars().count() / 2,
        "chars ran on"
    );
    let hit = found["result"]["structuredContent"]["lines"][0]["text"]
        .as_str()
        .unwrap_or_default();
    assert!(
        hit.contains("aguja") && hit.chars().count() <= 1_000,
        "{found}"
    );
}

#[test]
fn the_small_answers_about_a_task_say_what_truly_happened() {
    let served = Served::new();
    let filed = served.call(
        "propose",
        serde_json::json!({ "title": "Rina's booster", "date": "2026-12-01", "steps": ["Call the vet"] }),
    );
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let same = served.call(
        "reschedule",
        serde_json::json!({ "task": task, "date": "2026-12-01" }),
    );
    assert_eq!(
        same["result"]["structuredContent"]["moved"], false,
        "{same}"
    );
    assert!(text_of(&same).contains("nothing changed"), "{same}");

    let persons = persons_task(&served);
    served.person().adds_step(person::task(&persons), "pagar");
    served.call(
        "tick",
        serde_json::json!({ "task": persons, "steps": ["pagar"] }),
    );
    let reworded = served.call(
        "reword_step",
        serde_json::json!({ "task": persons, "step": "pagar", "text": "pagar la cuenta" }),
    );
    assert!(text_of(&reworded).contains("`tick` it again"), "{reworded}");
}

#[test]
fn a_fresh_page_keeps_no_step_back_that_would_take_the_note_off() {
    let served = Served::new();
    let task = persons_task(&served);
    let added = served.call(
        "append_doc",
        serde_json::json!({ "doc": task, "body": "Found the receipt." }),
    );
    assert_ne!(added["result"]["isError"], true, "{added}");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());

    let restored = served.call("restore_doc", serde_json::json!({ "doc": file }));

    assert_eq!(restored["result"]["isError"], true, "{restored}");
    assert!(
        text_of(&restored).contains("nothing is kept beside"),
        "{restored}"
    );
    assert!(on_disk(&served, &file).contains("Found the receipt."));
}

#[test]
fn a_closed_task_s_page_says_it_went_with_its_task() {
    let served = Served::new();
    let task = persons_task(&served);
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    served.person().closes(person::task(&task));

    let read = served.call("read_doc", serde_json::json!({ "doc": file }));

    assert!(text_of(&read).contains("closed task"), "{read}");
}

#[test]
fn a_hidden_task_s_page_and_tags_never_reach_another_answer() {
    let served = Served::new();
    let filed = served.call(
        "propose",
        serde_json::json!({ "title": "secret zebra plan", "tags": ["zebratag"] }),
    );
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let doc = served.person().writes_doc("# Planes\n\nAlgo.\n");
    served.written_for(&task, &format!("Ver [Planes](tisty:doc/{doc})."));
    let page = format!("t_{}-0001", task.to_ascii_lowercase());
    served.person().hides(person::task(&task));

    let outlined = served.call("outline_doc", serde_json::json!({ "doc": doc }));
    let flagged = served.call(
        "flag_doc",
        serde_json::json!({ "doc": doc, "reason": "moved", "instead": page, "body": "x" }),
    );
    let caught = served.call("catch_up", serde_json::json!({}));

    for answer in [&outlined, &flagged, &caught] {
        assert!(!answer.to_string().contains("secret zebra"), "{answer}");
        assert!(!answer.to_string().contains("zebratag"), "{answer}");
    }
    assert_eq!(flagged["result"]["isError"], true, "{flagged}");
}

#[test]
fn the_page_read_hands_back_is_written_back_as_it_came() {
    let served = Served::new();
    let filed = served.call(
        "propose",
        serde_json::json!({ "title": "Rina's booster", "date": "2026-12-09", "tags": ["vet"] }),
    );
    let task = filed["result"]["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.written_for(&task, "Mis palabras.");
    let file = format!("t_{}-0001", task.to_ascii_lowercase());
    let read = served.call("read", serde_json::json!({ "task": task }));
    let body = read["result"]["structuredContent"]["document"]["body"]
        .as_str()
        .unwrap()
        .replace("Mis palabras.", "Mis palabras, cambiadas.");
    let print = page_of(&served, &file)["print"].clone();

    let written = served.call(
        "write_doc",
        serde_json::json!({ "doc": file, "print": print, "body": body }),
    );

    assert_ne!(written["result"]["isError"], true, "{written}");
    assert!(on_disk(&served, &file).contains("cambiadas"));
}

#[test]
fn a_closed_task_s_refusal_asks_for_its_title_and_day_never_its_id() {
    let served = Served::new();
    let task = persons_task(&served);
    served.person().closes(person::task(&task));

    let noted = served.call("note", serde_json::json!({ "task": task, "body": "algo" }));

    let said = text_of(&noted);
    assert!(said.contains("the day it ended"), "{said}");
    assert!(!said.contains(&format!("its id {task}")), "{said}");
}

#[test]
fn a_long_log_s_outline_keeps_its_days_and_a_long_line_its_cap() {
    let served = Served::new();
    let days: String = (1..=300)
        .map(|n| format!("## Día {n}\n\nNada más.\n\n"))
        .collect();
    let log = served.person().writes_doc(&format!("# Bitácora\n\n{days}"));
    let wide = served
        .person()
        .writes_doc(&format!("# Ancho\n\n{}\n", "palabra ".repeat(20_000)));

    let outlined = served.call("outline_doc", serde_json::json!({ "doc": log }));
    let line = served.call(
        "read_doc",
        serde_json::json!({ "doc": wide, "from": 3, "to": 3 }),
    );

    let held = outlined["result"]["structuredContent"]["outline"]
        .as_array()
        .unwrap()
        .len();
    assert!(held > 100 && held <= 120, "{held}");
    let part = line["result"]["structuredContent"]["body"]
        .as_str()
        .unwrap();
    assert!(part.chars().count() <= 12_001, "{}", part.chars().count());
}

#[test]
fn an_outline_keeps_every_chapter_and_a_cut_line_says_so() {
    let served = Served::new();
    let chapters: String = (1..=10)
        .map(|chapter| {
            let days: String = (1..=30)
                .map(|day| format!("### Día {chapter}-{day}\n\nNada.\n\n"))
                .collect();
            format!("## Capítulo {chapter}\n\n{days}")
        })
        .collect();
    let book = served
        .person()
        .writes_doc(&format!("# Libro\n\n{chapters}"));
    let wide = served
        .person()
        .writes_doc(&format!("# Ancho\n\n{}\n", "palabra ".repeat(20_000)));

    let outlined = served.call("outline_doc", serde_json::json!({ "doc": book }));
    let line = served.call(
        "read_doc",
        serde_json::json!({ "doc": wide, "from": 3, "to": 3 }),
    );

    let titles: Vec<String> = outlined["result"]["structuredContent"]["outline"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|one| one["title"].as_str().map(str::to_string))
        .collect();
    for chapter in 1..=10 {
        assert!(
            titles.contains(&format!("Capítulo {chapter}")),
            "chapter {chapter} left out: {titles:?}"
        );
    }
    assert_eq!(line["result"]["structuredContent"]["cut"], true, "{line}");
    assert!(text_of(&line).contains("runs past"), "{line}");
}
