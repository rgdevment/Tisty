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

    fn sheet(&self) -> String {
        self.person().writes_doc(SHEET)
    }

    fn body(&self, file: &str) -> String {
        let root = self.home.path();
        let paths = tisty_core::Paths::new(root.join("data"), root.join("config"));
        tisty_core::docs::read(&paths.docs(), file).unwrap()
    }

    fn marks(&self, file: &str) -> Vec<tisty_core::model::FlaggedIdea> {
        let state = self.person().state();
        state
            .docs
            .values()
            .find(|one| one.file == file)
            .unwrap()
            .ideas_flagged
            .clone()
    }
}

fn said(result: &Value) -> &str {
    result["content"][0]["text"].as_str().unwrap()
}

fn refused(result: &Value) -> bool {
    result["isError"] == json!(true)
}

const SHEET: &str = "---\nview: notes\nbrainstorm: on\n---\n\n# Lluvia de ideas\n\n## 2026-06-14 08:12 · pink\n\nComprar entradas del concierto de julio\n\n## 2026-09-03 18:05 · teal\n\nPedir presupuesto para arreglar el techo\n\n## 2026-11-12 09:40 · yellow\n\nProbar pan con masa madre\n";

const CONCERT: &str = "2026-06-14 08:12 · pink";
const ROOF: &str = "2026-09-03 18:05 · teal";

#[test]
fn catch_up_names_brainstorm_and_how_many_ideas_wait_on_it() {
    let served = Served::new();
    let sheet = served.sheet();

    let caught = served.call("catch_up", json!({}));

    let glance = &caught["structuredContent"]["brainstorm"];
    assert_eq!(glance["doc"], json!(sheet), "{caught}");
    assert_eq!(glance["ideas"], json!(3));
    assert_eq!(glance["marked"], json!(0));
    assert!(
        said(&caught).contains("`rules` «ideas»"),
        "{}",
        said(&caught)
    );
}

#[test]
fn ideas_sent_to_brainstorm_land_each_on_its_own_dated_note() {
    let served = Served::new();
    let sheet = served.sheet();

    let jotted = served.call(
        "append_doc",
        json!({ "doc": sheet, "body": "- Llamar al vet\n- Revisar la bici\n\nUna idea\nen dos líneas" }),
    );

    assert!(!refused(&jotted), "{jotted}");
    assert_eq!(jotted["structuredContent"]["jotted"], json!(3));
    let body = served.body(&sheet);
    let ideas = tisty_core::brainstorm::ideas_on(&body);
    assert_eq!(ideas.len(), 6);
    assert_eq!(ideas[3].text, "Llamar al vet");
    assert_eq!(ideas[5].text, "Una idea\nen dos líneas");
    for pair in ideas.windows(2) {
        assert_ne!(
            pair[0].heading.rsplit_once(" · ").map(|(_, colour)| colour),
            pair[1].heading.rsplit_once(" · ").map(|(_, colour)| colour),
            "two notes side by side share a colour"
        );
    }
    assert!(body.starts_with(SHEET.trim_end()), "what was there changed");
}

#[test]
fn an_idea_past_the_window_s_limit_or_a_heading_writes_nothing() {
    let served = Served::new();
    let sheet = served.sheet();
    let long = "una idea ".repeat(20);

    let too_long = served.call(
        "append_doc",
        json!({ "doc": sheet, "body": format!("- corta\n- {long}") }),
    );
    let headed = served.call(
        "append_doc",
        json!({ "doc": sheet, "body": "## 2026-11-12 10:00 · blue\n\nidea" }),
    );
    let placed = served.call(
        "append_doc",
        json!({ "doc": sheet, "body": "idea", "under": ROOF }),
    );

    assert!(refused(&too_long), "{too_long}");
    assert!(said(&too_long).contains("140"), "{}", said(&too_long));
    assert!(refused(&headed), "{headed}");
    assert!(refused(&placed), "{placed}");
    assert_eq!(served.body(&sheet), SHEET);
}

