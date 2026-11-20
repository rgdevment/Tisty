use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

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
        served.person().lets_the_agent_in();
        served
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn connect(&self) -> Connection {
        let root = self.home.path();
        let mut child = Command::new(env!("CARGO_BIN_EXE_tisty"))
            .arg("mcp")
            .env("TISTY_DATA", root.join("data"))
            .env("TISTY_CONFIG", root.join("config"))
            .env("TISTY_CACHE", root.join("cache"))
            .env("TZ", "America/Santiago")
            .env("NO_COLOR", "1")
            .env("LANG", "en_US.UTF-8")
            .env_remove("LC_ALL")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let into = child.stdin.take().unwrap();
        let from = BufReader::new(child.stdout.take().unwrap());
        Connection {
            child,
            into,
            from,
            next: 0,
        }
    }
}

struct Connection {
    child: Child,
    into: ChildStdin,
    from: BufReader<ChildStdout>,
    next: u64,
}

impl Connection {
    fn asked(&mut self, name: &str, args: Value) -> Value {
        self.next += 1;
        let asked = json!({
            "jsonrpc": "2.0", "id": self.next, "method": "tools/call",
            "params": { "name": name, "arguments": args },
        });
        writeln!(self.into, "{asked}").unwrap();
        self.into.flush().unwrap();
        let mut line = String::new();
        self.from.read_line(&mut line).unwrap();
        let said: Value = serde_json::from_str(&line).unwrap();
        said["result"].clone()
    }

    fn call(&mut self, name: &str, args: Value) -> Value {
        let said = self.asked(name, args);
        assert_ne!(said["isError"], true, "{name}: {said}");
        said
    }

    fn brain(&mut self, args: Value) -> Value {
        self.call("brain", args)["structuredContent"].clone()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn titles(found: &Value, key: &str) -> Vec<String> {
    found[key]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|one| one["title"].as_str().unwrap_or_default().to_string())
        .collect()
}

fn ids(found: &Value, key: &str) -> Vec<String> {
    found[key]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|one| one["id"].as_str().unwrap_or_default().to_string())
        .collect()
}

#[test]
fn a_title_that_says_every_word_ranks_first_even_from_the_archive() {
    let served = Served::new();
    let solved = served.person().captures("Fix the sync stall after sleep");
    served.person().closes(solved);
    let open = served.person().captures("Write the release notes");
    served
        .person()
        .notes(open, "Mention that the sync stall after sleep is gone.");
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "sync sleep" }));

    assert_eq!(
        titles(&found, "matches"),
        ["Fix the sync stall after sleep", "Write the release notes"]
    );
    assert_eq!(found["matches"][1]["in"], "journal");
    assert!(found.get("every_word").is_none(), "{found}");
}

#[test]
fn when_no_hit_holds_every_word_those_holding_the_most_come_said_so() {
    let served = Served::new();
    served
        .person()
        .captures("Restart the watcher when the machine wakes");
    served.person().captures("Restart the watcher every hour");
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "watcher wakes hibernate" }));

    assert_eq!(found["every_word"], false, "{found}");
    assert_eq!(
        titles(&found, "matches"),
        [
            "Restart the watcher when the machine wakes",
            "Restart the watcher every hour"
        ]
    );
    let said = talk.call("brain", json!({ "query": "watcher wakes hibernate" }));
    assert!(
        said["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("2 task(s)"),
        "{said}"
    );
}

#[test]
fn the_line_quoted_is_the_newest_that_still_holds() {
    let served = Served::new();
    let task = served.person().captures("Settle where the cache lives");
    let mut talk = served.connect();
    talk.call(
        "note",
        json!({ "task": task.to_string(), "body": "Decided: the cache lives in memory." }),
    );
    let read = talk.call("read", json!({ "task": task.to_string(), "notes": 0 }));
    let first = read["structuredContent"]["journal"][0]["id"]
        .as_str()
        .unwrap()
        .to_string();
    talk.call(
        "note",
        json!({ "task": task.to_string(), "body": "Decided again: the cache lives on disk.", "replaces": first }),
    );
    talk.call(
        "note",
        json!({ "task": task.to_string(), "body": "Ran the tests on the cache: all pass." }),
    );

    let found = talk.brain(json!({ "query": "decided cache" }));

    assert_eq!(
        found["matches"][0]["line"],
        "Decided again: the cache lives on disk."
    );
}

