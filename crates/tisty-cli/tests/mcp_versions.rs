use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

const CLAUDE: &str = "claude-code";
const CODEX: &str = "codex";
const UNO: &str = "# Acta\n\nUno, como lo dejó el asistente.\n";
const DOS: &str = "# Acta\n\nDos, como lo dejó la persona.\n";

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

    fn paths(&self) -> tisty_core::Paths {
        let root = self.home.path();
        tisty_core::Paths::new(root.join("data"), root.join("config"))
    }

    fn call(&self, name: &str, args: Value) -> Value {
        self.call_as(CLAUDE, name, args)
    }

    fn call_as(&self, client: &str, name: &str, args: Value) -> Value {
        self.answered(client, name, args)["result"].clone()
    }

    fn answered(&self, client: &str, name: &str, args: Value) -> Value {
        let root = self.home.path();
        let hello = json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": { "protocolVersion": "2025-06-18", "capabilities": {},
                        "clientInfo": { "name": client, "version": "1" } },
        });
        let asked = json!({
            "jsonrpc": "2.0", "id": 2, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        });
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
        writeln!(child.stdin.take().unwrap(), "{hello}\n{asked}").unwrap();
        let out = child.wait_with_output().unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .find(|one| one["id"] == 2)
            .unwrap()
    }

    fn wrote(&self, body: &str) -> String {
        let made = self.call("write_doc", json!({ "body": body, "anyway": true }));
        made["structuredContent"]["doc"]
            .as_str()
            .unwrap_or_else(|| panic!("{made}"))
            .to_string()
    }

    fn read(&self, doc: &str) -> Value {
        self.call("read_doc", json!({ "doc": doc }))["structuredContent"].clone()
    }

    fn body_of(&self, doc: &str) -> String {
        self.read(doc)["body"].as_str().unwrap().to_string()
    }

    fn edits(&self, doc: &str, old: &str, new: &str) {
        went(&self.call("edit_doc", json!({ "doc": doc, "old": old, "new": new })));
    }

    fn versions(&self, doc: &str) -> Value {
        let said = self.call("read_doc", json!({ "doc": doc, "versions": true }));
        went(&said);
        said["structuredContent"].clone()
    }

    fn version_that_said(&self, doc: &str, words: &str) -> String {
        let listed = self.versions(doc);
        listed["versions"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row.get("elsewhere").is_none())
            .filter_map(|row| row["version"].as_str())
            .find(|version| {
                let read = self.call("read_doc", json!({ "doc": doc, "version": version }));
                read["structuredContent"]["body"]
                    .as_str()
                    .is_some_and(|body| body.contains(words))
            })
            .unwrap_or_else(|| panic!("no version says {words:?}: {listed}"))
            .to_string()
    }

    // The assistant, then the person, then the assistant: each writes over the other, so each text before is kept.
    fn written_by_turns(&self) -> String {
        let doc = self.wrote(UNO);
        self.person().rewrites_doc(&doc, DOS);
        self.edits(
            &doc,
            "Dos, como lo dejó la persona.",
            "Tres, otra vez el asistente.",
        );
        doc
    }

    fn task_with_a_page(&self) -> (String, String) {
        let made = self.call(
            "propose",
            json!({ "title": "Renovar el pasaporte", "list": "Personal", "description": "Pedir hora." }),
        );
        let task = made["structuredContent"]["id"]
            .as_str()
            .unwrap_or_else(|| panic!("{made}"))
            .to_string();
        went(&self.call("append_doc", json!({ "doc": task, "body": "Nota uno." })));
        went(&self.call_as(
            CODEX,
            "append_doc",
            json!({ "doc": task, "body": "Nota dos." }),
        ));
        let read = self.call("read", json!({ "task": task }));
        let page = read["structuredContent"]["document"]["file"]
            .as_str()
            .unwrap_or_else(|| panic!("{read}"))
            .to_string();
        (task, page)
    }
}

fn refused(said: &Value) -> bool {
    said["isError"] == json!(true)
}

fn went(said: &Value) {
    assert!(!refused(said), "{said}");
}

fn words(said: &Value) -> String {
    said["content"][0]["text"].as_str().unwrap().to_string()
}

fn volatile(at: &Value, found: &mut Vec<(String, String)>) {
    match at {
        Value::Object(fields) => {
            for (key, one) in fields {
                let label = match (key.as_str(), one.as_str()) {
                    ("version", Some(_)) => {
                        let seen = found
                            .iter()
                            .filter(|(_, l)| l.starts_with("[version"))
                            .count();
                        Some(format!("[version {}]", seen + 1))
                    }
                    ("written" | "until", Some(_)) => Some("[when]".to_string()),
                    ("doc", Some(_)) => Some("[doc]".to_string()),
                    ("print", Some(_)) => Some("[print]".to_string()),
                    _ => None,
                };
                match (label, one.as_str()) {
                    (Some(label), Some(said)) if !found.iter().any(|(was, _)| was == said) => {
                        found.push((said.to_string(), label));
                    }
                    _ => volatile(one, found),
                }
            }
        }
        Value::Array(all) => all.iter().for_each(|one| volatile(one, found)),
        _ => {}
    }
}

