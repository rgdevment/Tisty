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

    fn call(&self, name: &str, args: Value) -> Value {
        let root = self.home.path();
        let asked = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args },
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
        serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap()
    }

    fn filed(&self, title: &str) -> String {
        said(&self.call("propose", json!({ "title": title })))["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn paper(&self, body: &str) -> String {
        said(&self.call("write_doc", json!({ "body": body, "anyway": true })))["doc"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn task(&self, id: &str) -> Value {
        said(&self.call("read", json!({ "task": id }))).clone()
    }
}

fn said(answer: &Value) -> &Value {
    &answer["result"]["structuredContent"]
}

fn refused(answer: &Value) -> String {
    assert_eq!(
        answer["result"]["isError"], true,
        "it went through: {answer}"
    );
    answer["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string()
}

fn went(answer: &Value) {
    assert!(answer["result"]["isError"].is_null(), "{answer}");
}

#[test]
fn a_task_an_agent_filed_hangs_under_another_and_is_let_go_again() {
    let served = Served::new();
    let whole = served.filed("Release 2.0");
    let part = served.filed("Write the notes");

    went(&served.call("move", json!({ "task": &part, "part_of": &whole })));
    assert_eq!(served.task(&part)["part_of"], whole.as_str());

    went(&served.call("move", json!({ "task": &part, "part_of": null })));
    assert!(served.task(&part)["part_of"].is_null());
}

#[test]
fn a_task_the_person_opened_hangs_under_one_an_agent_filed() {
    let served = Served::new();
    let whole = served.filed("Release 2.0");
    let persons = served.person().captures("pack the boxes");
    served.person().opens_to_agents(persons);

    went(&served.call(
        "move",
        json!({ "task": persons.to_string(), "part_of": &whole }),
    ));

    assert_eq!(
        served.person().state().tasks[&persons]
            .part_of
            .map(|one| one.to_string()),
        Some(whole)
    );
}

#[test]
fn hanging_is_refused_where_the_window_refuses_it_and_where_the_person_has_not_decided() {
    let served = Served::new();
    let whole = served.filed("Release 2.0");
    let part = served.filed("Write the notes");
    let other = served.filed("Tag the build");
    went(&served.call("move", json!({ "task": &part, "part_of": &whole })));

    let itself = refused(&served.call("move", json!({ "task": &other, "part_of": &other })));
    assert!(itself.contains("itself"), "{itself}");
    let under_a_part = refused(&served.call("move", json!({ "task": &other, "part_of": &part })));
    assert!(under_a_part.contains("one level"), "{under_a_part}");
    let holding = refused(&served.call("move", json!({ "task": &whole, "part_of": &other })));
    assert!(holding.contains("parts of its own"), "{holding}");

    went(&served.call("say_done", json!({ "task": &other, "body": "tagged" })));
    let pending = refused(&served.call("move", json!({ "task": &part, "part_of": &other })));
    assert!(pending.contains("has not decided"), "{pending}");
}

#[test]
fn a_closed_task_and_one_kept_from_agents_take_no_part() {
    let served = Served::new();
    let part = served.filed("Write the notes");
    let closed = served.filed("Old release");
    served.person().closes(person::task(&closed));
    let persons = served.person().captures("the person's plan");

    let shut = refused(&served.call("move", json!({ "task": &part, "part_of": &closed })));
    assert!(shut.contains("closed"), "{shut}");
    let kept = refused(&served.call(
        "move",
        json!({ "task": &part, "part_of": persons.to_string() }),
    ));
    assert!(kept.contains("closed to agents"), "{kept}");
}

#[test]
fn a_list_is_chosen_only_for_what_an_agent_filed() {
    let served = Served::new();
    served.person().adds_list("Work");
    let mine = served.filed("Write the notes");
    let persons = served.person().captures("pack the boxes");
    served.person().opens_to_agents(persons);

    went(&served.call("move", json!({ "task": &mine, "list": "Work" })));
    assert_eq!(served.task(&mine)["list"], "Work");

    let theirs = refused(&served.call(
        "move",
        json!({ "task": persons.to_string(), "list": "Work" }),
    ));
    assert!(theirs.contains("its list is theirs"), "{theirs}");
}

#[test]
fn moving_needs_somewhere_to_move_to() {
    let served = Served::new();
    let mine = served.filed("Write the notes");

    let nowhere = refused(&served.call("move", json!({ "task": &mine })));
    assert!(nowhere.contains("`list`"), "{nowhere}");
}

#[test]
fn open_steps_handed_on_leave_one_task_and_join_the_other() {
    let served = Served::new();
    let from = served.call(
        "propose",
        json!({ "title": "Release", "steps": ["tag it", "write the notes", "ship"] }),
    );
    let from = said(&from)["id"].as_str().unwrap().to_string();
    let to = served.call("propose", json!({ "title": "Docs", "steps": ["outline"] }));
    let to = said(&to)["id"].as_str().unwrap().to_string();

    went(&served.call(
        "unplan",
        json!({ "task": &from, "step": "write the notes", "to": &to }),
    ));

    let texts = |read: &Value| -> Vec<String> {
        read["steps"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one["text"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(texts(&served.task(&from)), ["tag it", "ship"]);
    assert_eq!(texts(&served.task(&to)), ["outline", "write the notes"]);
    let page = served.task(&to)["document"].to_string();
    assert!(page.contains("write the notes"), "the page agrees: {page}");
}

#[test]
fn a_ticked_step_among_those_handed_on_moves_none() {
    let served = Served::new();
    let from = served.call(
        "propose",
        json!({ "title": "Release", "steps": ["tag it", "ship"] }),
    );
    let from = said(&from)["id"].as_str().unwrap().to_string();
    let to = served.filed("Docs");
    went(&served.call("tick", json!({ "task": &from, "step": "tag it" })));

    let ticked = refused(&served.call(
        "unplan",
        json!({ "task": &from, "steps": ["tag it", "ship"], "to": &to }),
    ));
    assert!(ticked.contains("ticked"), "{ticked}");
    assert!(
        served.task(&to)["steps"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );
}

#[test]
fn steps_go_only_to_a_task_this_hand_may_fill() {
    let served = Served::new();
    let from = served.call("propose", json!({ "title": "Release", "steps": ["ship"] }));
    let from = said(&from)["id"].as_str().unwrap().to_string();
    let persons = served.person().captures("the person's plan");

    let theirs = refused(&served.call(
        "unplan",
        json!({ "task": &from, "step": "ship", "to": persons.to_string() }),
    ));
    assert!(theirs.contains("have not opened it"), "{theirs}");
    assert_eq!(served.task(&from)["steps"][0]["text"], "ship");
}

#[test]
fn copies_of_one_thing_are_marked_in_one_call_alike() {
    let served = Served::new();
    let kept = served.paper("# Login\n\nHow it works.");
    let one = served.paper("# Login notes\n\nThe same.");
    let two = served.paper("# Login, again\n\nThe same.");

    let marked = served.call(
        "flag_doc",
        json!({
            "doc": [&one, &two],
            "reason": "duplicate_of",
            "instead": &kept,
            "body": "what Login says already",
        }),
    );
    went(&marked);

    let state = served.person().state();
    let marks: Vec<_> = state
        .docs
        .values()
        .filter_map(|doc| doc.flagged.clone())
        .collect();
    assert_eq!(marks.len(), 2);
    assert_eq!(marks[0].at, marks[1].at, "one click settles the set");
    assert!(
        marks
            .iter()
            .all(|one| one.reason.as_deref() == Some("duplicate_of"))
    );
    assert!(
        marks
            .iter()
            .all(|one| one.instead.as_deref() == Some(kept.as_str()))
    );
}

#[test]
fn in_a_set_one_marked_already_is_skipped_and_named() {
    let served = Served::new();
    let one = served.paper("# Draft one\n\nscratch");
    let two = served.paper("# Draft two\n\nscratch");
    went(&served.call(
        "flag_doc",
        json!({ "doc": &one, "reason": "scratch", "body": "a scratch" }),
    ));

    let again = served.call(
        "flag_doc",
        json!({ "doc": [&one, &two], "reason": "scratch", "body": "scratches" }),
    );
    went(&again);

    assert_eq!(said(&again)["skipped"], json!([one]));
    assert_eq!(said(&again)["doc"], json!([two]));
}

#[test]
fn a_reason_that_points_elsewhere_needs_somewhere_that_exists() {
    let served = Served::new();
    let one = served.paper("# Draft\n\nscratch");

    let nowhere = refused(&served.call(
        "flag_doc",
        json!({ "doc": &one, "reason": "moved", "body": "moved" }),
    ));
    assert!(nowhere.contains("`instead`"), "{nowhere}");
    let missing = refused(&served.call(
        "flag_doc",
        json!({ "doc": &one, "reason": "moved", "instead": "zz-0000", "body": "moved" }),
    ));
    assert!(missing.contains("no document or task"), "{missing}");
    let task = served.filed("Login work");
    went(&served.call(
        "flag_doc",
        json!({ "doc": &one, "reason": "moved", "instead": &task, "body": "moved to the task" }),
    ));
}

#[test]
fn a_reason_not_in_the_list_is_refused() {
    let served = Served::new();
    let one = served.paper("# Draft\n\nscratch");

    let odd = served.call(
        "flag_doc",
        json!({ "doc": &one, "reason": "boring", "body": "dull" }),
    );
    assert_eq!(odd["result"]["isError"], true, "{odd}");
}

#[test]
fn a_document_an_agent_wrote_is_compacted_and_says_so() {
    let served = Served::new();
    let doc = served.paper("# Login\n\nOne.\n\n## Session 1\n\nTwo.\n\n## Session 2\n\nTwo.\n");
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    let done = served.call(
        "write_doc",
        json!({
            "doc": &doc,
            "print": print,
            "compact": true,
            "body": "# Login\n\nOne. Two.\n",
        }),
    );
    went(&done);
    assert!(said(&done)["shrunk"]["after"].as_u64() < said(&done)["shrunk"]["before"].as_u64());
    assert_eq!(said(&done)["dropped"], json!([]));

    let read = said(&served.call("read_doc", json!({ "doc": &doc }))).clone();
    assert!(read["compacted"]["at"].is_string(), "{read}");
    went(&served.call("restore_doc", json!({ "doc": &doc })));
    let back = said(&served.call("read_doc", json!({ "doc": &doc }))).clone();
    assert!(
        back["body"].as_str().unwrap().contains("Session 1"),
        "{back}"
    );
    assert!(
        back["compacted"].is_null(),
        "going back settles the mark: {back}"
    );
}

#[test]
fn a_document_the_person_began_is_compacted_only_when_they_asked() {
    let served = Served::new();
    let doc = served
        .person()
        .writes_doc("# Recipes\n\nSoup.\n\n## Soup\n\nSoup.\n");
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();
    let asked = |asked: bool| {
        json!({
            "doc": &doc,
            "print": print,
            "compact": true,
            "asked": asked,
            "body": "# Recipes\n\n## Soup\n\nSoup.\n",
        })
    };

    let theirs = refused(&served.call("write_doc", asked(false)));
    assert!(theirs.contains("begun by the person"), "{theirs}");
    went(&served.call("write_doc", asked(true)));
}

#[test]
fn compacting_needs_a_document_to_compact() {
    let served = Served::new();

    let nothing = refused(&served.call(
        "write_doc",
        json!({ "body": "# Login\n\nOne.", "compact": true }),
    ));
    assert!(nothing.contains("`doc`"), "{nothing}");
}

#[test]
fn a_new_document_on_a_subject_already_kept_is_refused_naming_it() {
    let served = Served::new();
    let first = served.paper("# Inicio de sesión\n\nCómo funciona.");

    let again = refused(&served.call(
        "write_doc",
        json!({ "body": "# TIS-42 · inicio de sesion\n\nOtra vez." }),
    ));
    assert!(again.contains(&first), "{again}");
    assert!(again.contains("Inicio de sesión"), "{again}");

    went(&served.call(
        "write_doc",
        json!({ "body": "# TIS-42 · inicio de sesion\n\nOtra vez.", "anyway": true }),
    ));
}

#[test]
fn the_assistants_notes_are_written_again_whole_and_can_be_put_back() {
    let served = Served::new();
    let task = served.filed("Login work");
    went(&served.call("append_doc", json!({ "doc": &task, "body": "Tried A." })));
    went(&served.call("append_doc", json!({ "doc": &task, "body": "Tried B." })));

    went(&served.call(
        "append_doc",
        json!({ "doc": &task, "body": "Digest: A and B ruled out.\n- [ ] still open", "replace": true }),
    ));
    let read = served.task(&task);
    let page = read["document"].to_string();
    assert!(page.contains("Digest"), "{page}");
    assert!(!page.contains("Tried A."), "{page}");
    assert!(
        read["steps"].as_array().is_none_or(Vec::is_empty),
        "a box is no step: {read}"
    );

    went(&served.call("restore_doc", json!({ "doc": &task })));
    let back = served.task(&task)["document"].to_string();
    assert!(back.contains("Tried B."), "{back}");
}

#[test]
fn replacing_is_only_for_a_task_s_page() {
    let served = Served::new();
    let doc = served.paper("# Log\n\nOne.");

    let wrong = refused(&served.call(
        "append_doc",
        json!({ "doc": &doc, "body": "Two.", "replace": true }),
    ));
    assert!(wrong.contains("task"), "{wrong}");
}

#[test]
fn the_outline_names_alike_documents_repeated_headings_and_who_began_each_page() {
    let served = Served::new();
    let book = served.paper("# Login\n\n## Tokens\n\nA.\n");
    went(&served.call(
        "write_doc",
        json!({ "body": "# Code review 1\n\n## Tokens\n\nB.\n", "page_of": &book }),
    ));
    let loose = served.paper("# Code review 1\n\nThe same review, loose.");

    let outline = served.call("outline_doc", json!({ "doc": &book }));
    went(&outline);
    let told = said(&outline);
    assert_eq!(told["by"], "agent");
    assert_eq!(told["pages"][0]["by"], "agent");
    assert_eq!(told["alike"][0]["doc"], loose.as_str(), "{told}");
    assert_eq!(told["repeated"][0]["heading"], "Tokens", "{told}");
}

#[test]
fn a_new_document_outside_every_folder_is_pointed_at_the_closest_ones() {
    let served = Served::new();
    served.call("folder", json!({ "name": "Recipes" }));
    served.call("folder", json!({ "name": "Work" }));

    let near = served.call(
        "write_doc",
        json!({ "body": "# Recipes for winter\n\nSoup." }),
    );
    let text = near["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("no folder"), "{text}");
    assert!(text.contains("Recipes") && !text.contains("Work"), "{text}");
    assert!(
        said(&near)["for_the_person"]
            .as_str()
            .unwrap()
            .contains("outside any folder")
    );

    let far = served.call("write_doc", json!({ "body": "# Holidays\n\nBeach." }));
    let text = far["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("no folder's name is near"), "{text}");
    assert!(
        !text.contains("Recipes") && !text.contains("Work"),
        "{text}"
    );
}

#[test]
fn a_task_left_in_the_inbox_is_pointed_at_the_closest_list() {
    let served = Served::new();
    served.person().adds_list("Garden");

    let near = served.call("propose", json!({ "title": "Water the garden" }));
    let text = near["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("closest list by name: Garden"), "{text}");
}

#[test]
fn the_new_tool_and_its_fields_are_listed() {
    let served = Served::new();
    let unknown = served.call("move", json!({ "task": "x", "where": "y" }));
    let said = unknown["result"]["content"][0]["text"].as_str().unwrap();
    assert!(said.contains("part_of") && said.contains("list"), "{said}");
}