#[test]
fn a_document_card_brings_its_outline_print_and_who_cites_it() {
    let served = Served::new();
    let target = served
        .person()
        .writes_doc("# Sync design\n\n## Rounds\n\nEach round.\n\n## Conflicts\n\nCopies.\n");
    let citing = served.person().writes_doc(&format!(
        "# Sync notes\n\nSee [the design](tisty:doc/{target}).\n"
    ));
    let mut talk = served.connect();

    let card = talk.brain(json!({ "doc": target }));

    assert_eq!(card["title"], "Sync design");
    assert_eq!(card["outline"].as_array().unwrap().len(), 3);
    assert!(card["print"].as_str().is_some_and(|one| one.len() > 10));
    assert_eq!(card["mentioned_by"][0]["id"], json!(citing));
    assert_eq!(card["mentioned_by"][0]["title"], "Sync notes");
    assert!(card.get("pointed_at").is_none(), "{card}");
}

#[test]
fn what_is_hidden_never_shows_on_a_card_nor_in_a_search() {
    let served = Served::new();
    let target = served.person().writes_doc("# Budget\n\nNumbers.\n");
    let seen = served.person().captures("Plan the trip budget");
    served.person().notes(
        seen,
        &format!("Check [the budget](tisty:doc/{target}) first."),
    );
    let hidden = served.person().captures("Plan the secret budget gift");
    served.person().notes(
        hidden,
        &format!("Keep [the budget](tisty:doc/{target}) for the gift."),
    );
    served.person().hides(hidden);
    let mut talk = served.connect();

    let card = talk.brain(json!({ "doc": target, "links": "all" }));
    let found = talk.brain(json!({ "query": "budget" }));

    let around = card.to_string();
    assert!(around.contains(&seen.to_string()), "{around}");
    assert!(!around.contains(&hidden.to_string()), "{around}");
    assert!(!found.to_string().contains("secret"), "{found}");
    assert_eq!(found["total"], 1);
}

#[test]
fn a_task_card_says_its_state_stage_and_neighbours() {
    let served = Served::new();
    let mut talk = served.connect();
    let made = talk.call(
        "propose",
        json!({ "title": "Rank the search", "steps": ["Sort by place", "Quote the newest line"], "list": "Work" }),
    );
    let task = made["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    talk.call(
        "tick",
        json!({ "task": task, "steps": ["Sort by place"], "state": "Sorting by where it matched is done. Next: the newest line." }),
    );

    let card = talk.brain(json!({ "task": task }));

    assert_eq!(card["status"], "open");
    assert_eq!(card["stage"], "under_way");
    assert_eq!(card["steps"], json!([1, 2]));
    assert_eq!(
        card["state"],
        "Sorting by where it matched is done. Next: the newest line."
    );
    assert!(card["moved"].as_str().is_some(), "{card}");
    assert!(card["next"].as_str().unwrap().contains("`read`"));
}