// Ids, prints and hours change with every run; what an agent is told must not.
fn settled(said: &Value, also: &[(&str, &str)]) -> String {
    let mut found: Vec<(String, String)> = also
        .iter()
        .map(|(was, label)| (was.to_string(), label.to_string()))
        .collect();
    volatile(said, &mut found);
    found.sort_by_key(|(was, _)| std::cmp::Reverse(was.len()));
    let mut told = serde_json::to_string_pretty(said).unwrap();
    for (was, label) in &found {
        told = told.replace(was.as_str(), label);
    }
    told
}

#[test]
fn a_document_lists_the_versions_this_machine_keeps_with_who_wrote_each() {
    let served = Served::new();
    let doc = served.written_by_turns();

    let listed = served.versions(&doc);

    let rows = listed["versions"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{listed}");
    assert_eq!(rows[0]["by"], "the person");
    assert_eq!(rows[1]["by"], "Claude Code");
    assert_eq!(rows[0]["bytes"], DOS.len());
    assert_eq!(rows[1]["bytes"], UNO.len());
    assert!(rows.iter().all(|row| row.get("elsewhere").is_none()));
    insta::assert_snapshot!(settled(&listed, &[]));
}

#[test]
fn a_version_someone_else_signed_is_listed_under_their_name() {
    let served = Served::new();
    let doc = served.wrote(UNO);
    served.person().takes_an_edit_signed_by(&doc, DOS, "Rina");
    served.edits(
        &doc,
        "Dos, como lo dejó la persona.",
        "Tres, otra vez el asistente.",
    );

    let listed = served.versions(&doc);

    let rows = listed["versions"].as_array().unwrap();
    let hers = rows.iter().find(|row| row["bytes"] == DOS.len()).unwrap();
    assert_eq!(hers["by"], "Rina", "{listed}");
}

#[test]
fn a_document_nothing_wrote_over_lists_nothing_and_says_how_a_version_comes_to_be() {
    let served = Served::new();
    let doc = served.wrote(UNO);

    let listed = served.versions(&doc);

    assert_eq!(listed["versions"], json!([]));
    insta::assert_snapshot!(settled(&listed, &[]));
}

#[test]
fn a_past_version_is_read_under_a_notice_and_carries_no_print_to_write_with() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");

    let said = served.call("read_doc", json!({ "doc": doc, "version": first }));

    went(&said);
    let read = &said["structuredContent"];
    assert_eq!(read["body"], UNO);
    assert_eq!(read["by"], "Claude Code");
    assert!(read.get("print").is_none(), "{read}");
    assert!(
        read["notice"]
            .as_str()
            .unwrap()
            .contains("not what it says now"),
        "{read}"
    );
    assert!(
        words(&said).starts_with("(This is a past version"),
        "{said}"
    );
    assert!(served.body_of(&doc).contains("Tres"), "reading wrote");
    insta::assert_snapshot!(settled(read, &[]));
}

#[test]
fn going_back_to_a_version_is_undone_by_restore_alone_and_done_again_by_one_more() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let before = served.body_of(&doc);
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");

    let said = served.call("restore_doc", json!({ "doc": doc, "version": first }));

    went(&said);
    assert_eq!(served.body_of(&doc), UNO);
    assert_eq!(said["structuredContent"]["went_back_to"]["version"], first);
    insta::assert_snapshot!(settled(&said["structuredContent"], &[]));

    went(&served.call("restore_doc", json!({ "doc": doc })));
    assert_eq!(
        served.body_of(&doc),
        before,
        "restore alone did not undo it"
    );

    went(&served.call("restore_doc", json!({ "doc": doc })));
    assert_eq!(served.body_of(&doc), UNO, "restoring twice did not return");
}

#[test]
fn going_back_to_a_version_needs_no_even_if_more_though_the_person_wrote_since() {
    let served = Served::new();
    let doc = served.written_by_turns();
    served
        .person()
        .rewrites_doc(&doc, "# Acta\n\nCuatro, la persona de nuevo.\n");
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");

    let plain = served.call("restore_doc", json!({ "doc": doc }));
    assert!(
        refused(&plain),
        "the one step back went over the person's writing: {plain}"
    );

    went(&served.call("restore_doc", json!({ "doc": doc, "version": first })));
    assert_eq!(served.body_of(&doc), UNO);

    went(&served.call("restore_doc", json!({ "doc": doc })));
    assert!(
        served
            .body_of(&doc)
            .contains("Cuatro, la persona de nuevo."),
        "what the person wrote was not kept to come back to"
    );
}

#[test]
fn the_text_a_document_already_says_is_no_version_to_go_back_to() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");
    went(&served.call("restore_doc", json!({ "doc": doc, "version": first })));

    let again = served.call("restore_doc", json!({ "doc": doc, "version": first }));

    assert!(refused(&again), "{again}");
    assert!(
        words(&again).contains("already says what version"),
        "{again}"
    );
    let listed = served.versions(&doc);
    assert!(
        listed["versions"]
            .as_array()
            .unwrap()
            .iter()
            .all(|row| row["version"] != first.as_str()),
        "what it says now is listed as a past version: {listed}"
    );
    let read = served.call("read_doc", json!({ "doc": doc, "version": first }));
    assert!(refused(&read), "{read}");
    assert!(words(&read).contains("it is no past version"), "{read}");
}

