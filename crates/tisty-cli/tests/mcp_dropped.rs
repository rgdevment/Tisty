use std::io::Write;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use tempfile::TempDir;

mod person;

const NOBODY: &str = "01J00000000000000000000000";
const CARD_KEYS: [&str; 5] = ["id", "title", "status", "dropped", "reason"];
const PRIVATE_KEYS: [&str; 4] = ["document", "journal", "steps", "description"];

struct Served {
    home: TempDir,
}

impl Served {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let person = person::Person::at(home.path());
        person.begins("algo mio");
        person.lets_the_agent_in();
        Self { home }
    }

    fn person(&self) -> person::Person {
        person::Person::at(self.home.path())
    }

    fn call(&self, name: &str, args: Value) -> Value {
        let root = self.home.path();
        let asked = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": name, "arguments": args }
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
        let said: Value =
            serde_json::from_str(String::from_utf8_lossy(&out.stdout).trim()).unwrap();
        said["result"].clone()
    }

    fn files(&self, title: &str) -> String {
        let made = self.call(
            "propose",
            json!({ "title": title, "list": "Personal", "again": true }),
        );
        assert_eq!(made["structuredContent"]["proposed"], true, "{made}");
        made["structuredContent"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    }

    // The agent says it will not be done, and the person confirms by dropping it.
    fn dropped_for(&self, title: &str, why: &str) -> String {
        let id = self.files(title);
        let said = self.call("say_not_doing", json!({ "task": id, "body": why }));
        assert_ne!(said["isError"], true, "{said}");
        self.person().drops(person::task(&id));
        id
    }
}

fn text(said: &Value) -> String {
    said["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

fn card(said: &Value) -> &Value {
    &said["structuredContent"]
}

#[test]
fn reading_a_dropped_task_gives_only_its_card_and_the_reason_the_agent_gave() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    served
        .person()
        .describes(person::task(&id), "a private plan");
    served
        .person()
        .adds_step(person::task(&id), "a private step");

    let read = served.call("read", json!({ "task": id }));

    assert_ne!(read["isError"], true, "{read}");
    let card = card(&read);
    assert_eq!(card["title"], "migrate the billing database", "{read}");
    assert_eq!(card["status"], "dropped", "{read}");
    assert_eq!(card["reason"], "the vendor shut down", "{read}");
    assert!(card["dropped"].is_string(), "{read}");
    for key in card.as_object().unwrap().keys() {
        assert!(
            CARD_KEYS.contains(&key.as_str()) || key == "said",
            "unexpected key {key}: {read}"
        );
    }
    for key in PRIVATE_KEYS {
        assert!(card.get(key).is_none(), "{key} leaked: {read}");
    }
    let said = text(&read);
    assert!(said.contains("the person decided not to do it"), "{said}");
    assert!(!said.contains("a private"), "{said}");
}

#[test]
fn a_dropped_task_without_a_word_says_so() {
    let served = Served::new();
    let id = served.files("migrate the billing database");
    served.person().drops(person::task(&id));

    let read = served.call("read", json!({ "task": id }));

    assert!(card(&read).get("reason").is_none(), "{read}");
    assert!(text(&read).contains("they wrote no reason"), "{read}");
}

#[test]
fn a_note_of_the_day_it_was_dropped_stands_for_the_reason() {
    let served = Served::new();
    let id = served.files("migrate the billing database");
    served
        .person()
        .notes(person::task(&id), "not worth it anymore");
    served.person().drops(person::task(&id));

    let read = served.call("read", json!({ "task": id }));

    assert_eq!(card(&read)["reason"], "not worth it anymore", "{read}");
}

#[test]
fn brain_with_the_task_gives_the_same_card() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");

    let brain = served.call("brain", json!({ "task": id }));

    assert_ne!(brain["isError"], true, "{brain}");
    assert_eq!(card(&brain)["status"], "dropped", "{brain}");
    assert_eq!(card(&brain)["reason"], "the vendor shut down", "{brain}");
    for key in PRIVATE_KEYS {
        assert!(card(&brain).get(key).is_none(), "{key} leaked: {brain}");
    }
}