#[test]
fn a_project_groups_its_open_parts_by_how_far_each_got() {
    let served = Served::new();
    let mut talk = served.connect();
    let made = talk.call(
        "propose",
        json!({ "title": "Ship the brain", "list": "Work", "parts": [
            { "title": "Rank", "steps": ["one", "two"] },
            { "title": "Cards", "steps": ["one"] },
            { "title": "Gates", "steps": ["one"] },
            { "title": "Docs", "steps": ["one"] },
            { "title": "Tests", "steps": ["one"] }
        ]}),
    );
    let whole = made["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    let part = |title: &str| served.person().id_of(title).to_string();
    talk.call("tick", json!({ "task": part("Rank"), "steps": ["one"] }));
    talk.call(
        "tick",
        json!({ "task": part("Cards"), "state": "Blocked: waiting for the owner to pick the cap." }),
    );
    talk.call("tick", json!({ "task": part("Gates"), "steps": ["one"] }));
    talk.call(
        "say_done",
        json!({ "task": part("Gates"), "body": "Gates hold on both ends." }),
    );
    served.person().closes(served.person().id_of("Docs"));

    let view = talk.brain(json!({ "part_of": whole }));

    assert_eq!(
        view["stages"],
        json!({ "blocked": 1, "under_way": 1, "waiting": 1, "not_started": 1 })
    );
    assert_eq!(view["closed"], 1);
    assert_eq!(titles(&view, "parts"), ["Cards", "Rank", "Gates", "Tests"]);
    assert_eq!(
        view["parts"][0]["state"],
        "Blocked: waiting for the owner to pick the cap."
    );
    assert_eq!(view["parts"][1]["steps"], json!([1, 2]));
}

#[test]
fn a_cut_outline_is_walked_one_chapter_at_a_time() {
    let served = Served::new();
    let mut body = String::from("# Log\n\n");
    for chapter in 1..=4 {
        body.push_str(&format!("## Month {chapter}\n\n"));
        for day in 1..=40 {
            body.push_str(&format!("### Day {day} of month {chapter}\n\nWork.\n\n"));
        }
    }
    let doc = served.person().writes_doc(&body);
    let mut talk = served.connect();

    let card = talk.brain(json!({ "doc": doc }));
    let left = card["outline_left"].as_u64().unwrap();
    let month = card["outline"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["title"] == "Month 2")
        .unwrap()["at"]
        .as_u64()
        .unwrap();
    let walked = talk.brain(json!({ "doc": doc, "section": month }));

    assert!(left > 0, "{card}");
    let under = walked["outline"].as_array().unwrap();
    assert_eq!(under.len(), 41);
    assert_eq!(under[0]["title"], "Month 2");
    assert!(
        under
            .iter()
            .skip(1)
            .all(|one| one["title"].as_str().unwrap().ends_with("month 2"))
    );
}

#[test]
fn nothing_asked_names_the_lists_folders_and_tags_with_their_counts() {
    let served = Served::new();
    let mut talk = served.connect();
    talk.call(
        "propose",
        json!({ "title": "Pay the rent", "list": "Work", "tags": ["home"] }),
    );
    talk.call("folder", json!({ "name": "Recipes" }));
    let hidden = served.person().captures("Hide this one #home");
    served.person().hides(hidden);

    let zones = talk.brain(json!({}));

    let work = zones["lists"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["list"] == "Work")
        .unwrap()
        .clone();
    assert_eq!(work["open"], 1);
    assert!(
        zones["folders"]
            .as_array()
            .unwrap()
            .iter()
            .any(|one| one["folder"] == "Recipes")
    );
    let home = zones["tags"]
        .as_array()
        .unwrap()
        .iter()
        .find(|one| one["tag"] == "home")
        .unwrap()
        .clone();
    assert_eq!(home["times"], 1);
}

#[test]
fn the_base_follows_a_write_made_a_moment_before() {
    let served = Served::new();
    let target = served.person().writes_doc("# Plan\n\nSteps.\n");
    let mut talk = served.connect();
    talk.call("folder", json!({ "name": "Work" }));
    let first = talk.brain(json!({ "doc": target }));
    assert!(first.get("mentioned_by").is_none(), "{first}");

    let wrote = talk.call(
        "write_doc",
        json!({ "body": format!("# Review\n\nRead [the plan](tisty:doc/{target})."), "folder": "Work", "anyway": true }),
    );
    let review = wrote["structuredContent"]["doc"]
        .as_str()
        .unwrap()
        .to_string();
    let card = talk.brain(json!({ "doc": target }));

    assert_eq!(card["mentioned_by"][0]["id"], json!(review));
}

#[test]
fn neighbours_are_capped_and_say_how_many_more() {
    let served = Served::new();
    let target = served.person().writes_doc("# Hub\n\nThe centre.\n");
    for n in 0..60 {
        served.person().writes_doc(&format!(
            "# Spoke {n}\n\nSee [the hub](tisty:doc/{target}).\n"
        ));
    }
    let mut talk = served.connect();

    let card = talk.brain(json!({ "doc": target }));
    let links = talk.brain(json!({ "doc": target, "links": "mention" }));

    assert_eq!(card["mentioned_by"].as_array().unwrap().len(), 8);
    assert_eq!(card["mentioned_by_more"], 52);
    assert!(card["next"].as_str().unwrap().contains("`links`"));
    assert_eq!(links["mentioned_by"].as_array().unwrap().len(), 50);
    assert_eq!(links["mentioned_by_more"], 10);
}

#[test]
fn a_task_cited_in_a_document_shows_on_its_card_with_title_and_state() {
    let served = Served::new();
    let task = served.person().captures("Call the bank");
    let doc = served
        .person()
        .writes_doc(&format!("# Money\n\n[Call the bank](tisty:task/{task})\n"));
    let mut talk = served.connect();

    let card = talk.brain(json!({ "doc": doc }));

    let cited = card["cites"]
        .as_array()
        .or_else(|| card["mentions"].as_array())
        .unwrap();
    assert_eq!(cited[0]["id"], json!(task.to_string()));
    assert_eq!(cited[0]["title"], "Call the bank");
    assert_eq!(cited[0]["state"], "open");
}

#[test]
fn a_folder_alone_lists_its_documents() {
    let served = Served::new();
    let mut talk = served.connect();
    talk.call("folder", json!({ "name": "Recipes" }));
    talk.call(
        "write_doc",
        json!({ "body": "# Pancakes\n\nFlour.\n", "folder": "Recipes" }),
    );

    let listed = talk.brain(json!({ "folder": "Recipes" }));

    assert_eq!(listed["total"], 1);
    assert_eq!(listed["docs"][0]["title"], "Pancakes");
}

#[test]
fn brain_looks_at_one_thing_at_a_time() {
    let served = Served::new();
    let task = served.person().captures("Call the bank");
    let mut talk = served.connect();

    let both = talk.asked("brain", json!({ "doc": "x-1", "task": task.to_string() }));
    let lone = talk.asked("brain", json!({ "links": "cite" }));
    let walk = talk.asked("brain", json!({ "task": task.to_string(), "section": 1 }));

    assert_eq!(both["isError"], true);
    assert_eq!(lone["isError"], true);
    assert_eq!(walk["isError"], true);
    assert!(
        lone["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("`links`")
    );
}

#[test]
fn ids_are_handed_back_as_the_card_holds_them() {
    let served = Served::new();
    let task = served.person().captures("Call the bank");
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "bank" }));

    assert_eq!(ids(&found, "matches"), [task.to_string()]);
}

