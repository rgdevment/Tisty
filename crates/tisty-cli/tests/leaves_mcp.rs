use std::io::Write;
use std::process::{Command, Stdio};

use tempfile::TempDir;

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let served = Self {
            home: tempfile::tempdir().unwrap(),
        };
        served.cli(&["algo mio"]);
        served.cli(&["agent", "--on"]);
        served
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

    fn cli(&self, args: &[&str]) -> String {
        let out = self
            .command()
            .args(args)
            .stdin(Stdio::null())
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    fn call(&self, name: &str, args: serde_json::Value) -> serde_json::Value {
        let said = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": name, "arguments": args },
        })
        .to_string();
        self.talk(&[
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": { "protocolVersion": "2025-06-18", "capabilities": {},
                            "clientInfo": { "name": "test", "version": "1" } },
            })
            .to_string(),
            &said,
        ])
        .into_iter()
        .find(|one| one["id"] == 2)
        .unwrap()
    }

    fn talk(&self, said: &[&str]) -> Vec<serde_json::Value> {
        let mut child = self
            .command()
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        {
            let mut asks = child.stdin.take().unwrap();
            for one in said {
                writeln!(asks, "{one}").unwrap();
            }
        }
        let out = child.wait_with_output().unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect()
    }

    fn wrote(&self, body: &str, page_of: Option<&str>) -> String {
        let args = match page_of {
            Some(up) => serde_json::json!({ "body": body, "page_of": up }),
            None => serde_json::json!({ "body": body }),
        };
        self.call("write_doc", args)["result"]["structuredContent"]["doc"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn body_of(&self, doc: &str) -> String {
        self.call("read_doc", serde_json::json!({ "doc": doc }))["result"]["structuredContent"]
            ["body"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }

    fn pages_of(&self, doc: &str) -> Vec<String> {
        let said = self.call("read_doc", serde_json::json!({ "doc": doc }));
        said["result"]["structuredContent"]["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|one| one.as_str().unwrap().to_string())
            .collect()
    }

    fn data(&self) -> std::path::PathBuf {
        self.home.path().join("data")
    }
}

fn copied(from: &std::path::Path, into: &std::path::Path) {
    std::fs::create_dir_all(into).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let at = into.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copied(&entry.path(), &at);
        } else {
            std::fs::copy(entry.path(), at).unwrap();
        }
    }
}

#[test]
fn a_page_written_by_an_assistant_is_named_at_the_end_of_its_document() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nlas de este año.", None);
    let page = served.wrote("# Marzo\n\nlo que se dijo.", Some(&book));

    let body = served.body_of(&book);
    assert!(
        body.contains(&format!("![Marzo](tisty:doc/{page})")),
        "the document has to say where the page goes: {body}"
    );
    assert!(
        body.starts_with("# Actas\n\nlas de este año."),
        "and nothing that was written before may be touched: {body}"
    );
}

#[test]
fn a_page_titled_with_brackets_still_sits_where_its_document_names_it() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nlas de este ano.", None);
    let one = served.wrote("# Capitulo 1 [borrador]", Some(&book));
    let two = served.wrote("# Capitulo 2", Some(&book));

    assert_eq!(
        served.pages_of(&book),
        vec![one, two],
        "a title with brackets must not shove the page to the end"
    );
}

#[test]
fn moving_the_line_that_names_a_page_moves_the_page() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nlas de este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let two = served.wrote("# Abril", Some(&book));

    assert_eq!(served.pages_of(&book), vec![one.clone(), two.clone()]);

    let said = served.call(
        "edit_doc",
        serde_json::json!({
            "doc": book,
            "old": format!("![Marzo](tisty:doc/{one})\n\n![Abril](tisty:doc/{two})"),
            "new": format!("![Abril](tisty:doc/{two})\n\n![Marzo](tisty:doc/{one})"),
        }),
    );

    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert_eq!(served.pages_of(&book), vec![two, one]);
}

