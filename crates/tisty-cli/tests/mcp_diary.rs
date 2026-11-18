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

    for file in [&book, &page] {
        answered_as_missing(&served, "read_doc", |doc| json!({ "doc": doc }), file);
        answered_as_missing(&served, "brain", |doc| json!({ "doc": doc }), file);
        answered_as_missing(
            &served,
            "brain",
            |doc| json!({ "doc": doc, "links": "all" }),
            file,
        );
        answered_as_missing(
            &served,
            "brain",
            |doc| json!({ "doc": doc, "section": 0 }),
            file,
        );
        answered_as_missing(&served, "export_doc", |doc| json!({ "doc": doc }), file);
        answered_as_missing(
            &served,
            "append_doc",
            |doc| json!({ "doc": doc, "body": "Hola." }),
            file,
        );
        answered_as_missing(
            &served,
            "edit_doc",
            |doc| json!({ "doc": doc, "old": "Paseo", "new": "Vuelta" }),
            file,
        );
        answered_as_missing(&served, "edit_table", |doc| json!({ "doc": doc }), file);
        answered_as_missing(
            &served,
            "flag_doc",
            |doc| json!({ "doc": doc, "reason": "obsolete", "body": "viejo" }),
            file,
        );
        answered_as_missing(
            &served,
            "archive_doc",
            |doc| json!({ "doc": doc, "archived": true }),
            file,
        );
        answered_as_missing(&served, "restore_doc", |doc| json!({ "doc": doc }), file);
        answered_as_missing(
            &served,
            "file_doc",
            |doc| json!({ "doc": doc, "folder": "Personal" }),
            file,
        );
        answered_as_missing(
            &served,
            "page_doc",
            |doc| json!({ "doc": other, "page_of": doc }),
            file,
        );
        answered_as_missing(
            &served,
            "tongue_doc",
            |doc| json!({ "doc": other, "of": doc }),
            file,
        );
        answered_as_missing(
            &served,
            "brain",
            |doc| json!({ "doc": doc, "query": "Rina" }),
            file,
        );
        answered_as_missing(
            &served,
            "write_doc",
            |doc| json!({ "page_of": doc, "body": "# Hoja\n\nAlgo.\n" }),
            file,
        );
        answered_as_missing(
            &served,
            "import_doc",
            |doc| json!({ "page_of": doc, "path": outside.to_string_lossy() }),
            file,
        );
        answered_as_missing(
            &served,
            "attach",
            |doc| json!({ "doc": doc, "path": picture.to_string_lossy() }),
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
}

#[test]
fn a_shut_diary_is_left_out_of_every_listing_and_count() {
    let served = Served::new();
    served.call("catch_up", json!({}));
    served.diary();

    for (name, args) in [
        ("catch_up", json!({})),
        ("brain", json!({ "scope": "either" })),
        ("brain", json!({})),
        ("brain", json!({ "query": "paseo" })),
        ("brain", json!({ "query": "cerro", "scope": "either" })),
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
    let caught = served.call("catch_up", json!({}));
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
        "write_doc",
        json!({ "body": "---\ndiary: on\n---\n\n# Diario 2026\n" }),
    );
    let edited = served.call(
        "edit_doc",
        json!({ "doc": plain, "old": "# Planes", "new": "---\ndiary: on\n---\n\n# Planes" }),
    );
    let outside = served.home.path().join("diario.md");
    std::fs::write(&outside, "---\ndiary: on\n---\n\n# Diario 2026\n").unwrap();
    let imported = served.call("import_doc", json!({ "path": outside.to_string_lossy() }));

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

    let listed = served.call("brain", json!({ "scope": "either" }));
    assert!(listed.to_string().contains(&book), "{listed}");
    let read = served.call("read_doc", json!({ "doc": page }));
    assert!(read.to_string().contains("Paseo con Rina"), "{read}");

    let edited = served.call(
        "edit_doc",
        json!({ "doc": page, "old": "Paseo", "new": "Vuelta" }),
    );
    assert!(refused(&edited), "{edited}");
    assert!(
        edited.to_string().contains("nothing in it is rewritten"),
        "{edited}"
    );
    assert!(served.body(&page).contains("Paseo con Rina"));

    served.person().opens_the_diary_to_agents(false);
    untold(&served.call("brain", json!({ "scope": "either" })));
}

fn santiago_today() -> jiff::civil::Date {
    jiff::Zoned::now().in_tz("America/Santiago").unwrap().date()
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
        "append_doc",
        json!({ "doc": page, "under": "Notas", "body": "Algo." }),
    );
    let added = served.call(
        "append_doc",
        json!({ "doc": book, "body": "Jaime called about the trip." }),
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
        ("archive_doc", json!({ "doc": page, "archived": true })),
        (
            "flag_doc",
            json!({ "doc": book, "reason": "obsolete", "body": "viejo" }),
        ),
        ("file_doc", json!({ "doc": book, "folder": "Personal" })),
        ("page_doc", json!({ "doc": other, "page_of": book })),
        ("tongue_doc", json!({ "doc": other, "of": page })),
        ("restore_doc", json!({ "doc": page })),
        ("edit_table", json!({ "doc": page, "do": "add_row" })),
        ("write_doc", json!({ "doc": page, "body": "# Noviembre\n" })),
        (
            "write_doc",
            json!({ "page_of": book, "body": "# Hoja\n\nAlgo.\n" }),
        ),
        (
            "import_doc",
            json!({ "page_of": book, "path": outside.to_string_lossy() }),
        ),
        (
            "attach",
            json!({ "doc": page, "path": picture.to_string_lossy() }),
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
}

#[test]
fn a_list_of_names_or_a_translation_meets_the_same_door() {
    let served = Served::new();
    let (book, page) = served.diary();
    let other = served.person().writes_doc("# Planes\n\nAlgo.\n");
    served.person().opens_the_diary_to_agents(true);

    for (name, args) in [
        (
            "flag_doc",
            json!({ "doc": [book], "reason": "obsolete", "body": "viejo" }),
        ),
        (
            "file_doc",
            json!({ "doc": [other, book], "folder": "Personal" }),
        ),
        ("page_doc", json!({ "doc": [page] })),
        (
            "flag_doc",
            json!({ "doc": other, "reason": "duplicate_of", "instead": page, "body": "viejo" }),
        ),
        (
            "write_doc",
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
        "append_doc",
        json!({ "doc": old_book, "body": "Jaime called." }),
    );

    assert!(refused(&added), "{added}");
    assert!(!served.body(&page).contains("Jaime called."));
}