#[test]
fn a_long_sheet_is_read_in_parts_and_never_as_its_outline_of_minutes() {
    let served = Served::new();
    let mut body = SHEET.to_string();
    for at in 0..200 {
        body.push_str(&format!(
            "\n## 2026-10-{:02} 10:{:02} · blue\n\nUna idea bastante larga para llenar la hoja, la número {at}\n",
            1 + at % 28,
            at % 60
        ));
    }
    let sheet = served.person().writes_doc(&body);

    let read = served.call("read_doc", json!({ "doc": sheet }));

    let kept = &read["structuredContent"];
    assert_eq!(kept["brainstorm"], json!(true), "{read}");
    assert!(
        kept.get("outline").is_none(),
        "the sheet came as its outline"
    );
    assert!(kept["body"].as_str().unwrap().contains("Comprar entradas"));
    assert!(kept["next"].as_u64().is_some());
    assert!(
        said(&read).contains("More ideas from line"),
        "{}",
        said(&read)
    );
}

#[test]
fn filing_an_idea_in_a_document_takes_it_off_the_sheet_as_the_window_does() {
    let served = Served::new();
    let sheet = served.sheet();
    let house = served
        .person()
        .writes_doc("# Casa\n\nArreglos pendientes.\n");

    let filed = served.call(
        "append_doc",
        json!({
            "doc": house,
            "body": "> [!NOTE] Lluvia de ideas · 2026-11-12\n> Pedir presupuesto para arreglar el techo",
        }),
    );

    assert!(!refused(&filed), "{filed}");
    assert_eq!(filed["structuredContent"]["idea_filed"], json!(true));
    assert!(
        served
            .body(&house)
            .contains("> Pedir presupuesto para arreglar el techo")
    );
    let left = tisty_core::brainstorm::ideas_on(&served.body(&sheet));
    assert_eq!(left.len(), 2);
    assert!(left.iter().all(|one| one.heading != ROOF));
}

#[test]
fn a_task_proposed_from_an_idea_takes_it_off_the_sheet_and_only_once() {
    let served = Served::new();
    let sheet = served.sheet();
    let source = format!("{sheet}#{ROOF}");

    let made = served.call(
        "propose",
        json!({ "title": "Pedir presupuesto para el techo", "source": source }),
    );
    let again = served.call(
        "propose",
        json!({ "title": "Presupuesto del techo", "source": source }),
    );

    assert!(!refused(&made), "{made}");
    assert!(made["structuredContent"]["from_idea"].is_string(), "{made}");
    let left = tisty_core::brainstorm::ideas_on(&served.body(&sheet));
    assert!(left.iter().all(|one| one.heading != ROOF));
    assert_eq!(
        again["structuredContent"]["proposed"],
        json!(false),
        "{again}"
    );
}

#[test]
fn an_idea_is_marked_for_the_person_and_stays_on_the_sheet() {
    let served = Served::new();
    let sheet = served.sheet();

    let marked = served.call(
        "flag_doc",
        json!({
            "doc": format!("{sheet}#{CONCERT}"),
            "reason": "obsolete",
            "body": "El concierto fue el 20 de julio.",
        }),
    );
    let twice = served.call(
        "flag_doc",
        json!({ "doc": format!("{sheet}#{CONCERT}"), "reason": "obsolete", "body": "otra vez" }),
    );

    assert!(!refused(&marked), "{marked}");
    assert!(said(&marked).contains("Comprar entradas del concierto de julio"));
    assert_eq!(served.body(&sheet), SHEET, "marking changed the sheet");
    let marks = served.marks(&sheet);
    assert_eq!(marks.len(), 1);
    assert_eq!(marks[0].idea.heading, CONCERT);
    assert_eq!(marks[0].mark.reason.as_deref(), Some("obsolete"));
    assert!(refused(&twice), "{twice}");
    assert!(
        served
            .person()
            .state()
            .docs
            .values()
            .all(|one| one.flagged.is_none())
    );

    let read = served.call("read_doc", json!({ "doc": sheet }));
    assert_eq!(
        read["structuredContent"]["marked"][0]["idea"],
        json!(CONCERT),
        "{read}"
    );
}