#[test]
fn going_back_twice_to_one_version_of_a_document_with_pages_changes_nothing_the_second_time() {
    let served = Served::new();
    let doc = served.wrote(UNO);
    served.person().rewrites_doc(&doc, DOS);
    went(&served.call(
        "write_doc",
        json!({ "body": "# Anexo\n\nLo que cuelga del acta.\n", "page_of": doc, "anyway": true }),
    ));
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");

    went(&served.call("restore_doc", json!({ "doc": doc, "version": first })));
    let once = served.body_of(&doc);
    assert!(once.contains("Uno,") && once.contains("Anexo"), "{once}");

    let twice = served.call("restore_doc", json!({ "doc": doc, "version": first }));

    assert!(refused(&twice), "{twice}");
    assert!(words(&twice).contains("nothing was changed"), "{twice}");
    assert_eq!(served.body_of(&doc), once);
}

#[test]
fn an_archived_document_shows_its_versions_and_goes_back_to_none() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");
    went(&served.call("archive_doc", json!({ "doc": doc, "archived": true })));

    assert_eq!(served.versions(&doc)["kept_here"], 2);
    let back = served.call("restore_doc", json!({ "doc": doc, "version": first }));

    assert!(refused(&back), "{back}");
    assert!(words(&back).contains("put away"), "{back}");
    assert!(served.body_of(&doc).contains("Tres"));
}

#[test]
fn going_back_to_a_compaction_that_was_undone_does_not_make_it_wait_again() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let print = served.read(&doc)["print"].as_str().unwrap().to_string();
    went(&served.call(
        "write_doc",
        json!({ "doc": doc, "print": print, "compact": true, "body": "# Acta\n\nCompacta.\n" }),
    ));
    went(&served.call_as(CODEX, "restore_doc", json!({ "doc": doc })));
    served
        .person()
        .rewrites_doc(&doc, "# Acta\n\nCuatro, la persona.\n");
    served
        .person()
        .rewrites_doc(&doc, "# Acta\n\nCinco, la persona.\n");
    let compacted = served.version_that_said(&doc, "Compacta.");

    went(&served.call("restore_doc", json!({ "doc": doc, "version": compacted })));

    assert!(
        served.read(&doc).get("compacted").is_none(),
        "it waits again"
    );
    went(&served.call("restore_doc", json!({ "doc": doc })));
    went(&served.call("restore_doc", json!({ "doc": doc })));
    assert!(served.body_of(&doc).contains("Compacta."));
    assert!(
        served.read(&doc).get("compacted").is_none(),
        "undoing and redoing the going back made it wait again"
    );
    went(&served.call(
        "edit_doc",
        json!({ "doc": doc, "old": "Compacta.", "new": "Compacta, y luego corregida." }),
    ));
}

#[test]
fn a_version_is_named_by_how_its_id_begins_when_only_one_begins_so() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");

    let by_start = served.call("read_doc", json!({ "doc": doc, "version": &first[..8] }));
    went(&by_start);
    assert_eq!(by_start["structuredContent"]["body"], UNO);

    let empty = served.call("read_doc", json!({ "doc": doc, "version": "" }));
    assert!(empty["structuredContent"].get("print").is_some(), "{empty}");

    let unknown = served.call("read_doc", json!({ "doc": doc, "version": "zz99" }));
    assert!(refused(&unknown), "{unknown}");
    insta::assert_snapshot!(settled(&unknown, &[(doc.as_str(), "[doc]")]));

    let back = served.call("restore_doc", json!({ "doc": doc, "version": "zz99" }));
    assert!(refused(&back), "{back}");
    assert!(
        served.body_of(&doc).contains("Tres"),
        "an unknown version wrote"
    );
}

#[test]
fn a_start_two_versions_share_is_refused_and_says_to_send_it_whole() {
    let served = Served::new();
    let doc = served.wrote(UNO);
    let paths = served.paths();
    let base = jiff::Timestamp::now() - jiff::SignedDuration::from_hours(72);
    let mut starts = std::collections::BTreeMap::<char, usize>::new();
    let mut round = 0;
    while starts.values().all(|seen| *seen < 2) {
        let body = format!("# Acta\n\nvuelta {round}\n");
        let first = tisty_core::docs::print_of_text(&body)
            .chars()
            .next()
            .unwrap();
        *starts.entry(first).or_default() += 1;
        let by = tisty_core::history::Writer::at(
            paths.data(),
            None,
            base + jiff::SignedDuration::from_mins(31 * (round + 1)),
        );
        tisty_core::docs::write_by(&paths.docs(), &by, &doc, &body).unwrap();
        round += 1;
    }
    let by =
        tisty_core::history::Writer::at(paths.data(), Some(CLAUDE.into()), jiff::Timestamp::now());
    tisty_core::docs::write_by(&paths.docs(), &by, &doc, "# Acta\n\nla última\n").unwrap();
    let shared = starts.iter().find(|(_, seen)| **seen >= 2).unwrap().0;

    let said = served.call(
        "read_doc",
        json!({ "doc": doc, "version": shared.to_string() }),
    );

    assert!(refused(&said), "{said}");
    assert!(
        words(&said).contains("begins more than one version"),
        "{said}"
    );
}