#[test]
fn a_query_lists_a_dropped_task_apart_from_the_matches() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    served.files("write the quarterly report");

    let found = served.call("brain", json!({ "query": "migrate billing database" }));

    let whole = card(&found);
    let dropped = whole["dropped"]
        .as_array()
        .unwrap_or_else(|| panic!("{found}"));
    assert_eq!(dropped.len(), 1, "{found}");
    assert_eq!(dropped[0]["id"], id, "{found}");
    assert_eq!(dropped[0]["reason"], "the vendor shut down", "{found}");
    assert!(
        !whole["matches"].to_string().contains("billing"),
        "listed among the matches: {found}"
    );
    assert!(text(&found).contains("Decided not to do"), "{found}");
}

#[test]
fn a_query_finds_a_dropped_task_by_its_reason() {
    let served = Served::new();
    served.dropped_for("migrate the billing database", "the vendor shut down");

    let found = served.call("brain", json!({ "query": "vendor" }));

    assert_eq!(
        card(&found)["dropped"].as_array().unwrap().len(),
        1,
        "{found}"
    );
}

#[test]
fn an_open_scope_never_lists_dropped_tasks() {
    let served = Served::new();
    served.dropped_for("migrate the billing database", "the vendor shut down");

    let found = served.call(
        "brain",
        json!({ "query": "migrate billing database", "scope": "open" }),
    );

    assert!(card(&found).get("dropped").is_none(), "{found}");
    assert!(!text(&found).contains("Decided not to do"), "{found}");
}

#[test]
fn a_query_that_matches_nothing_dropped_adds_nothing() {
    let served = Served::new();
    served.dropped_for("migrate the billing database", "the vendor shut down");

    let found = served.call("brain", json!({ "query": "garden fence" }));

    assert!(card(&found).get("dropped").is_none(), "{found}");
}

#[test]
fn no_more_than_three_dropped_cards_come_with_a_query() {
    let served = Served::new();
    for n in 1..=5 {
        served.dropped_for(&format!("migrate billing database {n}"), "stopped");
    }

    let found = served.call("brain", json!({ "query": "migrate billing database" }));

    assert_eq!(
        card(&found)["dropped"].as_array().unwrap().len(),
        3,
        "{found}"
    );
}

#[test]
fn a_hidden_dropped_task_is_named_nowhere() {
    let served = Served::new();
    let id = served.dropped_for("secret zebra gift", "secret reason");
    served.person().hides(person::task(&id));

    let read = served.call("read", json!({ "task": id }));
    let by_task = served.call("brain", json!({ "task": id }));
    let by_query = served.call("brain", json!({ "query": "secret zebra gift" }));
    let nobody_read = served.call("read", json!({ "task": NOBODY }));
    let nobody_task = served.call("brain", json!({ "task": NOBODY }));
    let proposed = served.call(
        "propose",
        json!({ "title": "secret zebra gift again", "list": "Personal" }),
    );

    for said in [&read, &by_task, &by_query] {
        let all = said.to_string();
        assert!(!all.contains("secret reason"), "{said}");
        assert!(card(said).get("dropped").is_none(), "{said}");
    }
    assert_eq!(
        read.to_string().replace(&id, NOBODY),
        nobody_read.to_string()
    );
    assert_eq!(
        by_task.to_string().replace(&id, NOBODY),
        nobody_task.to_string()
    );
    assert!(
        !by_query.to_string().contains("Decided not to do"),
        "{by_query}"
    );
    assert!(!proposed.to_string().contains(&id), "{proposed}");
    assert!(card(&proposed).get("dropped_alike").is_none(), "{proposed}");
    assert_eq!(card(&proposed)["proposed"], true, "{proposed}");
}

#[test]
fn a_dropped_task_refuses_to_be_written() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    let before = served.person().ops().len();

    let noted = served.call("note", json!({ "task": id, "body": "more words" }));
    let ticked = served.call("tick", json!({ "task": id, "step": "a step" }));
    let reminded = served.call("remind", json!({ "task": id, "at": ["2030-01-01T10:00"] }));

    for said in [&noted, &ticked, &reminded] {
        assert_eq!(said["isError"], true, "{said}");
        let why = text(said);
        assert!(why.contains("dropped"), "{why}");
        assert!(why.contains("migrate the billing database"), "{why}");
    }
    assert_eq!(served.person().ops().len(), before);
}