#[test]
fn without_the_base_a_card_says_so_and_still_names_who_points_here() {
    let served = Served::new();
    let target = served.person().writes_doc("# Plan\n\nSteps.\n");
    let citing = served.person().writes_doc(&format!(
        "# Review\n\nRead [the plan](tisty:doc/{target}).\n"
    ));
    let task = served.person().captures("Call the bank");
    std::fs::create_dir_all(served.home.path().join("cache").join("brain.db")).unwrap();
    let mut talk = served.connect();

    let card = talk.brain(json!({ "doc": target }));
    let task_card = talk.brain(json!({ "task": task.to_string() }));
    let links = talk.asked("brain", json!({ "doc": target, "links": "all" }));
    let found = talk.brain(json!({ "query": "plan" }));

    assert!(
        card["base"].as_str().unwrap().contains("not ready"),
        "{card}"
    );
    assert_eq!(card["pointed_at"], json!([citing]));
    assert_eq!(task_card["title"], "Call the bank");
    assert!(task_card["base"].is_string(), "{task_card}");
    assert_eq!(links["isError"], true, "{links}");
    assert_eq!(found["docsTotal"], 2, "{found}");
    assert_eq!(found["docs"][0]["title"], "Plan", "a title first: {found}");
}

#[test]
fn a_tool_brain_took_over_answers_its_old_call_with_the_way_to_it() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Plan\n\nSteps.\n");
    let mut talk = served.connect();

    for (name, args) in [
        ("find", json!({ "query": "plan", "lines": false })),
        ("docs", json!({ "folders": true, "deep": true })),
        ("outline_doc", json!({ "doc": doc })),
        ("lists", json!({})),
        ("tags", json!({})),
    ] {
        let said = talk.asked(name, args);
        let text = said["content"][0]["text"].as_str().unwrap_or_default();
        assert_eq!(said["isError"], true, "{name}: {said}");
        assert!(
            text.starts_with(&format!("`{name}` moved into `brain`")),
            "{name}: {text}"
        );
    }
}