#[test]
fn a_copy_of_an_idea_points_at_the_idea_that_stays() {
    let served = Served::new();
    let sheet = served.sheet();

    let marked = served.call(
        "flag_doc",
        json!({
            "doc": [format!("{sheet}#{CONCERT}")],
            "reason": "duplicate_of",
            "instead": format!("{sheet}#{ROOF}"),
            "body": "Dice lo mismo.",
        }),
    );
    let pointless = served.call(
        "flag_doc",
        json!({ "doc": format!("{sheet}#{ROOF}"), "reason": "duplicate_of", "body": "sin destino" }),
    );

    assert!(!refused(&marked), "{marked}");
    assert!(
        said(&marked).contains("a duplicate of the idea «Pedir presupuesto"),
        "{}",
        said(&marked)
    );
    assert_eq!(
        served.marks(&sheet)[0].mark.instead,
        Some(format!("{sheet}#{ROOF}"))
    );
    assert!(refused(&pointless), "{pointless}");
}

#[test]
fn the_whole_sheet_is_never_marked_and_an_unknown_heading_marks_nothing() {
    let served = Served::new();
    let sheet = served.sheet();

    let whole = served.call(
        "flag_doc",
        json!({ "doc": sheet, "reason": "scratch", "body": "viejo" }),
    );
    let unknown = served.call(
        "flag_doc",
        json!({ "doc": format!("{sheet}#2026-01-01 00:00 · blue"), "reason": "scratch", "body": "x" }),
    );

    assert!(refused(&whole), "{whole}");
    assert!(said(&whole).contains("Mark the ideas"), "{}", said(&whole));
    assert!(refused(&unknown), "{unknown}");
    assert!(served.marks(&sheet).is_empty());
    assert!(
        served
            .person()
            .state()
            .docs
            .values()
            .all(|one| one.flagged.is_none())
    );
}

#[test]
fn a_write_that_would_take_an_idea_off_the_sheet_is_refused() {
    let served = Served::new();
    let sheet = served.sheet();

    let cut = served.call(
        "edit_doc",
        json!({
            "doc": sheet,
            "old": "## 2026-06-14 08:12 · pink\n\nComprar entradas del concierto de julio\n\n",
            "new": "",
        }),
    );
    let print = tisty_core::attach::printed(SHEET.as_bytes());
    let rewritten = served.call(
        "write_doc",
        json!({ "doc": sheet, "print": print, "body": SHEET.replace("## 2026-11-12 09:40 · yellow\n\nProbar pan con masa madre\n", "") }),
    );
    let reworded = served.call(
        "edit_doc",
        json!({ "doc": sheet, "old": "Probar pan con masa madre", "new": "Probar pan con masa madre el domingo" }),
    );

    assert!(refused(&cut), "{cut}");
    assert!(said(&cut).contains("flag_doc"), "{}", said(&cut));
    assert!(refused(&rewritten), "{rewritten}");
    assert!(!refused(&reworded), "{reworded}");
    assert_eq!(
        tisty_core::brainstorm::ideas_on(&served.body(&sheet)).len(),
        3
    );
}

#[test]
fn sorting_ideas_names_the_sheet_and_changes_nothing_until_the_person_says() {
    let served = Served::new();
    let sheet = served.sheet();
    let said = served.ask(
        "prompts/get",
        json!({ "name": "sort_ideas", "arguments": {} }),
    );
    let text = said["messages"][0]["content"]["text"].as_str().unwrap();

    assert!(
        text.contains(&format!("\"Lluvia de ideas\" [{sheet}]")),
        "{text}"
    );
    assert!(text.contains("Change nothing until I say which"), "{text}");
    assert_eq!(served.body(&sheet), SHEET);
}

#[test]
fn a_sheet_written_on_windows_still_keeps_every_idea() {
    let served = Served::new();
    let sheet = served.person().writes_doc(&SHEET.replace('\n', "\r\n"));

    let cut = served.call(
        "edit_doc",
        json!({
            "doc": sheet,
            "old": "## 2026-06-14 08:12 · pink\n\nComprar entradas del concierto de julio\n\n",
            "new": "",
        }),
    );

    assert!(refused(&cut), "{cut}");
    assert_eq!(served.body(&sheet), SHEET.replace('\n', "\r\n"));
}