#[test]
fn a_title_alike_to_a_dropped_one_is_held_until_again_is_set() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    let before = served.person().ops().len();

    let held = served.call(
        "propose",
        json!({ "title": "Migrate billing database", "list": "Personal" }),
    );

    assert_eq!(card(&held)["proposed"], false, "{held}");
    assert_eq!(card(&held)["dropped_alike"]["id"], id, "{held}");
    assert!(text(&held).contains("Nothing was written"), "{held}");
    assert_eq!(served.person().ops().len(), before);

    let filed = served.call(
        "propose",
        json!({ "title": "Migrate billing database", "list": "Personal", "again": true }),
    );

    assert_eq!(card(&filed)["proposed"], true, "{filed}");
    assert!(served.person().ops().len() > before);
}

#[test]
fn an_unrelated_title_is_filed_without_a_word_about_dropped_tasks() {
    let served = Served::new();
    served.dropped_for("migrate the billing database", "the vendor shut down");

    let filed = served.call(
        "propose",
        json!({ "title": "paint the garden fence", "list": "Personal" }),
    );

    assert_eq!(card(&filed)["proposed"], true, "{filed}");
    assert!(card(&filed).get("dropped_alike").is_none(), "{filed}");
}

#[test]
fn the_batch_form_says_which_line_was_not_written() {
    let served = Served::new();
    served.dropped_for("migrate the billing database", "the vendor shut down");

    let batch = served.call(
        "propose",
        json!({ "tasks": [
            { "title": "Migrate billing database", "list": "Personal" },
            { "title": "paint the garden fence", "list": "Personal" },
        ] }),
    );

    let said = text(&batch);
    assert!(
        said.starts_with("1 written, 0 already there, 1 turned away."),
        "{said}"
    );
    assert!(said.contains("not written"), "{said}");
    assert!(said.contains("`again` files it anyway"), "{said}");
    assert!(said.contains("paint the garden fence"), "{said}");
    let state = served.person().state();
    assert!(
        !state
            .tasks
            .values()
            .any(|t| t.title == "Migrate billing database")
    );
    assert!(
        state
            .tasks
            .values()
            .any(|t| t.title == "paint the garden fence")
    );
}

#[test]
fn a_shared_prefix_alone_does_not_hold_a_proposal() {
    let served = Served::new();
    served.dropped_for("Tisty MCP: import", "not needed");

    let filed = served.call(
        "propose",
        json!({ "title": "Tisty MCP: export", "list": "Personal" }),
    );

    assert_eq!(card(&filed)["proposed"], true, "{filed}");
}

fn assert_says_dropped(said: &Value, title: &str) {
    assert_eq!(said["isError"], true, "{said}");
    let why = text(said);
    assert!(why.contains("was dropped"), "{why}");
    assert!(why.contains(title), "{why}");
    assert!(!why.contains("no task here has the id"), "{why}");
}

fn assert_says_nothing_here(said: &Value) {
    assert_eq!(said["isError"], true, "{said}");
    assert!(text(said).contains("no task here has the id"), "{said}");
}

#[test]
fn attaching_to_a_dropped_task_says_it_was_dropped() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    let before = served.person().ops().len();

    let said = served.call(
        "attach",
        json!({ "task": id, "path": served.home.path().join("a.txt").to_string_lossy() }),
    );

    assert_says_dropped(&said, "migrate the billing database");
    assert_eq!(served.person().ops().len(), before);
}

#[test]
fn proposing_a_part_of_a_dropped_task_says_it_was_dropped() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    let before = served.person().ops().len();

    let said = served.call(
        "propose",
        json!({ "title": "paint the garden fence", "part_of": id }),
    );

    assert_says_dropped(&said, "migrate the billing database");
    assert_eq!(served.person().ops().len(), before);
}

#[test]
fn moving_under_a_dropped_task_says_it_was_dropped() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    let other = served.files("paint the garden fence");
    let before = served.person().ops().len();

    let said = served.call("move", json!({ "task": other, "part_of": id }));

    assert_says_dropped(&said, "migrate the billing database");
    assert_eq!(served.person().ops().len(), before);
}

#[test]
fn hidden_and_unknown_ids_keep_the_plain_answer() {
    let served = Served::new();
    let secret = served.dropped_for("secret zebra gift", "secret reason");
    served.person().hides(person::task(&secret));
    let other = served.files("paint the garden fence");
    let path = served
        .home
        .path()
        .join("a.txt")
        .to_string_lossy()
        .into_owned();

    for gone in [secret.as_str(), NOBODY] {
        let attached = served.call("attach", json!({ "task": gone, "path": path }));
        let proposed = served.call(
            "propose",
            json!({ "title": "something else", "part_of": gone }),
        );
        let moved = served.call("move", json!({ "task": other, "part_of": gone }));
        for said in [&attached, &proposed, &moved] {
            assert_says_nothing_here(said);
            assert!(!said.to_string().contains("secret"), "{said}");
        }
    }
}