fn logged(served: &Served) -> String {
    let store = served.home.path().join("data").join("store");
    let mut said = String::new();
    for dir in std::fs::read_dir(store).unwrap().filter_map(Result::ok) {
        if let Ok(one) = std::fs::read_to_string(dir.path().join("active.tisty")) {
            said.push_str(&one);
        }
    }
    said
}

#[test]
fn a_tool_that_moved_writes_nothing() {
    let served = Served::new();
    let doc = served.person().writes_doc("# Plan\n\nSteps.\n");
    let mut talk = served.connect();
    talk.brain(json!({}));
    let before = logged(&served);

    for (name, args) in [
        ("find", json!({ "query": "plan" })),
        ("docs", json!({ "folders": true })),
        ("outline_doc", json!({ "doc": doc })),
        ("lists", json!({})),
        ("tags", json!({})),
        ("read_doc", json!({ "doc": doc, "section": 0 })),
    ] {
        let said = talk.asked(name, args);
        assert_eq!(said["isError"], true, "{name}: {said}");
    }

    assert_eq!(logged(&served), before, "nothing was written");
    talk.call("folder", json!({ "name": "Casa" }));
    assert_ne!(logged(&served), before, "as a write is");
}

#[test]
fn a_limit_past_a_hundred_brings_a_hundred_and_says_how_to_go_on() {
    let served = Served::new();
    for n in 0..105 {
        served.person().captures(&format!("Water plant {n}"));
    }
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "water", "limit": 500 }));

    assert_eq!(found["total"], 105, "{found}");
    assert_eq!(found["matches"].as_array().unwrap().len(), 100);
    assert!(
        found["next"].as_str().unwrap().contains("`after` 100"),
        "{found}"
    );
}

#[test]
fn a_long_line_is_quoted_cut_around_the_word_it_says() {
    let served = Served::new();
    let filler = "palabra ".repeat(60);
    served
        .person()
        .writes_doc(&format!("# Diario\n\n{filler}el riego de marzo {filler}\n"));
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "riego" }));

    let line = found["docs"][0]["line"].as_str().unwrap();
    assert!(line.contains("el riego de marzo"), "{line}");
    assert!(
        line.chars().count() <= 122,
        "a line of nearly a thousand characters is quoted around the word: {line}"
    );
}

#[test]
fn a_search_cut_short_says_how_to_go_on_and_empty_fields_are_named() {
    let served = Served::new();
    for n in 0..12 {
        served.person().captures(&format!("Water plant {n}"));
    }
    let mut talk = served.connect();

    let first = talk.brain(json!({ "query": "water" }));
    let rest = talk.brain(json!({ "query": "water", "after": 10 }));
    let empty = talk.asked("brain", json!({ "tag": "" }));

    assert_eq!(first["total"], 12);
    assert_eq!(first["matches"].as_array().unwrap().len(), 10);
    assert!(
        first["next"].as_str().unwrap().contains("`after` 10"),
        "{first}"
    );
    assert_eq!(rest["matches"].as_array().unwrap().len(), 2);
    assert!(rest.get("next").is_none(), "{rest}");
    assert_eq!(empty["isError"], true, "{empty}");
    assert!(
        empty["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("came empty")
    );
}

#[test]
fn a_section_is_asked_by_a_heading_the_card_numbers() {
    let served = Served::new();
    let doc = served
        .person()
        .writes_doc("# Plan\n\n## One\n\nA.\n\n## Two\n\nB.\n");
    let mut talk = served.connect();

    let word = talk.asked("brain", json!({ "doc": doc, "section": "one" }));
    let past = talk.asked("brain", json!({ "doc": doc, "section": 9 }));

    assert_eq!(word["isError"], true, "{word}");
    assert!(
        word["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("`at`")
    );
    assert_eq!(past["isError"], true, "{past}");
    assert!(
        past["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("from 0 to 2")
    );
}

#[test]
fn a_hidden_or_unknown_task_has_no_card() {
    let served = Served::new();
    let hidden = served.person().captures("Secret gift");
    served.person().hides(hidden);
    let mut talk = served.connect();

    let hid = talk.asked("brain", json!({ "task": hidden.to_string() }));
    let odd = talk.asked("brain", json!({ "task": "not-an-id" }));

    assert_eq!(hid["isError"], true, "{hid}");
    assert!(!hid.to_string().contains("Secret"), "{hid}");
    assert_eq!(odd["isError"], true, "{odd}");
}

#[test]
fn a_common_word_alone_brings_nothing_when_a_hit_holds_more() {
    let served = Served::new();
    served
        .person()
        .captures("Restart the watcher after hibernate");
    served.person().captures("Water the plants after lunch");
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "watcher restarted after hibernate" }));

    assert_eq!(found["every_word"], false, "{found}");
    assert_eq!(
        titles(&found, "matches"),
        ["Restart the watcher after hibernate"]
    );
    assert_eq!(found["total"], 1);
}

