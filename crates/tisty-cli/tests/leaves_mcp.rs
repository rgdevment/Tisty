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