#[test]
fn a_long_version_comes_as_its_outline_and_is_read_on_by_parts_with_no_print() {
    let served = Served::new();
    let long: String = (1..=140)
        .map(|n| format!("## Sesión {n}\n\n{}\n\n", "Lo acordado ese día. ".repeat(5)))
        .collect();
    let long = format!("# Bitácora\n\n{long}");
    assert!(long.chars().count() > 12_000);
    let doc = served.wrote(&long);
    served
        .person()
        .rewrites_doc(&doc, "# Bitácora\n\nResumida.\n");
    let version = served.versions(&doc)["versions"][0]["version"]
        .as_str()
        .unwrap()
        .to_string();

    let whole = served.call("read_doc", json!({ "doc": doc, "version": version }));
    went(&whole);
    let outlined = &whole["structuredContent"];
    assert_eq!(outlined["whole"], false, "{outlined}");
    assert!(outlined.get("body").is_none() && outlined.get("print").is_none());
    assert!(words(&whole).contains("sending `version` again"), "{whole}");

    let first = served.call(
        "read_doc",
        json!({ "doc": doc, "version": version, "chars": 5000 }),
    );
    went(&first);
    let next = first["structuredContent"]["next"].as_u64().unwrap();
    assert!(
        words(&first).contains("with this `version` and `from`"),
        "{first}"
    );

    let second = served.call(
        "read_doc",
        json!({ "doc": doc, "version": version, "chars": 5000, "cursor": next }),
    );
    went(&second);
    assert_eq!(second["structuredContent"]["from"], next);
    let part = |read: &Value| {
        read["structuredContent"]["body"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let (first, second) = (part(&first), part(&second));
    let read: Vec<&str> = first.lines().chain(second.lines()).collect();
    let kept = tisty_core::history::versions(served.paths().data(), &doc)
        .into_iter()
        .find(|one| one.print.starts_with(&version))
        .and_then(|one| tisty_core::history::read(served.paths().data(), &doc, &one.print))
        .unwrap();
    let whole: Vec<&str> = kept.lines().take(read.len()).collect();
    assert_eq!(read, whole, "the parts are not the version");

    let section = served.call(
        "read_doc",
        json!({ "doc": doc, "version": version, "section": 3 }),
    );
    went(&section);
    assert!(
        section["structuredContent"]["body"]
            .as_str()
            .unwrap()
            .starts_with("## Sesión 3"),
        "{section}"
    );
}

#[test]
fn past_forty_versions_the_list_hands_a_next_to_ask_the_older_ones_with() {
    let served = Served::new();
    let doc = served.wrote(UNO);
    let paths = served.paths();
    let base = jiff::Timestamp::now() - jiff::SignedDuration::from_hours(48);
    for round in 0..45 {
        let by = tisty_core::history::Writer::at(
            paths.data(),
            None,
            base + jiff::SignedDuration::from_mins(31 * (round + 1)),
        );
        let body = format!("# Acta\n\nvuelta {round}\n");
        tisty_core::docs::write_by(&paths.docs(), &by, &doc, &body).unwrap();
    }

    let newest = served.versions(&doc);
    assert_eq!(newest["kept_here"], 45, "{newest}");
    assert_eq!(newest["versions"].as_array().unwrap().len(), 40);
    assert_eq!(newest["next"], 40);
    assert!(
        newest["said"]
            .as_str()
            .unwrap()
            .contains("`versions` again with `cursor` 40"),
        "{newest}"
    );

    let older = served.call(
        "read_doc",
        json!({ "doc": doc, "versions": true, "cursor": 40 }),
    );
    went(&older);
    let older = &older["structuredContent"];
    assert_eq!(older["versions"].as_array().unwrap().len(), 5);
    assert!(older.get("next").is_none(), "{older}");
    let oldest = older["versions"][4]["version"].as_str().unwrap();
    let read = served.call("read_doc", json!({ "doc": doc, "version": oldest }));
    assert_eq!(read["structuredContent"]["body"], UNO);
}

#[test]
fn a_version_kept_on_another_machine_is_marked_elsewhere_and_neither_read_nor_gone_back_to() {
    let served = Served::new();
    let doc = served.wrote(UNO);
    let there = "# Acta\n\nDos, escrito en el otro equipo.\n";
    let last = "# Acta\n\nTres, por el asistente del otro equipo.\n";
    let aside = tempfile::tempdir().unwrap();
    let person = served.person();
    person.another_machine_writes(&doc, there, None, aside.path());
    person.another_machine_writes(&doc, last, Some(CODEX), aside.path());
    person.names_that_machine("Portátil de Rina", aside.path());
    person.syncs_that_machine(aside.path());
    person.takes_in_what_that_machine_wrote(&doc, last, Some(CODEX));

    let listed = served.versions(&doc);

    let rows = listed["versions"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{listed}");
    assert_eq!(
        (&listed["kept_here"], &listed["kept_elsewhere"]),
        (&json!(1), &json!(1))
    );
    let away = rows
        .iter()
        .find(|row| row.get("elsewhere").is_some())
        .unwrap();
    assert_eq!(away["elsewhere"], "Portátil de Rina");
    assert_eq!(away["by"], "the person");
    assert_eq!(away["bytes"], there.len());
    let here = rows
        .iter()
        .find(|row| row.get("elsewhere").is_none())
        .unwrap();
    assert_eq!(here["by"], "Claude Code");
    insta::assert_snapshot!("listed_with_one_elsewhere", settled(&listed, &[]));

    let elsewhere = away["version"].as_str().unwrap();
    let read = served.call("read_doc", json!({ "doc": doc, "version": elsewhere }));
    assert!(refused(&read), "{read}");
    insta::assert_snapshot!(
        "reading_one_kept_elsewhere",
        settled(&read, &[(doc.as_str(), "[doc]"), (elsewhere, "[version]")])
    );

    let back = served.call("restore_doc", json!({ "doc": doc, "version": elsewhere }));
    assert!(refused(&back), "{back}");
    assert_eq!(words(&back), words(&read));
    assert_eq!(
        served.body_of(&doc),
        last,
        "a version that is not here wrote"
    );

    let kept = here["version"].as_str().unwrap();
    let read = served.call("read_doc", json!({ "doc": doc, "version": kept }));
    assert_eq!(read["structuredContent"]["body"], UNO);
}

#[test]
fn a_machine_nobody_named_is_called_another_machine_and_the_text_still_here_is_not_its() {
    let served = Served::new();
    let doc = served.wrote(UNO);
    let aside = tempfile::tempdir().unwrap();
    let person = served.person();
    person.another_machine_writes(&doc, DOS, None, aside.path());
    person.another_machine_writes(&doc, "# Acta\n\nTres.\n", Some(CODEX), aside.path());
    person.syncs_that_machine(aside.path());
    assert_eq!(served.body_of(&doc), UNO, "the round brought no body yet");

    let listed = served.versions(&doc);

    let rows = listed["versions"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{listed}");
    assert_eq!(rows[0]["elsewhere"], "another machine");
    let said = listed["said"].as_str().unwrap();
    assert!(
        said.starts_with("Nothing of what") && said.contains("1 marked `elsewhere`"),
        "{said}"
    );
    assert_eq!(rows[0]["by"], "the person");
    assert_eq!(rows[0]["bytes"], DOS.len());
}

#[test]
fn a_log_older_than_the_history_names_no_version_this_machine_wrote_over() {
    let served = Served::new();
    let doc = served.written_by_turns();
    served
        .person()
        .rewrites_doc(&doc, "# Acta\n\nCuatro, la persona de nuevo.\n");
    assert_eq!(served.versions(&doc)["kept_here"], 3);
    std::fs::remove_dir_all(served.paths().data().join("history")).unwrap();

    let listed = served.versions(&doc);

    assert_eq!(listed["versions"], json!([]), "{listed}");
    assert!(listed.get("kept_elsewhere").is_none(), "{listed}");
    assert!(
        listed["said"]
            .as_str()
            .unwrap()
            .starts_with("Nothing of what"),
        "{listed}"
    );
}

#[test]
fn a_tasks_id_lists_and_reads_the_versions_of_its_page() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();

    let listed = served.versions(&task);

    assert_eq!(listed["doc"], page);
    assert_eq!(listed["title"], "Renovar el pasaporte");
    let rows = listed["versions"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{listed}");
    assert_eq!(rows[0]["by"], "Claude Code");
    assert_eq!(
        served.versions(&page),
        listed,
        "the page's own name lists another history"
    );
    insta::assert_snapshot!(
        "listed_by_a_tasks_id",
        settled(&listed, &[(page.as_str(), "[page]")])
    );

    let version = rows[0]["version"].as_str().unwrap();
    let read = served.call("read_doc", json!({ "doc": task, "version": version }));
    went(&read);
    let body = read["structuredContent"]["body"].as_str().unwrap();
    assert!(
        body.contains("Nota uno.") && !body.contains("Nota dos."),
        "{body}"
    );
    insta::assert_snapshot!(
        "read_by_a_tasks_id",
        settled(
            &json!({
                "notice": read["structuredContent"]["notice"],
                "title": read["structuredContent"]["title"],
                "until": read["structuredContent"]["until"],
                "written": read["structuredContent"]["written"],
            }),
            &[(page.as_str(), "[page]"), (version, "[version]")]
        )
    );
}

#[test]
fn going_back_uncut_on_a_page_with_a_digest_still_undoes_and_redoes_itself() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    went(&served.call(
        "append_doc",
        json!({ "doc": task, "body": "Resumen: uno y dos descartadas.", "replace": true }),
    ));
    went(&served.call("append_doc", json!({ "doc": task, "body": "Nota tres." })));
    let with_three = served.body_of(&page);
    let before_the_digest = served.version_that_said(&task, "Nota dos.");

    went(&served.call(
        "restore_doc",
        json!({ "doc": task, "version": before_the_digest }),
    ));
    let went_back = served.body_of(&page);
    assert!(went_back.contains("Nota dos.") && !went_back.contains("Resumen"));

    went(&served.call("restore_doc", json!({ "doc": task })));
    assert_eq!(served.body_of(&page), with_three, "the undo missed");
    went(&served.call("restore_doc", json!({ "doc": task })));
    assert_eq!(
        served.body_of(&page),
        went_back,
        "the second undo swapped the digest instead of going back again"
    );
}

#[test]
fn going_back_on_a_page_with_a_digest_is_undone_before_the_digest_is_swapped() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    went(&served.call(
        "append_doc",
        json!({ "doc": task, "body": "Resumen: uno y dos descartadas.", "replace": true }),
    ));
    went(&served.call_as(
        CODEX,
        "append_doc",
        json!({ "doc": task, "body": "Nota tres." }),
    ));
    let with_three = served.body_of(&page);
    assert!(with_three.contains("Resumen") && with_three.contains("Nota tres."));
    let digest_alone = served.version_that_said(&task, "Resumen: uno y dos descartadas.");

    went(&served.call(
        "restore_doc",
        json!({ "doc": task, "version": digest_alone }),
    ));
    let went_back = served.body_of(&page);
    assert!(
        went_back.contains("Resumen") && !went_back.contains("Nota tres."),
        "{went_back}"
    );

    went(&served.call("restore_doc", json!({ "doc": task })));
    assert_eq!(
        served.body_of(&page),
        with_three,
        "restore alone swapped the digest instead of undoing the going back"
    );

    went(&served.call("restore_doc", json!({ "doc": task })));
    assert_eq!(
        served.body_of(&page),
        went_back,
        "restoring twice did not return"
    );
}

#[test]
fn going_back_on_a_tasks_id_reaches_the_assistants_notes_and_nothing_else() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    let notes_only = served.versions(&task)["versions"][0]["version"]
        .as_str()
        .unwrap()
        .to_string();

    went(&served.call("restore_doc", json!({ "doc": task, "version": notes_only })));
    let now = served.body_of(&page);
    assert!(
        now.contains("Nota uno.") && !now.contains("Nota dos."),
        "{now}"
    );

    went(&served.call("restore_doc", json!({ "doc": task })));
    let both = served.body_of(&page);
    assert!(both.contains("Nota dos."), "{both}");

    let id = person::task(&task);
    served.person().rewrites_its_page(
        id,
        &page,
        &both.replace("Pedir hora.", "Pedir hora y llevar la foto."),
    );
    let refused_whole = served.call("restore_doc", json!({ "doc": task, "version": notes_only }));
    assert!(refused(&refused_whole), "{refused_whole}");
    assert!(
        words(&refused_whole).contains("reaches only the assistant's notes")
            && words(&refused_whole).contains(&format!("`{page}`")),
        "{refused_whole}"
    );
    assert!(served.body_of(&page).contains("llevar la foto"));
}

#[test]
fn going_back_by_the_pages_own_name_takes_the_whole_page_back() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    let before = served.body_of(&page);
    served.person().rewrites_its_page(
        person::task(&task),
        &page,
        &before.replace("Pedir hora.", "Pedir hora y llevar la foto."),
    );
    let version = served.version_that_said(&page, "Nota dos.");

    let by_task = served.call("restore_doc", json!({ "doc": task, "version": version }));
    assert!(refused(&by_task), "{by_task}");

    went(&served.call("restore_doc", json!({ "doc": page, "version": version })));
    assert_eq!(served.body_of(&page), before);

    let undo_by_task = served.call("restore_doc", json!({ "doc": task }));
    assert!(refused(&undo_by_task), "{undo_by_task}");
    assert!(
        words(&undo_by_task).contains(&format!("`restore_doc` with `{page}` as `doc`")),
        "{undo_by_task}"
    );
    went(&served.call("restore_doc", json!({ "doc": page })));
    assert!(served.body_of(&page).contains("llevar la foto"));
}