#[test]
fn words_nothing_holds_say_so_and_a_scope_lists_a_projects_parts_plainly() {
    let served = Served::new();
    let mut talk = served.connect();
    let made = talk.call(
        "propose",
        json!({ "title": "Ship it", "list": "Work", "parts": [
            { "title": "Rank", "steps": ["one"] },
            { "title": "Cards", "steps": ["one"] }
        ]}),
    );
    let whole = made["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    served.person().closes(served.person().id_of("Rank"));

    let none = talk.brain(json!({ "query": "zebra quokka" }));
    let closed = talk.brain(json!({ "part_of": whole, "scope": "archive" }));

    assert!(none.get("every_word").is_none(), "{none}");
    assert_eq!(none["total"], 0);
    assert!(closed.get("stages").is_none(), "{closed}");
    assert_eq!(titles(&closed, "matches"), ["Rank"]);
}

#[test]
fn a_task_page_asked_as_a_document_shows_its_tasks_neighbours() {
    let served = Served::new();
    let plan = served.person().writes_doc("# Plan\n\nSteps.\n");
    let task = served.person().captures("Follow the plan");
    let mut talk = served.connect();
    talk.call(
        "append_doc",
        json!({ "doc": task.to_string(), "body": format!("See [the plan](tisty:doc/{plan}).") }),
    );
    let read = talk.call("read", json!({ "task": task.to_string() }));
    let page = read["structuredContent"]["document"]["file"]
        .as_str()
        .unwrap()
        .to_string();

    let card = talk.brain(json!({ "doc": page }));

    assert!(card.get("base").is_none(), "{card}");
    assert_eq!(card["mentions"][0]["id"], json!(plan), "{card}");
    assert!(card.get("status").is_none(), "{card}");
}

#[test]
fn a_task_cards_outside_links_are_capped_like_its_neighbours() {
    let served = Served::new();
    let task = served.person().captures("Collect the sources");
    for n in 0..12 {
        served.person().notes(
            task,
            &format!("Read [source {n}](https://example.com/{n})."),
        );
    }
    let mut talk = served.connect();

    let card = talk.brain(json!({ "task": task.to_string() }));

    assert_eq!(card["links"].as_array().unwrap().len(), 8, "{card}");
    assert_eq!(card["links_more"], 4, "{card}");
}

#[test]
fn a_long_project_pages_on_and_says_how() {
    let served = Served::new();
    let mut talk = served.connect();
    let made = talk.call(
        "propose",
        json!({ "title": "Big", "list": "Work", "parts": [
            { "title": "One", "steps": ["a"] },
            { "title": "Two", "steps": ["a"] },
            { "title": "Three", "steps": ["a"] }
        ]}),
    );
    let whole = made["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let first = talk.brain(json!({ "part_of": whole, "limit": 2 }));
    let rest = talk.brain(json!({ "part_of": whole, "limit": 2, "after": 2 }));

    assert_eq!(first["parts"].as_array().unwrap().len(), 2);
    assert!(
        first["next"].as_str().unwrap().contains("`after` 2"),
        "{first}"
    );
    assert_eq!(rest["parts"].as_array().unwrap().len(), 1);
    assert!(rest.get("next").is_none(), "{rest}");
}

#[test]
fn zones_name_forty_tags_and_count_the_rest() {
    let served = Served::new();
    for n in 0..41 {
        served.person().captures(&format!("Task {n} #tag{n:02}"));
    }
    let mut talk = served.connect();

    let zones = talk.brain(json!({}));

    assert_eq!(zones["tags"].as_array().unwrap().len(), 40);
    assert_eq!(zones["tags_more"], 1);
}

#[test]
fn a_closed_task_has_no_stage_and_an_old_state_says_so() {
    let served = Served::new();
    let mut talk = served.connect();
    let made = talk.call(
        "propose",
        json!({ "title": "Write it", "steps": ["a"], "list": "Work" }),
    );
    let task = made["structuredContent"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    talk.call("tick", json!({ "task": task, "state": "Half written." }));
    talk.call("note", json!({ "task": task, "body": "Decided: shorter." }));

    let open = talk.brain(json!({ "task": task }));
    served.person().closes(task.parse().unwrap());
    let closed = talk.brain(json!({ "task": task }));

    assert_eq!(open["stage"], "under_way");
    assert_eq!(open["state_old"], true, "{open}");
    assert!(closed.get("stage").is_none(), "{closed}");
}

#[test]
fn putting_away_a_document_names_the_task_page_that_points_at_it() {
    let served = Served::new();
    let plan = served.person().writes_doc("# Plan\n\nSteps.\n");
    let task = served.person().captures("Follow the plan");
    let hidden = served.person().captures("Hidden follow-up");
    let mut talk = served.connect();
    for one in [task, hidden] {
        talk.call(
            "append_doc",
            json!({ "doc": one.to_string(), "body": format!("See [the plan](tisty:doc/{plan}).") }),
        );
    }
    let page = talk.call("read", json!({ "task": task.to_string() }))["structuredContent"]
        ["document"]["file"]
        .as_str()
        .unwrap()
        .to_string();
    served.person().hides(hidden);

    let put = talk.call("archive_doc", json!({ "doc": plan }));

    assert_eq!(put["structuredContent"]["pointed_at"], json!([page]));
}

#[test]
fn of_two_documents_saying_it_alike_the_newer_comes_first() {
    let served = Served::new();
    let older = served
        .person()
        .wrote_doc_days_ago("# Fruit\n\nThe kiwi is ripe.\n", 30);
    let newer = served
        .person()
        .wrote_doc_days_ago("# Market\n\nThe kiwi is cheap.\n", 1);
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "kiwi" }));

    assert_eq!(found["docs"][0]["doc"], json!(newer), "{found}");
    assert_eq!(found["docs"][1]["doc"], json!(older), "{found}");
}

