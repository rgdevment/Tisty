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
        served.person().begins("algo mio");
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn talk(&self, asked: &[(&str, Value)]) -> Vec<Value> {
        let root = self.home.path();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
            .arg("mcp")
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        {
            let mut pipe = child.stdin.take().unwrap();
            for (at, (method, params)) in asked.iter().enumerate() {
                let one =
                    json!({ "jsonrpc": "2.0", "id": at + 1, "method": method, "params": params });
                writeln!(pipe, "{one}").unwrap();
            }
        }
        let out = child.wait_with_output().unwrap();
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect(line))
            .collect()
    }

    fn ask(&self, method: &str, params: Value) -> Value {
        self.talk(&[(method, params)]).remove(0)
    }

    fn call(&self, name: &str, arguments: Value) -> Value {
        let said = self.ask(
            "tools/call",
            json!({ "name": name, "arguments": arguments }),
        );
        assert_ne!(said["result"]["isError"], true, "{said}");
        said["result"]["structuredContent"].clone()
    }

    fn prompted(&self, name: &str, arguments: Value) -> String {
        let said = self.ask(
            "prompts/get",
            json!({ "name": name, "arguments": arguments }),
        );
        let messages = said["result"]["messages"].as_array().expect("messages");
        assert_eq!(messages.len(), 1, "{said}");
        assert_eq!(messages[0]["role"], "user", "{said}");
        messages[0]["content"]["text"].as_str().unwrap().to_string()
    }

    fn written(&self, body: &str) -> String {
        self.call("write_doc", json!({ "body": body }))["doc"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn knocked(&self) -> bool {
        self.home
            .path()
            .join("config/private/knocked.json")
            .exists()
    }
}

fn read_back(said: &Value) -> (&str, &str) {
    let one = &said["result"]["contents"][0];
    (
        one["mimeType"].as_str().unwrap(),
        one["text"].as_str().unwrap(),
    )
}

#[test]
fn every_version_it_speaks_is_told_it_offers_prompts_and_resources_beside_tools() {
    let served = Served::new();
    let versions = ["2026-07-28", "2025-11-25", "2025-06-18"];

    let discovered = served.ask("server/discover", json!({}));
    assert_eq!(
        discovered["result"]["supportedVersions"],
        json!(versions),
        "{discovered}"
    );
    let offered = json!({ "tools": {}, "prompts": {}, "resources": {} });
    assert_eq!(discovered["result"]["capabilities"], offered);

    for version in versions {
        let greeted = served.ask(
            "initialize",
            json!({ "protocolVersion": version, "capabilities": {},
                    "clientInfo": { "name": "old", "version": "1" } }),
        );
        assert_eq!(greeted["result"]["protocolVersion"], version, "{greeted}");
        assert_eq!(greeted["result"]["capabilities"], offered, "{greeted}");
    }
}

#[test]
fn the_prompts_a_person_is_offered_as_commands() {
    let served = Served::new();

    let said = served.ask("prompts/list", json!({}));

    insta::assert_json_snapshot!(said["result"]);
}

#[test]
fn every_prompt_takes_at_most_one_word_and_works_without_it() {
    let served = Served::new();
    let said = served.ask("prompts/list", json!({}));

    for prompt in said["result"]["prompts"].as_array().unwrap() {
        let name = prompt["name"].as_str().unwrap();
        let takes = prompt["arguments"].as_array().unwrap();
        assert_eq!(
            takes.len(),
            1,
            "a client splits what is typed on spaces: {prompt}"
        );
        assert_ne!(takes[0]["required"], true, "{prompt}");

        let bare = served.prompted(name, json!({}));
        let given = served.prompted(
            name,
            json!({ takes[0]["name"].as_str().unwrap(): "passport" }),
        );
        assert!(bare.contains('`'), "a prompt names the tools: {bare}");
        assert!(given.contains("«passport»"), "{given}");
        assert!(!bare.contains("passport"), "{bare}");
    }
}

#[test]
fn the_words_each_prompt_hands_the_agent() {
    let served = Served::new();
    let mut said = String::new();
    for (name, takes) in [
        ("plan_task", "work"),
        ("solved_before", "problem"),
        ("keep_decision", "subject"),
        ("leave_off", "task"),
        ("pick_up", "task"),
        ("brainstorm", "subject"),
        ("sort_ideas", "subject"),
    ] {
        let given = served.prompted(name, json!({ takes: "the passport" }));
        said.push_str(&format!("## {name} «the passport»\n\n{given}\n\n"));
        if name != "pick_up" {
            let bare = served.prompted(name, Value::Null);
            said.push_str(&format!("## {name}\n\n{bare}\n\n"));
        }
    }

    insta::assert_snapshot!(said);
}

#[test]
fn picking_up_names_the_open_task_the_agent_last_worked_on() {
    let served = Served::new();
    served.person().lets_the_agent_in();
    let passport = served.call("propose", json!({ "title": "Renew the passport" }))["id"]
        .as_str()
        .unwrap()
        .to_string();
    let stamps = served.call("propose", json!({ "title": "Buy stamps" }))["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.call(
        "note",
        json!({ "task": passport, "body": "The photo is ready." }),
    );

    let said = served.prompted("pick_up", json!({}));
    assert!(said.contains("`catch_up`"), "{said}");
    assert!(
        said.contains(&format!("\"Renew the passport\" in the inbox [{passport}]")),
        "{said}"
    );
    let leaving = served.prompted("leave_off", json!({}));
    assert!(
        leaving.contains(&format!("\"Renew the passport\" in the inbox [{passport}]")),
        "{leaving}"
    );

    served.person().closes(person::task(&passport));

    let said = served.prompted("pick_up", json!({}));
    assert!(
        !said.contains("Buy stamps"),
        "the task it was on is closed, and an older one does not stand in for it: {said}"
    );
    assert!(!said.contains("Renew the passport"), "{said}");
    assert!(said.contains("the open task we were on"), "{said}");

    served.call(
        "note",
        json!({ "task": stamps, "body": "Ten, at the corner shop." }),
    );
    let said = served.prompted("pick_up", json!({}));
    assert!(
        said.contains("\"Buy stamps\""),
        "working on it again makes it the one: {said}"
    );

    served.call("say_done", json!({ "task": stamps, "body": "Bought ten." }));

    let said = served.prompted("pick_up", json!({}));
    assert!(
        !said.contains("Buy stamps"),
        "what it said is done waits for the person, not for more work: {said}"
    );
    assert!(said.contains("the open task we were on"), "{said}");
}

#[test]
fn picking_up_before_an_agent_is_let_in_does_not_knock() {
    let served = Served::new();

    let said = served.prompted("pick_up", json!({}));

    assert!(said.contains("the open task we were on"), "{said}");
    assert!(!served.knocked());
}

#[test]
fn a_prompt_it_does_not_know_or_an_argument_it_does_not_take_is_refused() {
    let served = Served::new();

    let unknown = served.ask("prompts/get", json!({ "name": "plan" }));
    let misnamed = served.ask(
        "prompts/get",
        json!({ "name": "plan_task", "arguments": { "task": "x" } }),
    );
    let numbered = served.ask(
        "prompts/get",
        json!({ "name": "plan_task", "arguments": { "work": 3 } }),
    );

    for refused in [&unknown, &misnamed, &numbered] {
        assert_eq!(refused["error"]["code"], -32602, "{refused}");
    }
    assert!(
        unknown["error"]["message"]
            .as_str()
            .unwrap()
            .contains("plan_task"),
        "{unknown}"
    );
}

#[test]
fn before_an_agent_is_let_in_nothing_is_listed_and_listing_does_not_knock() {
    let served = Served::new();
    served.person().writes_doc("# Recipes\n\nRice.");

    let said = served.ask("resources/list", json!({}));

    assert_eq!(said["result"]["resources"], json!([]), "{said}");
    assert!(said["result"].get("nextCursor").is_none(), "{said}");
    assert!(!served.knocked());
}

#[test]
fn every_document_not_put_away_is_listed_newest_first_with_where_it_sits() {
    let served = Served::new();
    served.person().lets_the_agent_in();
    served.call("folder", json!({ "name": "Kitchen" }));
    let recipes = served.call(
        "write_doc",
        json!({ "body": "# Recipes\n\nRice.", "folder": "Kitchen" }),
    )["doc"]
        .as_str()
        .unwrap()
        .to_string();
    let old = served.written("# Old plan\n\nGone.");
    let plan = served.written("# Release plan\n\nShip it.");
    served.call("archive_doc", json!({ "doc": old }));

    let said = served.ask("resources/list", json!({}));

    assert_eq!(
        said["result"]["resources"],
        json!([
            { "uri": format!("tisty:doc/{plan}"), "name": "Release plan", "mimeType": "text/markdown" },
            { "uri": format!("tisty:doc/{recipes}"), "name": "Recipes", "mimeType": "text/markdown",
              "description": "Recipes — in Kitchen" },
        ]),
        "{said}"
    );
    assert_eq!(said["result"]["cacheScope"], "private");
}

#[test]
fn a_long_list_goes_on_with_the_cursor_it_hands_back() {
    let served = Served::new();
    served.person().lets_the_agent_in();
    let writes: Vec<(&str, Value)> = (0..51)
        .map(|n| {
            (
                "tools/call",
                json!({ "name": "write_doc", "arguments": { "body": format!("# Note {n}\n\nOne.") } }),
            )
        })
        .collect();
    let wrote = served.talk(&writes);
    let oldest = wrote[0]["result"]["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();

    let first = served.ask("resources/list", json!({}));
    let cursor = first["result"]["nextCursor"].as_str().expect("a cursor");
    served.written("# Written between pages\n\nNew.");
    served.call(
        "append_doc",
        json!({ "doc": oldest, "body": "Edited between pages." }),
    );
    let rest = served.ask("resources/list", json!({ "cursor": cursor }));
    let wrong = served.ask("resources/list", json!({ "cursor": "page two" }));

    assert_eq!(first["result"]["resources"].as_array().unwrap().len(), 50);
    assert_eq!(rest["result"]["resources"].as_array().unwrap().len(), 1);
    assert_eq!(rest["result"]["resources"][0]["name"], "Note 0", "{rest}");
    assert!(rest["result"].get("nextCursor").is_none(), "{rest}");
    assert_eq!(wrong["error"]["code"], -32602, "{wrong}");
}

#[test]
fn a_short_document_is_read_whole_as_markdown() {
    let served = Served::new();
    served.person().lets_the_agent_in();
    let doc = served.written("# Recipes\n\nRice with *beans*.");
    let uri = format!("tisty:doc/{doc}");

    let said = served.ask("resources/read", json!({ "uri": uri }));

    let (mime, text) = read_back(&said);
    assert_eq!(said["result"]["contents"][0]["uri"], uri.as_str());
    assert_eq!(mime, "text/markdown");
    assert!(text.starts_with("# Recipes"), "{text}");
    assert!(text.contains("Rice with *beans*."), "{text}");
}

#[test]
fn a_long_document_comes_as_its_outline_and_never_poured_whole() {
    let served = Served::new();
    served.person().lets_the_agent_in();
    let filler = "Words that only fill the page. ".repeat(12);
    let mut body = String::from("# Handbook\n\n");
    for n in 1..=20 {
        body.push_str(&format!("## Chapter {n}\n\n"));
        for line in 0..3 {
            body.push_str(&format!("Paragraph {n}.{line}: {filler}\n\n"));
        }
    }
    assert!(body.chars().count() > 12_000);
    let doc = served.written(&body);

    let outline = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}") }),
    );
    let (mime, text) = read_back(&outline);
    assert_eq!(mime, "text/plain");
    assert!(text.contains("Chapter 20"), "{text}");
    assert!(!text.contains("Paragraph 20.2"), "{text}");
    assert!(text.contains(&format!("tisty:doc/{doc}?from=")), "{text}");

    let part = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}?from=3&to=9") }),
    );
    let (mime, text) = read_back(&part);
    assert_eq!(mime, "text/markdown");
    assert!(text.starts_with("## Chapter 1"), "{text}");
    assert!(!text.contains("Chapter 2"), "{text}");

    let all = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}?from=1") }),
    );
    let (_, text) = read_back(&all);
    assert!(!text.contains("Paragraph 20.2"), "{text}");
    assert!(
        text.contains("it goes on at") && text.contains(&format!("tisty:doc/{doc}?from=")),
        "{text}"
    );
}