#[test]
fn a_going_back_by_the_pages_name_its_task_refuses_leaves_the_digest_swap_alone() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    went(&served.call(
        "append_doc",
        json!({ "doc": task, "body": "Resumen: uno y dos descartadas.", "replace": true }),
    ));
    let digest = served.body_of(&page);
    went(&served.call_as(
        CODEX,
        "edit_doc",
        json!({ "doc": page, "old": "<!-- - [ ] The first step -->", "new": "- [ ] Llevar la foto" }),
    ));
    served.person().ticks(person::task(&task), 1);
    let with_the_step = served.body_of(&page);
    let before_the_step = served.version_that_said(&page, "Resumen: uno y dos descartadas.");

    let refused_back = served.call(
        "restore_doc",
        json!({ "doc": page, "version": before_the_step }),
    );
    assert!(refused(&refused_back), "{refused_back}");
    assert_eq!(served.body_of(&page), with_the_step);

    let out = served.call("restore_doc", json!({ "doc": task }));
    went(&out);
    assert_eq!(
        out["structuredContent"]["notes_put_back"], true,
        "a going back that never stood blocked the digest's swap: {out}"
    );
    assert!(!served.body_of(&page).contains("Resumen"), "{digest}");
}

#[test]
fn undoing_a_whole_page_going_back_leaves_the_digest_to_swap_by_the_tasks_id() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    went(&served.call(
        "append_doc",
        json!({ "doc": task, "body": "Resumen: uno y dos descartadas.", "replace": true }),
    ));
    let digest = served.body_of(&page);
    served.person().rewrites_its_page(
        person::task(&task),
        &page,
        &digest.replace("Pedir hora.", "Pedir hora y llevar la foto."),
    );
    let before_the_digest = served.version_that_said(&page, "Nota dos.");
    went(&served.call(
        "restore_doc",
        json!({ "doc": page, "version": before_the_digest }),
    ));
    went(&served.call("restore_doc", json!({ "doc": page })));
    assert!(served.body_of(&page).contains("Resumen"));

    let swapped = served.call("restore_doc", json!({ "doc": task }));

    went(&swapped);
    assert_eq!(
        swapped["structuredContent"]["notes_put_back"], true,
        "{swapped}"
    );
    let now = served.body_of(&page);
    assert!(
        now.contains("Nota dos.") && now.contains("llevar la foto"),
        "{now}"
    );
}

