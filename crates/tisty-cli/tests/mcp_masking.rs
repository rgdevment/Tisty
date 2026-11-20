use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

const SECRET: &str = "secret zebra";
const MASK: &str = "a task out of sight";
const GONE: &str = "01J00000000000000000000000";

struct Served {
    home: TempDir,
    hidden: String,
    shown: String,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        let mut served = Self {
            home,
            hidden: String::new(),
            shown: String::new(),
        };
        served.hidden = served.filed("secret zebra plan");
        served.shown = served.filed("Pagar la luz");
        served.person().hides(person::task(&served.hidden));
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn filed(&self, title: &str) -> String {
        let made = self.call("propose", json!({ "title": title, "list": "Personal" }));
        made["structuredContent"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn naming(&self, task: &str) -> String {
        format!(
            "# Plan\n\nVer ![secret zebra plan](tisty:task/{task}) antes.\n\nY [Pagar la luz](tisty:task/{}).\n",
            self.shown
        )
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

    fn read(&self, doc: &str) -> Value {
        self.call("read", json!({ "doc": doc }))["structuredContent"].clone()
    }
}

fn untold(said: &Value) {
    assert!(!said.to_string().contains(SECRET), "{said}");
}

#[test]
fn a_hidden_task_named_in_a_document_reads_like_one_never_there_by_every_way_in() {
    let served = Served::new();
    let doc = served.person().writes_doc(&served.naming(&served.hidden));
    let twin = served.person().writes_doc(&served.naming(GONE));
    let into = served.home.path().join("out");
    std::fs::create_dir_all(&into).unwrap();

    let read = served.read(&doc);
    untold(&read);
    let body = read["body"].as_str().unwrap();
    assert!(
        body.contains(&format!("![{MASK}](tisty:task/{})", served.hidden)),
        "{body}"
    );
    assert!(
        body.contains("[Pagar la luz]"),
        "a task in sight keeps its name: {body}"
    );
    let twin_body = served.read(&twin)["body"].as_str().unwrap().to_string();
    assert_eq!(
        body.replace(&served.hidden, "X"),
        twin_body.replace(GONE, "X"),
        "a hidden task is told from one never there"
    );

    for (name, args) in [
        ("read", json!({ "doc": doc, "from": 3, "to": 3 })),
        ("read", json!({ "doc": doc, "section": 0 })),
        ("brain", json!({ "doc": doc })),
        ("brain", json!({ "doc": doc, "query": "antes" })),
        ("brain", json!({ "query": "antes" })),
        (
            "export_doc",
            json!({ "doc": doc, "into": into.to_string_lossy() }),
        ),
        (
            "edit_doc",
            json!({ "doc": doc, "old": "nada así", "new": "x" }),
        ),
    ] {
        untold(&served.call(name, args));
    }
    untold(&served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}") }),
    ));
    for one in walked(&into) {
        let text = std::fs::read_to_string(&one).unwrap();
        assert!(!text.contains(SECRET), "{}: {text}", one.display());
    }
}

fn walked(at: &std::path::Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(at)
        .unwrap()
        .flatten()
        .flat_map(|one| match one.path().is_dir() {
            true => walked(&one.path()),
            false => vec![one.path()],
        })
        .collect()
}

#[test]
fn a_task_s_own_words_naming_a_hidden_one_wear_the_mask_too() {
    let served = Served::new();
    served.person().describes(
        person::task(&served.shown),
        &format!(
            "Después de [secret zebra plan](tisty:task/{}).",
            served.hidden
        ),
    );

    for (name, args) in [
        ("read", json!({ "task": served.shown })),
        ("brain", json!({ "query": "despues" })),
        ("brain", json!({ "task": served.shown })),
    ] {
        let said = served.call(name, args.clone());
        assert_ne!(said["isError"], true, "{name} {args}: {said}");
        untold(&said);
    }
}

#[test]
fn a_description_read_masked_is_described_back_with_the_person_s_words() {
    let served = Served::new();
    let was = format!(
        "Después de [secret zebra plan](tisty:task/{}).",
        served.hidden
    );
    served.person().describes(person::task(&served.shown), &was);

    let read = served.call("read", json!({ "task": served.shown }));
    let shown = read["structuredContent"]["description"].as_str().unwrap();
    let described = served.call(
        "describe",
        json!({ "task": served.shown, "body": format!("{shown} Hoy.") }),
    );

    assert_ne!(described["isError"], true, "{described}");
    untold(&described);
    let state = served.person().state();
    let now = state.tasks[&person::task(&served.shown)]
        .description
        .clone()
        .unwrap_or_default();
    assert_eq!(now, format!("{was} Hoy."));
}

#[test]
fn what_was_read_masked_is_written_back_byte_for_byte() {
    let served = Served::new();
    let was = served.naming(&served.hidden);
    let doc = served.person().writes_doc(&was);

    let read = served.read(&doc);
    let shown = read["body"].as_str().unwrap().to_string();
    let rewritten = served.call(
        "write_doc",
        json!({ "doc": doc, "print": read["print"], "body": shown }),
    );
    assert_ne!(rewritten["isError"], true, "{rewritten}");
    untold(&rewritten);
    assert_eq!(
        served.body(&doc),
        was,
        "the whole write lost the person's words"
    );

    let line = format!("Ver ![{MASK}](tisty:task/{}) antes.", served.hidden);
    let edited = served.call(
        "edit_doc",
        json!({ "doc": doc, "old": line, "new": format!("{line} Hoy.") }),
    );
    assert_ne!(edited["isError"], true, "{edited}");
    untold(&edited);
    assert!(
        served.body(&doc).contains(&format!(
            "Ver ![secret zebra plan](tisty:task/{}) antes. Hoy.",
            served.hidden
        )),
        "{}",
        served.body(&doc)
    );

    let added = served.call(
        "append_doc",
        json!({ "doc": doc, "body": format!("Otra vez [{MASK}](tisty:task/{}).", served.hidden) }),
    );
    assert_ne!(added["isError"], true, "{added}");
    assert!(
        served.body(&doc).contains(&format!(
            "Otra vez [secret zebra plan](tisty:task/{}).",
            served.hidden
        )),
        "{}",
        served.body(&doc)
    );
    untold(&served.read(&doc));
}

#[test]
fn a_version_read_back_wears_the_same_mask() {
    let served = Served::new();
    let doc = served.person().writes_doc(&served.naming(&served.hidden));
    let edited = served.call(
        "edit_doc",
        json!({ "doc": doc, "old": "antes.", "new": "luego." }),
    );
    assert_ne!(edited["isError"], true, "{edited}");
    let versions = served.call("read", json!({ "doc": doc, "versions": true }));
    untold(&versions);
    let first = versions["structuredContent"]["versions"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()["version"]
        .as_str()
        .unwrap()
        .to_string();

    let old = served.call("read", json!({ "doc": doc, "version": first }));

    assert_ne!(old["isError"], true, "{old}");
    untold(&old);
}
