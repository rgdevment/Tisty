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
        self.ask("tools/call", json!({ "name": name, "arguments": args }))
    }

    fn ask(&self, method: &str, params: Value) -> Value {
        let root = self.home.path();
        let asked =
            json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params }).to_string();
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

    fn body(&self, file: &str) -> String {
        let root = self.home.path();
        let paths = tisty_core::Paths::new(root.join("data"), root.join("config"));
        tisty_core::docs::read(&paths.docs(), file).unwrap()
    }

    fn diary(&self) -> (String, String) {
        self.person()
            .writes_in_the_diary(day(), "Paseo con Rina por el cerro.")
    }
}

fn day() -> jiff::civil::Date {
    "2026-11-15".parse().unwrap()
}

fn refused(result: &Value) -> bool {
    result["isError"] == json!(true)
}

const MISSING: &str = "zzzzzzzz-0001";

fn untold(answer: &Value) {
    let said = answer.to_string();
    for word in ["Rina", "Diario", "Noviembre", "diary"] {
        assert!(!said.contains(word), "{word} reached the agent: {said}");
    }
}

fn answered_as_missing(served: &Served, name: &str, args: impl Fn(&str) -> Value, file: &str) {
    let hidden = served.call(name, args(file));
    let missing = served.call(name, args(MISSING));
    untold(&hidden);
    assert_eq!(
        refused(&hidden),
        refused(&missing),
        "{name}: {hidden} / {missing}"
    );
    let said = hidden.to_string().replace(file, MISSING);
    assert_eq!(
        said,
        missing.to_string(),
        "{name} told a hidden diary from nothing"
    );
}