#[test]
fn hanging_a_document_as_a_page_with_page_doc_lands_it_last_until_the_text_names_it() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nde este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let loose = served.wrote("# Suelto\n\nun documento aparte.", None);

    let said = served.call(
        "page_doc",
        serde_json::json!({ "doc": loose, "page_of": book }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");

    assert_eq!(served.pages_of(&book), vec![one, loose.clone()]);

    let body = served.body_of(&book);
    assert!(
        !body.contains(&loose),
        "hanging it as a page does not by itself name it in the text: {body}"
    );
}

#[test]
fn naming_a_hung_page_in_the_text_with_edit_doc_moves_it_from_the_end_to_where_it_is_named() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nde este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let two = served.wrote("# Abril", Some(&book));
    let loose = served.wrote("# Enero\n\nun documento aparte.", None);

    served.call(
        "page_doc",
        serde_json::json!({ "doc": loose, "page_of": book }),
    );
    assert_eq!(
        served.pages_of(&book),
        vec![one.clone(), two.clone(), loose.clone()]
    );

    let old = format!("![Marzo](tisty:doc/{one})\n\n");
    let new = format!("![Enero](tisty:doc/{loose})\n\n{old}");
    let said = served.call(
        "edit_doc",
        serde_json::json!({ "doc": book, "old": old, "new": new }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");

    assert_eq!(served.pages_of(&book), vec![loose, one, two]);
}

#[test]
fn append_doc_settles_the_order_of_two_pages_that_were_never_named_before() {
    let served = Served::new();
    let book = served.wrote("# Actas\n\nde este año.", None);
    let one = served.wrote("# Marzo", Some(&book));
    let a = served.wrote("# Suelto A\n\ncontenido.", None);
    let b = served.wrote("# Suelto B\n\ncontenido.", None);
    served.call("page_doc", serde_json::json!({ "doc": a, "page_of": book }));
    served.call("page_doc", serde_json::json!({ "doc": b, "page_of": book }));

    assert_eq!(
        served.pages_of(&book),
        vec![one.clone(), a.clone(), b.clone()]
    );

    let said = served.call(
        "append_doc",
        serde_json::json!({
            "doc": book,
            "body": format!("![B](tisty:doc/{b})\n\n![A](tisty:doc/{a})\n"),
        }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");

    assert_eq!(served.pages_of(&book), vec![one, b, a]);
}

#[test]
fn a_page_order_pulled_in_from_another_machine_settles_to_match_this_machines_own_text_on_the_next_write()
 {
    let here = Served::new();
    let book = here.wrote("# Actas\n\nde este ano.", None);
    let one = here.wrote("# Marzo", Some(&book));
    let two = here.wrote("# Abril", Some(&book));
    assert_eq!(here.pages_of(&book), vec![one.clone(), two.clone()]);

    // A second machine pulls this store, then swaps the two pages on its own, offline.
    let there = Served::new();
    copied(&here.data().join("docs"), &there.data().join("docs"));
    copied(&here.data().join("store"), &there.data().join("store"));
    let old = format!("![Marzo](tisty:doc/{one})\n\n![Abril](tisty:doc/{two})");
    let new = format!("![Abril](tisty:doc/{two})\n\n![Marzo](tisty:doc/{one})");
    let said = there.call(
        "edit_doc",
        serde_json::json!({ "doc": book, "old": old, "new": new }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert_eq!(there.pages_of(&book), vec![two.clone(), one.clone()]);

    // Pulling that swap back into the first machine's own store, without touching its own copy
    // of the document's text, leaves the tree and the visible text disagreeing about the order.
    for device in std::fs::read_dir(there.data().join("store")).unwrap() {
        let device = device.unwrap();
        copied(
            &device.path(),
            &here.data().join("store").join(device.file_name()),
        );
    }
    assert_eq!(
        here.pages_of(&book),
        vec![two.clone(), one.clone()],
        "the log now carries the other machine's move"
    );
    assert_eq!(
        here.body_of(&book),
        format!("# Actas\n\nde este ano.\n\n{old}\n"),
        "but this machine's own file on disk still reads the way it always did"
    );

    // The next write on this machine settles the order back to what its own text says.
    let said = here.call(
        "append_doc",
        serde_json::json!({ "doc": book, "body": "Fin." }),
    );
    assert!(said["result"]["isError"].as_bool() != Some(true), "{said}");
    assert_eq!(
        here.pages_of(&book),
        vec![one, two],
        "saving settles the order back to what the text in front of the person says"
    );
}