#[test]
fn a_note_on_a_dropped_task_says_how_it_ended_not_that_it_is_put_away() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down.");

    let noted = served.call("note", json!({ "task": id, "body": "more words" }));

    assert_eq!(noted["isError"], true, "{noted}");
    let why = text(&noted);
    assert!(why.contains("dropped"), "{why}");
    assert!(!why.contains("does not reach"), "{why}");
    assert!(!why.contains(".."), "{why}");
}

#[test]
fn a_note_on_a_hidden_task_still_says_it_is_put_away() {
    let served = Served::new();
    let id = served.files("secret zebra gift");
    served.person().hides(person::task(&id));

    let noted = served.call("note", json!({ "task": id, "body": "more words" }));

    assert_eq!(noted["isError"], true, "{noted}");
    assert!(!text(&noted).contains("secret zebra"), "{noted}");
}

#[test]
fn reschedule_and_say_done_on_a_dropped_task_are_refused_and_write_nothing() {
    let served = Served::new();
    let id = served.dropped_for("migrate the billing database", "the vendor shut down");
    let before = served.person().ops().len();

    let moved = served.call("reschedule", json!({ "task": id, "date": "2030-01-01" }));
    let done = served.call("say_done", json!({ "task": id, "body": "did it" }));

    for said in [&moved, &done] {
        assert_says_dropped(said, "migrate the billing database");
        assert!(text(said).contains("takes no writing"), "{said}");
    }
    assert_eq!(served.person().ops().len(), before);
}

#[test]
fn brain_part_of_a_dropped_whole_gives_the_card_and_no_parts() {
    let served = Served::new();
    let whole = served.files("release plan");
    served.call(
        "propose",
        json!({ "title": "write the changelog", "part_of": whole }),
    );
    served.person().drops(person::task(&whole));

    let seen = served.call("brain", json!({ "part_of": whole }));

    assert_says_dropped(&seen, "release plan");
    assert!(!text(&seen).contains("changelog"), "{seen}");
}

#[test]
fn a_part_alike_to_a_dropped_task_is_refused_unless_the_whole_says_again() {
    let served = Served::new();
    served.dropped_for("migrate the billing database", "the vendor shut down");
    let before = served.person().ops().len();
    let args = |again: bool| {
        let mut one = json!({
            "title": "quarterly cleanup",
            "list": "Personal",
            "parts": [{ "title": "Migrate billing database" }],
        });
        if again {
            one["again"] = json!(true);
        }
        one
    };

    let refused = served.call("propose", args(false));

    assert_eq!(refused["isError"], true, "{refused}");
    let why = text(&refused);
    assert!(why.contains("reads as a task the person dropped"), "{why}");
    assert!(why.contains("Nothing was written"), "{why}");
    assert_eq!(served.person().ops().len(), before);
    let state = served.person().state();
    assert!(!state.tasks.values().any(|t| t.title == "quarterly cleanup"));

    let filed = served.call("propose", args(true));

    assert_ne!(filed["isError"], true, "{filed}");
    let state = served.person().state();
    assert!(state.tasks.values().any(|t| t.title == "quarterly cleanup"));
    assert!(
        state
            .tasks
            .values()
            .any(|t| t.title == "Migrate billing database" && !t.is_dropped())
    );
}

#[test]
fn hidden_ids_keep_the_plain_answer_on_note_and_brain_part_of() {
    let served = Served::new();
    let id = served.dropped_for("secret zebra gift", "secret reason");
    served.person().hides(person::task(&id));

    for said in [
        served.call("note", json!({ "task": id, "body": "more" })),
        served.call("brain", json!({ "part_of": id })),
        served.call("reschedule", json!({ "task": id, "date": "2030-01-01" })),
    ] {
        assert_eq!(said["isError"], true, "{said}");
        assert!(!said.to_string().contains("secret"), "{said}");
        assert!(!text(&said).contains("was dropped"), "{said}");
    }
}