#[test]
fn a_digest_written_after_a_going_back_is_still_swapped_out_and_back_in() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    let first_note = served.version_that_said(&task, "Nota uno.");
    went(&served.call("restore_doc", json!({ "doc": task, "version": first_note })));
    let gone_back = served.body_of(&page);
    went(&served.call(
        "append_doc",
        json!({ "doc": task, "body": "Resumen: la nota uno.", "replace": true }),
    ));
    let digest = served.body_of(&page);

    let out = served.call("restore_doc", json!({ "doc": task }));
    went(&out);
    assert_eq!(out["structuredContent"]["notes_put_back"], true, "{out}");
    assert_eq!(served.body_of(&page), gone_back);

    let back_in = served.call("restore_doc", json!({ "doc": task }));
    went(&back_in);
    assert_eq!(
        back_in["structuredContent"]["notes_put_back"], true,
        "the digest came back as a plain undo: {back_in}"
    );
    assert_eq!(served.body_of(&page), digest);
}

#[test]
fn a_closed_tasks_page_still_lists_its_versions_and_takes_no_going_back() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    let version = served.versions(&task)["versions"][0]["version"]
        .as_str()
        .unwrap()
        .to_string();
    served.person().closes(person::task(&task));

    assert_eq!(served.versions(&task)["versions"][0]["version"], version);
    went(&served.call("read_doc", json!({ "doc": task, "version": version })));

    for doc in [&task, &page] {
        let back = served.call("restore_doc", json!({ "doc": doc, "version": version }));
        assert!(refused(&back), "{doc}: {back}");
    }
    assert!(
        served.read(&page)["body"]
            .as_str()
            .unwrap()
            .contains("Nota dos.")
    );
}