#[test]
fn an_address_that_leads_nowhere_is_refused_with_why() {
    let served = Served::new();
    served.person().lets_the_agent_in();
    let doc = served.written("# Recipes\n\nRice.");

    let unknown = served.ask(
        "resources/read",
        json!({ "uri": "tisty:doc/q7ntmzbm-0001" }),
    );
    let elsewhere = served.ask("resources/read", json!({ "uri": format!("file:///{doc}") }));
    let paged = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}?page=2") }),
    );
    let zero = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}?from=0") }),
    );
    let backwards = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}?from=3&to=1") }),
    );

    assert_eq!(unknown["error"]["code"], -32002, "{unknown}");
    assert_eq!(elsewhere["error"]["code"], -32002, "{elsewhere}");
    assert_eq!(paged["error"]["code"], -32602, "{paged}");
    assert_eq!(zero["error"]["code"], -32602, "{zero}");
    assert_eq!(backwards["error"]["code"], -32602, "{backwards}");
}

#[test]
fn reading_a_document_before_an_agent_is_let_in_says_who_turns_one_on() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Recipes\n\nRice.");

    let said = served.ask(
        "resources/read",
        json!({ "uri": format!("tisty:doc/{doc}") }),
    );

    assert!(
        said["error"]["message"]
            .as_str()
            .unwrap()
            .contains("no agent is registered"),
        "{said}"
    );
    assert_eq!(
        said["error"]["code"], -32603,
        "the document is there: {said}"
    );
}

#[test]
fn the_template_says_the_address_a_document_and_its_lines_are_read_at() {
    let served = Served::new();

    let said = served.ask("resources/templates/list", json!({}));

    let templates = said["result"]["resourceTemplates"].as_array().unwrap();
    assert_eq!(templates.len(), 1, "{said}");
    assert_eq!(templates[0]["uriTemplate"], "tisty:doc/{doc}{?from,to}");
    assert_eq!(templates[0]["mimeType"], "text/markdown");
}
