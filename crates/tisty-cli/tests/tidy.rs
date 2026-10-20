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
fn a_part_stays_under_a_whole_kept_closed_to_agents() {
    let served = Served::new();
    let elsewhere = served.filed("Release 2.0");
    let persons = served.person().captures("move house");
    let part = served.person().adds_part(persons, "book the van");
    served.person().opens_to_agents(part);

    let moved = refused(&served.call(
        "move",
        json!({ "task": part.to_string(), "part_of": &elsewhere }),
    ));
    assert!(moved.contains("closed to agents"), "{moved}");
    let let_go =
        refused(&served.call("move", json!({ "task": part.to_string(), "part_of": null })));
    assert!(let_go.contains("closed to agents"), "{let_go}");
    assert_eq!(served.person().state().tasks[&part].part_of, Some(persons));
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
    assert!(said(&done)["size"]["after"].as_u64() < said(&done)["size"]["before"].as_u64());
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
            "person_asked": asked,
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
fn only_open_documents_hold_a_subject_and_pages_never_do() {
    let served = Served::new();
    let put_away = served.paper("# Login\n\nOld.");
    went(&served.call("archive_doc", json!({ "doc": &put_away, "archived": true })));
    went(&served.call("write_doc", json!({ "body": "# Login\n\nNew." })));

    let marked = served.paper("# Billing\n\nOld.");
    went(&served.call(
        "flag_doc",
        json!({ "doc": &marked, "reason": "obsolete", "body": "retired" }),
    ));
    went(&served.call("write_doc", json!({ "body": "# Billing\n\nNew." })));

    let one = served.paper("# Search\n\nA.");
    let two = served.paper("# Export\n\nB.");
    went(&served.call(
        "write_doc",
        json!({ "body": "# Overview\n\nA.", "page_of": &one }),
    ));
    went(&served.call(
        "write_doc",
        json!({ "body": "# Overview\n\nB.", "page_of": &two }),
    ));
}

#[test]
fn a_refused_import_leaves_no_file_behind() {
    let served = Served::new();
    served.paper("# Login\n\nHow it works.");
    let from = tempfile::tempdir_in(std::env::temp_dir()).unwrap();
    std::fs::write(from.path().join("shot.png"), b"\x89PNG\r\n\x1a\n").unwrap();
    std::fs::write(
        from.path().join("login.md"),
        "# Login\n\n![shot](shot.png)\n",
    )
    .unwrap();
    let shelf = served.home.path().join("data").join("attachments");

    let again = refused(&served.call(
        "import_doc",
        json!({ "path": from.path().join("login.md").to_str().unwrap() }),
    ));

    assert!(again.contains("already holds that subject"), "{again}");
    assert!(served.person().state().kept.is_empty());
    assert_eq!(files_under(&shelf), 0);
}

#[test]
fn an_import_refused_once_its_files_are_copied_takes_back_only_those_it_copied() {
    let served = Served::new();
    let from = tempfile::tempdir_in(std::env::temp_dir()).unwrap();
    std::fs::write(from.path().join("shot.png"), b"\x89PNG\r\n\x1a\n").unwrap();
    std::fs::write(from.path().join("fresh.png"), b"\x89PNG\r\n\x1a\n\0").unwrap();
    std::fs::write(
        from.path().join("shots.md"),
        "# Shots\n\n![shot](shot.png)\n",
    )
    .unwrap();
    went(&served.call(
        "import_doc",
        json!({ "path": from.path().join("shots.md").to_str().unwrap() }),
    ));
    let shelf = served.home.path().join("data").join("attachments");
    assert_eq!(files_under(&shelf), 1);
    let docs = served.person().state().docs.len();
    let head = "# Long\n\n![shot](shot.png) ![fresh](fresh.png)\n\n";
    let filler = "a".repeat(64_000 - head.chars().count() - 1);
    std::fs::write(from.path().join("long.md"), format!("{head}{filler}\n")).unwrap();

    let long = refused(&served.call(
        "import_doc",
        json!({ "path": from.path().join("long.md").to_str().unwrap() }),
    ));

    assert!(long.contains("characters"), "{long}");
    assert_eq!(
        files_under(&shelf),
        1,
        "the file kept before stays, the new one goes"
    );
    assert_eq!(served.person().state().kept.len(), 1);
    assert_eq!(served.person().state().docs.len(), docs);
}

#[test]
fn a_compacted_document_takes_no_other_write_until_the_person_decides() {
    let served = Served::new();
    let doc = served.paper("# Login\n\nOne.\n\n## Session 1\n\nTwo.\n\n## Session 2\n\nTwo.\n");
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();
    went(&served.call(
        "write_doc",
        json!({
            "doc": &doc,
            "print": print,
            "compact": true,
            "body": "# Login\n\nOne. Two.\n",
        }),
    ));
    let now = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    let added = refused(&served.call("append_doc", json!({ "doc": &doc, "body": "Three." })));
    let edited = refused(&served.call(
        "edit_doc",
        json!({ "doc": &doc, "old": "One. Two.", "new": "One, two." }),
    ));
    let whole = refused(&served.call(
        "write_doc",
        json!({ "doc": &doc, "print": now, "body": "# Login\n\nOne.\n" }),
    ));
    for told in [&added, &edited, &whole] {
        assert!(told.contains("waits for the person"), "{told}");
    }

    went(&served.call("restore_doc", json!({ "doc": &doc })));
    let back = said(&served.call("read_doc", json!({ "doc": &doc }))).clone();
    assert!(
        back["body"].as_str().unwrap().contains("Session 1"),
        "{back}"
    );
    assert!(back["compacted"].is_null(), "{back}");
    went(&served.call("append_doc", json!({ "doc": &doc, "body": "Three." })));
}

#[test]
fn the_person_keeping_a_compaction_in_the_conversation_lets_the_next_write_through() {
    let served = Served::new();
    let doc = served.paper("# Login\n\nOne.\n\n## Session 1\n\nTwo.\n\n## Session 2\n\nTwo.\n");
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();
    went(&served.call(
        "write_doc",
        json!({
            "doc": &doc,
            "print": print,
            "compact": true,
            "body": "# Login\n\nOne. Two.\n",
        }),
    ));

    went(&served.call(
        "append_doc",
        json!({ "doc": &doc, "body": "Three.", "keep_compaction": true }),
    ));

    let read = said(&served.call("read_doc", json!({ "doc": &doc }))).clone();
    assert!(read["compacted"].is_null(), "{read}");
    assert!(read["body"].as_str().unwrap().contains("Three."), "{read}");
}

#[test]
fn a_compaction_that_loses_a_link_is_refused_naming_it() {
    let served = Served::new();
    let other = served.paper("# Tokens\n\nHow they are kept.\n");
    let doc = served.paper(&format!(
        "# Login\n\nOne, see https://example.com/old and [tokens](tisty:doc/{other}).\n\n## Session 1\n\nTwo.\n\n## Session 2\n\nTwo.\n"
    ));
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    let lost = refused(&served.call(
        "write_doc",
        json!({ "doc": &doc, "print": print, "compact": true, "body": "# Login\n\nOne. Two.\n" }),
    ));

    assert!(lost.contains("nowhere in the new one"), "{lost}");
    assert!(lost.contains("https://example.com/old"), "{lost}");
    assert!(lost.contains(&format!("tisty:doc/{other}")), "{lost}");
    assert!(lost.contains("Nothing was written"), "{lost}");
    let read = said(&served.call("read_doc", json!({ "doc": &doc }))).clone();
    assert!(read["compacted"].is_null(), "{read}");
    assert!(
        read["body"].as_str().unwrap().contains("Session 1"),
        "{read}"
    );
}

#[test]
fn a_compaction_lists_the_links_its_history_only_mentions() {
    let served = Served::new();
    let doc = served.paper(
        "# Login\n\nOne, see https://example.com/old.\n\n## Session 1\n\nTwo.\n\n## Session 2\n\nTwo.\n",
    );
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    let done = served.call(
        "write_doc",
        json!({
            "doc": &doc,
            "print": print,
            "compact": true,
            "body": "# Login\n\nOne. Two.\n\n## History\n\nTook out `https://example.com/old`.\n",
        }),
    );

    went(&done);
    assert_eq!(
        said(&done)["unlinked"],
        json!(["https://example.com/old"]),
        "{done}"
    );
}

#[test]
fn a_compaction_still_the_last_write_is_corrected_and_undoes_to_the_first_body() {
    let served = Served::new();
    let doc = served.paper("# Login\n\nOne.\n\n## Session 1\n\nTwo.\n\n## Session 2\n\nTwo.\n");
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();
    went(&served.call(
        "write_doc",
        json!({ "doc": &doc, "print": print, "compact": true, "body": "# Login\n\nOne. Two.\n" }),
    ));
    let now = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    went(&served.call(
        "write_doc",
        json!({ "doc": &doc, "print": now, "compact": true, "body": "# Login\n\nOne, two.\n" }),
    ));
    let read = said(&served.call("read_doc", json!({ "doc": &doc }))).clone();
    assert!(read["compacted"]["at"].is_string(), "{read}");
    assert!(
        read["body"].as_str().unwrap().contains("One, two."),
        "{read}"
    );

    went(&served.call("restore_doc", json!({ "doc": &doc })));
    let back = said(&served.call("read_doc", json!({ "doc": &doc }))).clone();
    let body = back["body"].as_str().unwrap();
    assert!(body.contains("Session 1"), "{back}");
    assert!(!body.contains("One. Two."), "{back}");
    assert!(back["compacted"].is_null(), "{back}");
}

#[test]
fn a_compaction_written_over_since_is_not_compacted_again() {
    let served = Served::new();
    let doc = served.paper("# Login\n\nOne.\n\n## Session 1\n\nTwo.\n\n## Session 2\n\nTwo.\n");
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();
    went(&served.call(
        "write_doc",
        json!({ "doc": &doc, "print": print, "compact": true, "body": "# Login\n\nOne. Two.\n" }),
    ));
    served
        .person()
        .saves_doc(&doc, "# Login\n\nOne. Two. Three.\n");
    let now = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    let again = refused(&served.call(
        "write_doc",
        json!({ "doc": &doc, "print": now, "compact": true, "body": "# Login\n\nOne.\n" }),
    ));
    assert!(again.contains("waits for the person"), "{again}");
}

#[test]
fn a_compaction_that_grows_the_body_says_so() {
    let served = Served::new();
    let doc = served.paper("# Login\n\nOne.\n");
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    let done = served.call(
        "write_doc",
        json!({
            "doc": &doc,
            "print": print,
            "compact": true,
            "body": "# Login\n\nOne, and a good deal more than there was.\n",
        }),
    );

    went(&done);
    let text = done["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("The body grew from"), "{text}");
    assert!(text.contains("say why to the person"), "{text}");
}

#[test]
fn the_outline_names_links_that_lead_nowhere() {
    let served = Served::new();
    let doc = served.paper("# Login\n\nSee [gone](tisty:doc/zzzzzzzz-9999).\n");

    let outline = served.call("outline_doc", json!({ "doc": &doc }));
    went(&outline);
    let told = said(&outline);
    assert_eq!(told["dead"][0]["to"], "tisty:doc/zzzzzzzz-9999", "{told}");
    assert_eq!(told["dead"][0]["in"], doc.as_str(), "{told}");
    let text = outline["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("Links that lead nowhere (1)"), "{text}");
}

#[test]
fn links_to_nothing_or_to_documents_marked_now_need_no_line_in_the_history() {
    let served = Served::new();
    let old = served.paper("# Old plan\n\nDone long ago.");
    let doc = served.paper(&format!(
        "# Login\n\nOne.\n\n[the plan](tisty:doc/{old})\n\n[gone](tisty:doc/zzzzzzzz-0099)\n\n## Session 1\n\nTwo.\n"
    ));
    went(&served.call(
        "flag_doc",
        json!({ "doc": &old, "reason": "obsolete", "body": "Its branch was merged." }),
    ));
    let print = said(&served.call("read_doc", json!({ "doc": &doc })))["print"].clone();

    went(&served.call(
        "write_doc",
        json!({ "doc": &doc, "print": print, "compact": true, "body": "# Login\n\nOne. Two.\n" }),
    ));
}

fn files_under(at: &std::path::Path) -> usize {
    std::fs::read_dir(at).map_or(0, |all| {
        all.flatten()
            .map(|one| match one.path().is_dir() {
                true => files_under(&one.path()),
                false => 1,
            })
            .sum()
    })
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
    assert!(text.contains("nothing kept in one reads like it"), "{text}");
    assert!(
        !text.contains("Recipes") && !text.contains("Work"),
        "{text}"
    );
}

#[test]
fn a_new_document_is_pointed_at_the_folder_whose_documents_read_like_it() {
    let served = Served::new();
    served.call("folder", json!({ "name": "Projects" }));
    served.call("folder", json!({ "name": "Kitchen" }));
    for body in [
        "# Plan: attachments in one place

Where they live. #attachments",
        "# Plan for the attachments bin

How they are let go. #attachments",
    ] {
        let doc = served.paper(body);
        served.call("file_doc", json!({ "doc": doc, "folder": "Projects" }));
    }

    let loose = served.call(
        "write_doc",
        json!({
            "body": "# Plan: join duplicate attachments

Keep one copy. #attachments",
            "anyway": true
        }),
    );

    let text = loose["result"]["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("closest to what these hold"), "{text}");
    assert!(
        text.contains("Projects (2 alike)") && !text.contains("Kitchen"),
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