#[test]
fn a_task_out_of_sight_answers_for_its_versions_like_an_id_nobody_has() {
    const NOBODY: &str = "01J00000000000000000000000";
    let served = Served::new();
    let (task, _) = served.task_with_a_page();
    let version = served.versions(&task)["versions"][0]["version"]
        .as_str()
        .unwrap()
        .to_string();
    served.person().hides(person::task(&task));

    for (name, args) in [
        ("read_doc", json!({ "versions": true })),
        ("read_doc", json!({ "version": version })),
        ("restore_doc", json!({ "version": version })),
    ] {
        let with = |doc: &str| {
            let mut args = args.clone();
            args["doc"] = json!(doc);
            served.call(name, args)
        };
        let (hidden, nobody) = (with(&task), with(NOBODY));
        assert!(refused(&hidden), "{name}: {hidden}");
        assert_eq!(
            hidden.to_string().replace(&task, NOBODY),
            nobody.to_string(),
            "{name} told a hidden task from nothing"
        );
    }
}

#[test]
fn a_hidden_tasks_page_by_its_own_name_answers_like_a_page_nobody_has() {
    const NOBODY: &str = "t_01j00000000000000000000000-0001";
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    let version = served.versions(&page)["versions"][0]["version"]
        .as_str()
        .unwrap()
        .to_string();
    served.person().hides(person::task(&task));

    for (name, args) in [
        ("read_doc", json!({ "versions": true })),
        ("read_doc", json!({ "version": version })),
        ("restore_doc", json!({ "version": version })),
        ("restore_doc", json!({ "version": " " })),
    ] {
        let with = |doc: &str| {
            let mut args = args.clone();
            args["doc"] = json!(doc);
            served.call(name, args)
        };
        let (hidden, nobody) = (with(&page), with(NOBODY));
        assert!(refused(&hidden), "{name} {args}: {hidden}");
        assert_eq!(
            hidden.to_string().replace(&page, NOBODY),
            nobody.to_string(),
            "{name} {args} told a hidden page from nothing"
        );
    }
}