#[test]
fn a_word_only_a_tasks_page_holds_still_counts_for_that_task() {
    let served = Served::new();
    let paged = served.person().captures("Invoice export");
    served.person().captures("Answer the stripe webhook");
    let mut talk = served.connect();
    talk.call(
        "append_doc",
        json!({ "doc": paged.to_string(), "body": "The stripe webhook sends it." }),
    );

    let found = talk.brain(json!({ "query": "invoice stripe webhook refund" }));

    assert_eq!(found["every_word"], false, "{found}");
    assert_eq!(
        found["matches"][0]["id"],
        json!(paged.to_string()),
        "{found}"
    );
}

#[test]
fn a_quoted_phrase_stays_whole_when_no_hit_holds_every_word() {
    let served = Served::new();
    served.person().captures("Fix the sync stall on wake");
    served.person().captures("The stall happens before sync");
    let mut talk = served.connect();

    let found = talk.brain(json!({ "query": "\"sync stall\" hibernate" }));

    assert_eq!(found["every_word"], false, "{found}");
    assert_eq!(titles(&found, "matches"), ["Fix the sync stall on wake"]);
}

fn guessed_between(served: &Served, rows: &[(String, String, f64, &str, &str)]) {
    let db = rusqlite::Connection::open(served.home.path().join("cache").join("brain.db")).unwrap();
    for (a, b, score, method, shared) in rows {
        db.execute(
            "INSERT INTO alike(a, b, score, shared, method) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![a, b, score, shared, method],
        )
        .unwrap();
    }
}