#[test]
fn a_shut_diary_answers_every_tool_as_a_document_that_is_not_there() {
    let served = Served::new();
    let (book, page) = served.diary();
    let other = served.person().writes_doc("# Planes\n\nAlgo.\n");
    let outside = served.home.path().join("hoja.md");
    std::fs::write(&outside, "# Hoja\n\nAlgo.\n").unwrap();
    let picture = served.home.path().join("foto.txt");
    std::fs::write(&picture, "una foto").unwrap();
    let exported = served.home.path().join("out.md");

    for file in [&book, &page] {
        answered_as_missing(&served, "workspace_read", |doc| json!({ "doc": doc }), file);
        answered_as_missing(
            &served,
            "workspace_read",
            |doc| json!({ "doc": doc, "view": "card" }),
            file,
        );
        answered_as_missing(
            &served,
            "workspace_read",
            |doc| json!({ "doc": doc, "view": "links", "links": "all" }),
            file,
        );
        answered_as_missing(
            &served,
            "workspace_read",
            |doc| json!({ "doc": doc, "view": "card", "section": 0 }),
            file,
        );
        answered_as_missing(
            &served,
            "file_transfer",
            |doc| json!({ "action": "export_doc", "doc": doc, "path": exported.to_string_lossy() }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_edit",
            |doc| json!({ "doc": doc, "mode": "append", "content": "Hola." }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_edit",
            |doc| json!({ "doc": doc, "mode": "patch", "old": "Paseo", "content": "Vuelta" }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_edit_table",
            |doc| json!({ "action": "add_row", "doc": doc }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_annotate",
            |doc| json!({ "action": "flag", "docs": [doc], "reason": "obsolete", "body": "viejo" }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_annotate",
            |doc| json!({ "action": "archive", "docs": [doc] }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_edit",
            |doc| json!({ "doc": doc, "mode": "restore" }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_edit",
            |doc| json!({ "doc": doc, "mode": "restore", "version": "0a1b2c3d" }),
            file,
        );
        answered_as_missing(
            &served,
            "workspace_read",
            |doc| json!({ "doc": doc, "view": "versions" }),
            file,
        );
        answered_as_missing(
            &served,
            "workspace_read",
            |doc| json!({ "doc": doc, "version": "0a1b2c3d" }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_organize",
            |doc| json!({ "action": "file", "docs": [doc], "folder": "Personal" }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_organize",
            |doc| json!({ "action": "add_page", "docs": [other], "parent": doc }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_organize",
            |doc| json!({ "action": "link_translation", "docs": [other], "parent": doc, "lang": "en" }),
            file,
        );
        answered_as_missing(
            &served,
            "workspace_search",
            |doc| json!({ "mode": "in_doc", "doc": doc, "query": "Rina" }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_create",
            |doc| json!({ "page_of": doc, "body": "# Hoja\n\nAlgo.\n" }),
            file,
        );
        answered_as_missing(
            &served,
            "file_transfer",
            |doc| json!({ "action": "import_doc", "page_of": doc, "path": outside.to_string_lossy() }),
            file,
        );
        answered_as_missing(
            &served,
            "file_transfer",
            |doc| json!({ "action": "attach", "doc": doc, "path": picture.to_string_lossy() }),
            file,
        );
        answered_as_missing(
            &served,
            "doc_annotate",
            |doc| json!({ "action": "summarize", "docs": [doc], "summary": "Paseos." }),
            file,
        );
        let read = served.ask(
            "resources/read",
            json!({ "uri": format!("tisty:doc/{file}") }),
        );
        let nothing = served.ask(
            "resources/read",
            json!({ "uri": format!("tisty:doc/{MISSING}") }),
        );
        untold(&read);
        assert_eq!(
            read.to_string().replace(file.as_str(), MISSING),
            nothing.to_string()
        );
    }
    assert!(served.body(&page).contains("Paseo con Rina por el cerro."));
    assert!(!served.body(&page).contains("Hola."));
    assert!(served.person().state().agent_notes.is_empty());
}

#[test]
fn a_shut_diary_is_left_out_of_every_listing_and_count() {
    let served = Served::new();
    served.call("workspace_catch_up", json!({}));
    served.diary();

    for (name, args) in [
        ("workspace_catch_up", json!({})),
        (
            "workspace_search",
            json!({ "mode": "folder", "scope": "either" }),
        ),
        ("workspace_search", json!({})),
        ("workspace_search", json!({ "query": "paseo" })),
        (
            "workspace_search",
            json!({ "query": "cerro", "scope": "either" }),
        ),
    ] {
        let searching = args.get("query").is_some();
        let answer = served.call(name, args);
        assert!(!refused(&answer), "{name}: {answer}");
        untold(&answer);
        if searching {
            assert_eq!(
                answer["structuredContent"]["docsTotal"],
                json!(0),
                "{answer}"
            );
        }
    }
    let listed = served.ask("resources/list", json!({}));
    untold(&listed);
    let caught = served.call("workspace_catch_up", json!({}));
    assert_eq!(
        caught["structuredContent"]["counts"]["docs"],
        json!(0),
        "{caught}"
    );
}

#[test]
fn an_agent_never_starts_a_diary_of_its_own() {
    let served = Served::new();
    let plain = served.person().writes_doc("# Planes\n\nAlgo.\n");

    let written = served.call(
        "doc_create",
        json!({ "body": "---\ndiary: on\n---\n\n# Diario 2026\n" }),
    );
    let edited = served.call(
        "doc_edit",
        json!({ "doc": plain, "mode": "patch", "old": "# Planes", "content": "---\ndiary: on\n---\n\n# Planes" }),
    );
    let outside = served.home.path().join("diario.md");
    std::fs::write(&outside, "---\ndiary: on\n---\n\n# Diario 2026\n").unwrap();
    let imported = served.call(
        "file_transfer",
        json!({ "action": "import_doc", "path": outside.to_string_lossy() }),
    );

    for answer in [&written, &edited, &imported] {
        assert!(refused(answer), "{answer}");
        assert!(
            answer.to_string().contains("only the person starts one"),
            "{answer}"
        );
    }
    assert!(!served.body(&plain).contains("diary"));
    let state = served.person().state();
    assert!(state.docs.values().all(|one| !one.diary));
}

#[test]
fn an_opened_diary_is_seen_but_its_words_are_never_rewritten() {
    let served = Served::new();
    let (book, page) = served.diary();
    served.person().opens_the_diary_to_agents(true);

    let listed = served.call(
        "workspace_search",
        json!({ "mode": "folder", "scope": "either" }),
    );
    assert!(listed.to_string().contains(&book), "{listed}");
    let read = served.call("workspace_read", json!({ "doc": page }));
    assert!(read.to_string().contains("Paseo con Rina"), "{read}");

    let edited = served.call(
        "doc_edit",
        json!({ "doc": page, "mode": "patch", "old": "Paseo", "content": "Vuelta" }),
    );
    assert!(refused(&edited), "{edited}");
    assert!(
        edited.to_string().contains("nothing in it is rewritten"),
        "{edited}"
    );
    assert!(served.body(&page).contains("Paseo con Rina"));

    served.person().opens_the_diary_to_agents(false);
    untold(&served.call(
        "workspace_search",
        json!({ "mode": "folder", "scope": "either" }),
    ));
}

fn santiago_today() -> jiff::civil::Date {
    jiff::ZonedDateTime::now()
        .in_tz("America/Santiago")
        .unwrap()
        .date()
}

#[test]
fn an_agent_adds_only_at_the_end_of_today_signed_once_the_diary_is_open() {
    let served = Served::new();
    let today = santiago_today();
    let (book, page) = served
        .person()
        .writes_in_the_diary(today, "Paseo con Rina por el cerro.");
    served.person().opens_the_diary_to_agents(true);

    let under = served.call(
        "doc_edit",
        json!({ "doc": page, "mode": "append", "under": "Notas", "content": "Algo." }),
    );
    let added = served.call(
        "doc_edit",
        json!({ "doc": book, "mode": "append", "content": "Jaime called about the trip." }),
    );

    assert!(refused(&under), "{under}");
    assert!(!refused(&added), "{added}");
    assert_eq!(added["structuredContent"]["doc"], json!(page), "{added}");
    let written = served.body(&page);
    let day = format!(
        "{} {}",
        tisty_core::diary::SPANISH.weekdays
            [usize::from(today.weekday().to_monday_zero_offset().unsigned_abs())],
        today.day()
    );
    let mine = written.find("Paseo con Rina por el cerro.").unwrap();
    let theirs = written.find("Jaime called about the trip.").unwrap();
    assert!(written.contains(&format!("## {day}")), "{written}");
    assert!(mine < theirs, "{written}");
    assert!(
        written.contains("** · assistant\nJaime called about the trip."),
        "{written}"
    );
    assert!(
        written.trim_end().ends_with("Jaime called about the trip."),
        "{written}"
    );
    assert!(!written.contains("Algo."));
}

#[test]
fn what_an_open_diary_said_before_is_never_listed_nor_read() {
    let served = Served::new();
    let (book, page) = served.diary();
    served.person().opens_the_diary_to_agents(true);

    for file in [&book, &page] {
        for args in [
            json!({ "doc": file, "view": "versions" }),
            json!({ "doc": file, "version": "0a1b2c3d" }),
        ] {
            let answer = served.call("workspace_read", args);
            assert!(refused(&answer), "{answer}");
            assert!(answer.to_string().contains("no version of it"), "{answer}");
        }
    }
    assert!(!refused(
        &served.call("workspace_read", json!({ "doc": page }))
    ));
}

#[test]
fn every_other_change_to_an_open_diary_is_refused() {
    let served = Served::new();
    let (book, page) = served.diary();
    let other = served.person().writes_doc("# Planes\n\nAlgo.\n");
    let outside = served.home.path().join("hoja.md");
    std::fs::write(&outside, "# Hoja\n\nAlgo.\n").unwrap();
    let picture = served.home.path().join("foto.txt");
    std::fs::write(&picture, "una foto").unwrap();
    served.person().opens_the_diary_to_agents(true);

    for (name, args) in [
        (
            "doc_annotate",
            json!({ "action": "archive", "docs": [page] }),
        ),
        (
            "doc_annotate",
            json!({ "action": "flag", "docs": [book], "reason": "obsolete", "body": "viejo" }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [book], "folder": "Personal" }),
        ),
        (
            "doc_organize",
            json!({ "action": "add_page", "docs": [other], "parent": book }),
        ),
        (
            "doc_organize",
            json!({ "action": "link_translation", "docs": [other], "parent": page, "lang": "en" }),
        ),
        ("doc_edit", json!({ "doc": page, "mode": "restore" })),
        (
            "doc_edit",
            json!({ "doc": page, "mode": "restore", "version": "0a1b2c3d" }),
        ),
        (
            "doc_edit_table",
            json!({ "doc": page, "action": "add_row" }),
        ),
        (
            "doc_edit",
            json!({ "doc": page, "mode": "rewrite", "content": "# Noviembre\n" }),
        ),
        (
            "doc_create",
            json!({ "page_of": book, "body": "# Hoja\n\nAlgo.\n" }),
        ),
        (
            "file_transfer",
            json!({ "action": "import_doc", "page_of": book, "path": outside.to_string_lossy() }),
        ),
        (
            "file_transfer",
            json!({ "action": "attach", "doc": page, "path": picture.to_string_lossy() }),
        ),
        (
            "doc_annotate",
            json!({ "action": "summarize", "docs": [book], "summary": "Paseos." }),
        ),
        (
            "doc_annotate",
            json!({ "action": "summarize", "docs": [page], "notes": "Leer antes." }),
        ),
    ] {
        let answer = served.call(name, args);
        assert!(refused(&answer), "{name}: {answer}");
        assert!(
            answer.to_string().contains("its words are theirs"),
            "{name}: {answer}"
        );
    }
    assert!(served.body(&page).contains("Paseo con Rina por el cerro."));
    let state = served.person().state();
    assert!(
        state
            .docs
            .values()
            .all(|one| !one.archived && one.flagged.is_none())
    );
    assert_eq!(state.docs.len(), 3, "no page was hung under the diary");
    assert!(
        state.agent_notes.is_empty(),
        "a note only agents read reached the diary"
    );
}

#[test]
fn a_list_of_names_or_a_translation_meets_the_same_door() {
    let served = Served::new();
    let (book, page) = served.diary();
    let other = served.person().writes_doc("# Planes\n\nAlgo.\n");
    served.person().opens_the_diary_to_agents(true);

    for (name, args) in [
        (
            "doc_annotate",
            json!({ "action": "flag", "docs": [book], "reason": "obsolete", "body": "viejo" }),
        ),
        (
            "doc_organize",
            json!({ "action": "file", "docs": [other, book], "folder": "Personal" }),
        ),
        (
            "doc_organize",
            json!({ "action": "remove_page", "docs": [page] }),
        ),
        (
            "doc_annotate",
            json!({ "action": "flag", "docs": [other], "reason": "duplicate_of", "instead": page, "body": "viejo" }),
        ),
        (
            "doc_create",
            json!({ "translation_of": page, "lang": "en", "body": "# November\n\nSome.\n" }),
        ),
    ] {
        let answer = served.call(name, args);
        assert!(refused(&answer), "{name}: {answer}");
        assert!(
            answer.to_string().contains("its words are theirs"),
            "{name}: {answer}"
        );
    }
    let state = served.person().state();
    assert!(
        state
            .docs
            .values()
            .all(|one| one.flagged.is_none() && one.folder.is_none()),
        "a list is refused whole, so the other document stays where it was"
    );
    assert_eq!(state.docs.len(), 3, "nothing joined the diary or left it");
}

#[test]
fn an_agent_never_writes_today_into_a_locked_book_by_naming_another_year() {
    let served = Served::new();
    let today = santiago_today();
    let (book, page) = served.person().writes_in_the_diary(today, "Hoy.");
    let last_year = today.checked_sub(jiff::Span::new().years(1)).unwrap();
    let (old_book, _) = served.person().writes_in_the_diary(last_year, "Antes.");
    served.person().locks_doc(&book);
    served.person().opens_the_diary_to_agents(true);

    let added = served.call(
        "doc_edit",
        json!({ "doc": old_book, "mode": "append", "content": "Jaime called." }),
    );

    assert!(refused(&added), "{added}");
    assert!(!served.body(&page).contains("Jaime called."));
}

#[test]
fn a_note_an_agent_left_on_the_diary_before_reaches_no_reader() {
    let served = Served::new();
    let (book, page) = served.diary();
    served
        .person()
        .finds_an_old_note_on(&book, "Resumen viejo del diario");
    served
        .person()
        .finds_an_old_note_on(&page, "Resumen viejo del mes");
    served.person().opens_the_diary_to_agents(true);

    for (name, args) in [
        ("workspace_read", json!({ "doc": book })),
        ("workspace_read", json!({ "doc": page })),
        ("workspace_read", json!({ "doc": book, "view": "card" })),
        (
            "workspace_search",
            json!({ "mode": "folder", "scope": "either" }),
        ),
    ] {
        let said = served.call(name, args.clone());
        assert!(!refused(&said), "{name} {args}: {said}");
        assert!(
            !said.to_string().contains("Resumen viejo"),
            "{name} {args}: {said}"
        );
    }
}