#[test]
fn nothing_but_ideas_is_added_to_the_sheet_and_it_never_holds_a_page() {
    let served = Served::new();
    let sheet = served.sheet();
    let loose = served.home.path().join("plano.png");
    std::fs::write(&loose, b"\x89PNG\r\n\x1a\nthe drawing").unwrap();
    let other = served
        .person()
        .writes_doc("# Casa\n\nArreglos pendientes.\n");

    let attached = served.call(
        "attach",
        json!({ "doc": sheet, "path": loose.to_string_lossy() }),
    );
    let hung = served.call("page_doc", json!({ "doc": other, "page_of": sheet }));
    let hanging = served.call("page_doc", json!({ "doc": sheet, "page_of": other }));
    let written = served.call(
        "write_doc",
        json!({ "body": "# Un anexo\n\nAlgo largo.\n", "page_of": sheet }),
    );

    for one in [&attached, &hung, &hanging, &written] {
        assert!(refused(one), "{one}");
        assert!(said(one).contains("Brainstorm"), "{}", said(one));
    }
    assert_eq!(served.body(&sheet), SHEET);
}

#[test]
fn going_back_over_the_persons_new_ideas_is_refused() {
    let served = Served::new();
    let sheet = served.sheet();
    let reworded = served.call(
        "edit_doc",
        json!({ "doc": sheet, "old": "Probar pan con masa madre", "new": "Probar pan de masa madre" }),
    );
    let now = served.body(&sheet);
    served.person().saves_doc(
        &sheet,
        &format!("{now}\n## 2026-11-12 11:00 · blue\n\nLlamar al vet\n"),
    );

    let back = served.call("restore_doc", json!({ "doc": sheet, "even_if_more": true }));

    assert!(!refused(&reworded), "{reworded}");
    assert!(refused(&back), "{back}");
    assert!(served.body(&sheet).contains("Llamar al vet"));
}

#[test]
fn an_idea_may_open_with_a_tag() {
    let served = Served::new();
    let sheet = served.sheet();

    let jotted = served.call(
        "append_doc",
        json!({ "doc": sheet, "body": "#casa pintar la reja" }),
    );

    assert!(!refused(&jotted), "{jotted}");
    let ideas = tisty_core::brainstorm::ideas_on(&served.body(&sheet));
    assert_eq!(ideas.last().unwrap().text, "#casa pintar la reja");
}

#[test]
fn a_second_sheet_is_guarded_and_its_ideas_are_marked_like_the_first() {
    let served = Served::new();
    served.sheet();
    let second = served.person().writes_doc(
        "---\nbrainstorm: on\n---\n\n# Ideas del trabajo\n\n## 2026-11-10 10:00 · lime\n\nRevisar la factura\n",
    );

    let whole = served.call(
        "flag_doc",
        json!({ "doc": second, "reason": "scratch", "body": "viejo" }),
    );
    let one = served.call(
        "flag_doc",
        json!({
            "doc": format!("{second}#2026-11-10 10:00 · lime"),
            "reason": "obsolete",
            "body": "Ya se pagó.",
        }),
    );

    assert!(refused(&whole), "{whole}");
    assert!(!refused(&one), "{one}");
    assert_eq!(served.marks(&second).len(), 1);
}

#[test]
fn an_idea_quoted_on_a_tasks_page_leaves_the_sheet_too() {
    let served = Served::new();
    let sheet = served.sheet();
    let task = served.call("propose", json!({ "title": "Arreglar la casa" }))["structuredContent"]
        ["id"]
        .as_str()
        .unwrap()
        .to_string();

    let filed = served.call(
        "append_doc",
        json!({
            "doc": task,
            "body": "> [!NOTE] Lluvia de ideas · 2026-11-12\n> Pedir presupuesto para arreglar el techo",
        }),
    );

    assert!(!refused(&filed), "{filed}");
    assert_eq!(
        filed["structuredContent"]["idea_filed"],
        json!(true),
        "{filed}"
    );
    let left = tisty_core::brainstorm::ideas_on(&served.body(&sheet));
    assert!(left.iter().all(|one| one.heading != ROOF));
}

#[test]
fn a_source_with_a_hash_that_is_no_idea_files_as_ever() {
    let served = Served::new();
    let sheet = served.sheet();

    let made = served.call(
        "propose",
        json!({ "title": "Contestar a la vecina", "source": "sereno#1" }),
    );

    assert!(!refused(&made), "{made}");
    assert!(
        made["structuredContent"].get("from_idea").is_none(),
        "{made}"
    );
    assert_eq!(served.body(&sheet), SHEET);
}