#[test]
fn a_card_lists_what_tisty_guesses_apart_and_never_names_a_hidden_task() {
    let served = Served::new();
    let budget = served.person().captures("Plan the family budget");
    let spending = served.person().captures("Count December spending");
    let twin = served.person().captures("Presupuesto familiar");
    let hidden = served.person().captures("Plan the secret gift");
    served.person().hides(hidden);
    let mut talk = served.connect();
    talk.brain(json!({ "task": budget.to_string() }));
    guessed_between(
        &served,
        &[
            (
                budget.to_string(),
                spending.to_string(),
                0.42,
                "words",
                "budget family",
            ),
            (
                budget.to_string(),
                twin.to_string(),
                0.91,
                "words",
                "familiar",
            ),
            (spending.to_string(), twin.to_string(), 0.97, "meaning", ""),
            (
                budget.to_string(),
                hidden.to_string(),
                0.99,
                "words",
                "plan",
            ),
        ],
    );

    let said = talk.call("brain", json!({ "task": budget.to_string() }));
    let card = &said["structuredContent"];
    let text = said["content"][0]["text"].as_str().unwrap();

    let guessed = card["guessed"].as_array().unwrap();
    assert_eq!(guessed.len(), 2, "{card}");
    assert_eq!(guessed[0]["id"], json!(twin.to_string()));
    assert_eq!(guessed[0]["shared"], json!(["familiar"]));
    assert!(guessed[0].get("by").is_none(), "{card}");
    assert_eq!(guessed[1]["title"], "Count December spending");
    assert_eq!(guessed[1]["shared"], json!(["budget", "family"]));
    assert_eq!(guessed[1]["score"], 0.42);
    assert!(
        text.contains("Guessed alike by Tisty, linked by no one"),
        "{text}"
    );
    assert!(text.contains("(shares familiar)"), "{text}");
    let spent = talk.brain(json!({ "task": spending.to_string() }));
    assert_eq!(spent["guessed"].as_array().unwrap().len(), 1, "{spent}");
    for told in [
        said.to_string(),
        talk.brain(json!({ "task": twin.to_string() })).to_string(),
    ] {
        assert!(!told.contains("secret gift"), "{told}");
        assert!(!told.contains(&hidden.to_string()), "{told}");
    }
    let links = talk.brain(json!({ "task": budget.to_string(), "links": "all" }));
    assert!(
        links.get("guessed").is_none(),
        "a guess is no link: {links}"
    );
}

#[test]
fn what_tisty_guesses_alike_stays_off_an_export_and_off_the_markdown_read_hands_a_tool() {
    let served = Served::new();
    let spanish = served
        .person()
        .writes_doc("# Pagar la cuenta del agua\n\nPagar la cuenta del agua de la casa.\n");
    let english = served
        .person()
        .writes_doc("# Pay the water bill\n\nPay the water bill of the house.\n");
    let mut talk = served.connect();
    talk.brain(json!({ "doc": spanish }));
    guessed_between(
        &served,
        &[(
            spanish.clone(),
            english.clone(),
            0.48,
            "dictionary",
            "agua≈water pagar≈pay casa≈house",
        )],
    );
    let card = talk.brain(json!({ "doc": spanish }));
    assert_eq!(card["guessed"][0]["id"], json!(english), "{card}");
    let out = tempfile::Builder::new()
        .tempdir_in(std::env::temp_dir())
        .unwrap();

    talk.call(
        "export_doc",
        json!({ "doc": spanish, "into": out.path().to_str().unwrap() }),
    );
    let markdown = talk.call("read", json!({ "doc": spanish, "as": "markdown" }));

    let mut told = vec![markdown.to_string()];
    let mut walk = vec![out.path().to_path_buf()];
    while let Some(at) = walk.pop() {
        for entry in std::fs::read_dir(&at).unwrap().flatten() {
            let path = entry.path();
            match path.is_dir() {
                true => walk.push(path),
                false => told.push(std::fs::read_to_string(&path).unwrap()),
            }
        }
    }
    assert!(told.len() > 1, "the export left no file");
    assert!(
        told.iter()
            .all(|one| one.contains("Pagar la cuenta del agua")),
        "{told:?}"
    );
    for gone in ["≈", "water bill", english.as_str()] {
        assert!(
            told.iter().all(|one| !one.contains(gone)),
            "{gone} went out: {told:?}"
        );
    }
}