#[test]
fn a_locked_document_shows_its_versions_and_goes_back_to_none() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");
    served.person().locks_doc(&doc);

    assert_eq!(served.versions(&doc)["kept_here"], 2);
    let back = served.call("restore_doc", json!({ "doc": doc, "version": first }));

    assert!(refused(&back), "{back}");
    assert!(words(&back).contains("is locked"), "{back}");
    assert!(served.body_of(&doc).contains("Tres"));
}

#[test]
fn a_compaction_waiting_for_the_person_takes_no_version_until_it_is_undone() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");
    let before = served.body_of(&doc);
    let print = served.read(&doc)["print"].as_str().unwrap().to_string();
    went(&served.call(
        "write_doc",
        json!({ "doc": doc, "print": print, "compact": true, "body": "# Acta\n\nCompacta.\n" }),
    ));

    let waiting = served.call("restore_doc", json!({ "doc": doc, "version": first }));

    assert!(refused(&waiting), "{waiting}");
    insta::assert_snapshot!(settled(&waiting, &[(doc.as_str(), "[doc]")]));
    assert!(served.body_of(&doc).contains("Compacta."));

    went(&served.call("restore_doc", json!({ "doc": doc })));
    assert_eq!(
        served.body_of(&doc),
        before,
        "restore alone no longer undoes a compaction"
    );
    went(&served.call("restore_doc", json!({ "doc": doc, "version": first })));
    assert_eq!(served.body_of(&doc), UNO);
}

#[test]
fn listing_reading_and_carrying_out_are_asked_one_at_a_time() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let first = served.version_that_said(&doc, "Uno, como lo dejó el asistente.");

    let both = served.call(
        "read_doc",
        json!({ "doc": doc, "versions": true, "version": first }),
    );
    assert!(refused(&both), "{both}");
    assert!(words(&both).contains("send one of the two"), "{both}");

    for args in [
        json!({ "doc": doc, "versions": true, "as": "markdown" }),
        json!({ "doc": doc, "version": first, "as": "markdown" }),
    ] {
        let carried = served.call("read_doc", args);
        assert!(refused(&carried), "{carried}");
        assert!(words(&carried).contains("as it reads now"), "{carried}");
    }

    let blank = served.call("read_doc", json!({ "doc": doc, "version": " " }));
    assert!(
        blank["structuredContent"].get("print").is_some(),
        "an empty version is a field left out, as everywhere: {blank}"
    );

    let plain = served.call("read_doc", json!({ "doc": doc, "versions": false }));
    assert!(plain["structuredContent"].get("print").is_some(), "{plain}");

    let by_parts = served.call(
        "read_doc",
        json!({ "doc": doc, "versions": true, "from": 2 }),
    );
    assert!(refused(&by_parts), "{by_parts}");
    assert!(words(&by_parts).contains("takes no `from`"), "{by_parts}");
}

#[test]
fn a_cursor_past_the_last_version_says_so_instead_of_an_empty_list() {
    let served = Served::new();
    let doc = served.written_by_turns();

    let past = served.call(
        "read_doc",
        json!({ "doc": doc, "versions": true, "cursor": 9 }),
    );

    went(&past);
    assert_eq!(past["structuredContent"]["versions"], json!([]));
    let said = past["structuredContent"]["said"].as_str().unwrap();
    assert!(said.contains("the list holds 2 in all"), "{said}");
}

#[test]
fn a_version_sent_empty_is_a_field_left_out_like_every_other() {
    let served = Served::new();
    let doc = served.written_by_turns();
    let before = served.body_of(&doc);

    let blank = served.call("restore_doc", json!({ "doc": doc, "version": " " }));

    went(&blank);
    assert!(
        blank["structuredContent"].get("went_back_to").is_none(),
        "{blank}"
    );
    assert_ne!(served.body_of(&doc), before, "it undid nothing");
    went(&served.call("restore_doc", json!({ "doc": doc })));
    assert_eq!(served.body_of(&doc), before, "the undo did not undo itself");
}

#[test]
fn a_digest_is_still_swapped_back_after_another_write_on_the_page() {
    let served = Served::new();
    let (task, page) = served.task_with_a_page();
    went(&served.call(
        "append_doc",
        json!({ "doc": task, "body": "Resumen: uno y dos descartadas.", "replace": true }),
    ));
    went(&served.call(
        "edit_doc",
        json!({ "doc": page, "old": "Pedir hora.", "new": "Pedir hora y llevar la foto." }),
    ));

    let swapped = served.call("restore_doc", json!({ "doc": task }));

    went(&swapped);
    assert_eq!(
        swapped["structuredContent"]["notes_put_back"], true,
        "{swapped}"
    );
    let now = served.body_of(&page);
    assert!(
        now.contains("Nota uno.") && now.contains("llevar la foto"),
        "{now}"
    );
}

#[test]
fn a_tool_named_for_history_is_pointed_at_the_two_that_hold_it() {
    let served = Served::new();

    let said = served
        .answered(CLAUDE, "doc_history", json!({}))
        .to_string();

    assert!(said.contains("unknown tool: doc_history"), "{said}");
    assert!(
        said.contains("`versions` lists its past versions and `version` reads one"),
        "{said}"
    );
}
